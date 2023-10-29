use std::mem;
use std::ops::RangeInclusive;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Ident, Index, Token, Visibility};

use super::ast;

#[derive(Debug, Clone)]
pub struct LocationVariant {
    #[allow(dead_code)]
    type_ident: Ident,
    variant_ident: Ident,
    display_name: String,
    spell_range: Option<RangeInclusive<u32>>,
    full_path: TokenStream,
}

impl LocationVariant {
    fn new(
        type_ident: Ident,
        variant_ident: Ident,
        display_name: String,
        spell_range: Option<RangeInclusive<u32>>,
    ) -> Self {
        let full_path = quote! { #type_ident::#variant_ident };
        Self {
            type_ident,
            variant_ident,
            display_name,
            spell_range,
            full_path,
        }
    }

    pub fn new_start(type_ident: Ident) -> Self {
        Self::new(
            type_ident,
            format_ident!("Start"),
            String::from("Start"),
            None,
        )
    }

    pub fn new_basic_section(
        type_ident: Ident,
        second_half_start: Option<u32>,
        seq: u32,
        override_name: Option<String>,
    ) -> Self {
        let prefix = if second_half_start.is_some() {
            "Second"
        } else {
            "First"
        };

        let seq_num = if let Some(second_half_start) = second_half_start {
            seq.saturating_sub(second_half_start)
        } else {
            seq
        };

        Self::new(
            type_ident,
            format_ident!("{}Half{}", prefix, seq_num + 1),
            override_name.unwrap_or_else(|| format!("{} Half {}", prefix, seq_num + 1)),
            None,
        )
    }

    pub fn new_boss_spells(
        type_ident: Ident,
        midboss: bool,
        seq: u32,
        spell_range: RangeInclusive<u32>,
    ) -> Self {
        let prefix = if midboss { "Midboss" } else { "Boss" };
        Self::new(
            type_ident,
            format_ident!("{}Spell{}", prefix, seq + 1),
            format!("{} Spell {}", prefix, seq + 1),
            Some(spell_range),
        )
    }

    pub fn new_boss_last_spell(
        type_ident: Ident,
        seq: Option<u32>,
        spell_range: RangeInclusive<u32>,
    ) -> Self {
        if let Some(seq) = seq {
            Self::new(
                type_ident,
                format_ident!("LastSpell{}", seq + 1),
                format!("Last Spell {}", seq + 1),
                Some(spell_range),
            )
        } else {
            Self::new(
                type_ident,
                Ident::new("LastSpell", Span::call_site()),
                String::from("Last Spell"),
                Some(spell_range),
            )
        }
    }

    pub fn new_boss_nonspell(type_ident: Ident, midboss: bool, seq: u32) -> Self {
        let prefix = if midboss { "Midboss" } else { "Boss" };

        Self::new(
            type_ident,
            format_ident!("{}Nonspell{}", prefix, seq + 1),
            format!("{} Nonspell {}", prefix, seq + 1),
            None,
        )
    }

    pub fn variant_ident(&self) -> &Ident {
        &self.variant_ident
    }

    pub fn full_path(&self) -> &TokenStream {
        &self.full_path
    }

    pub fn needs_spell_id(&self) -> bool {
        self.spell_range.is_some()
    }

    pub fn spell_range(&self) -> Option<&RangeInclusive<u32>> {
        self.spell_range.as_ref()
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn match_pattern<'a>(
        &self,
        spell_capture_ident: Option<&'a Ident>,
    ) -> (Option<&'a Ident>, TokenStream) {
        let path = self.full_path();
        if self.needs_spell_id() {
            if let Some(cap_ident) = spell_capture_ident {
                (spell_capture_ident, quote! { #path(#cap_ident) })
            } else {
                (None, quote! { #path(_) })
            }
        } else {
            (None, quote! { #path })
        }
    }
}

fn range_to_tokens<Idx: ToTokens>(range: &RangeInclusive<Idx>) -> TokenStream {
    let start = range.start();
    let end = range.end();
    quote! { #start..=#end }
}

#[derive(Debug, Clone)]
pub enum BossPhase {
    Nonspell {
        variant: LocationVariant,
    },
    Spells {
        variant: LocationVariant,
        spell_ids: RangeInclusive<u32>,
    },
    LastSpell {
        variant: LocationVariant,
        spell_ids: RangeInclusive<u32>,
    },
}

impl BossPhase {
    pub fn variant(&self) -> &LocationVariant {
        match self {
            Self::Nonspell { variant, .. }
            | Self::Spells { variant, .. }
            | Self::LastSpell { variant, .. } => variant,
        }
    }

    pub fn match_result(&self) -> &TokenStream {
        self.variant().full_path()
    }
}

#[derive(Debug, Clone)]
pub struct BossFight {
    midboss: bool,
    phases: Vec<BossPhase>,
}

impl BossFight {
    pub fn to_fallback_match_result(&self) -> TokenStream {
        match &self.phases[0] {
            BossPhase::Nonspell { variant } => {
                let full_path = variant.full_path();
                quote! { Some(#full_path) }
            }
            BossPhase::Spells { .. } | BossPhase::LastSpell { .. } => quote! { None },
        }
    }

    pub fn to_resolve_arm(&self, state_ident: &Ident, fallback_result: TokenStream) -> TokenStream {
        let mut prev_was_nonspell = false;
        let mut n_healthbars: u32 =
            self.phases
                .iter()
                .enumerate()
                .fold(0, move |n_healthbars, (idx, phase)| match phase {
                    BossPhase::Nonspell { .. } => {
                        prev_was_nonspell = true;
                        n_healthbars + 1
                    }
                    BossPhase::Spells { .. } => {
                        if mem::replace(&mut prev_was_nonspell, false)
                            || (idx == self.phases.len() - 1)
                        {
                            n_healthbars
                        } else {
                            n_healthbars + 1
                        }
                    }
                    BossPhase::LastSpell { .. } => n_healthbars,
                });

        let spell_ranges: Vec<_> = self
            .phases
            .iter()
            .filter_map(|phase| {
                if let BossPhase::Spells { spell_ids, .. }
                | BossPhase::LastSpell { spell_ids, .. } = phase
                {
                    let result = phase.match_result();
                    let id_pattern = range_to_tokens(spell_ids);
                    Some(quote! {
                        Some((#id_pattern, spell)) => Some(#result(spell))
                    })
                } else {
                    None
                }
            })
            .collect();

        let nonspells: Vec<_> = self
            .phases
            .iter()
            .filter_map(|phase| match phase {
                BossPhase::Nonspell { .. } => {
                    prev_was_nonspell = true;

                    n_healthbars = n_healthbars.saturating_sub(1);
                    let healthbar = n_healthbars as u8;

                    let result = phase.match_result();
                    Some(quote! {
                        #healthbar => Some(#result)
                    })
                }
                BossPhase::Spells { .. } => {
                    if !mem::replace(&mut prev_was_nonspell, false) {
                        n_healthbars = n_healthbars.saturating_sub(1);
                    }

                    None
                }
                BossPhase::LastSpell { .. } => None,
            })
            .collect();

        let nonspell_match = if nonspells.is_empty() {
            quote! { None }
        } else {
            quote! {
                match boss.remaining_lifebars() {
                    #(#nonspells,)*
                    _ => None
                }
            }
        };

        if spell_ranges.is_empty() {
            quote! {
                {
                    use crate::memory::traits::{StageData, BossData, BossLifebars};

                    if let Some(boss) = #state_ident.active_boss() {
                        #nonspell_match
                    } else {
                        #fallback_result
                    }
                },
            }
        } else {
            quote! {
                {
                    use crate::memory::traits::{StageData, BossData, BossLifebars};

                    if let Some(boss) = #state_ident.active_boss() {
                        match boss.active_spell().map(|state| (state.raw_spell_id(), state.spell())) {
                            #(#spell_ranges,)*
                            Some(_) => None,
                            None => #nonspell_match
                        }
                    } else {
                        #fallback_result
                    }
                },
            }
        }
    }
}

#[derive(Debug)]
pub enum FrameSpanIter<'a> {
    Single(Option<&'a LocationVariant>),
    Boss(bool, std::slice::Iter<'a, BossPhase>),
}

impl<'a> Iterator for FrameSpanIter<'a> {
    type Item = &'a LocationVariant;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Single(inner) => inner.take(),
            Self::Boss(_midboss, inner) => inner.next().map(|phase| phase.variant()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FrameSpanType {
    Single(LocationVariant),
    Boss(BossFight),
}

impl FrameSpanType {
    fn to_fallback_match_result(&self) -> TokenStream {
        match self {
            Self::Single(variant) => {
                let path = variant.full_path();
                quote! { Some(#path) }
            }
            Self::Boss(fight) => fight.to_fallback_match_result(),
        }
    }

    fn to_resolve_arm(&self, state_ident: &Ident, fallback_result: TokenStream) -> TokenStream {
        match self {
            Self::Single(variant) => {
                let path = variant.full_path();
                quote! { Some(#path), }
            }
            Self::Boss(fight) => fight.to_resolve_arm(state_ident, fallback_result),
        }
    }

    pub fn iter_variants(&self) -> FrameSpanIter<'_> {
        match self {
            Self::Single(variant) => FrameSpanIter::Single(Some(variant)),
            Self::Boss(fight) => FrameSpanIter::Boss(fight.midboss, fight.phases.iter()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FrameSpan {
    start_frame: u32,
    span_type: FrameSpanType,
}

impl FrameSpan {
    fn to_time_match_arm(
        &self,
        state_ident: &Ident,
        next_span: Option<&FrameSpan>,
        fallback_span: Option<&FrameSpan>,
    ) -> TokenStream {
        let fallback_result = fallback_span
            .or(next_span)
            .map(|span| span.span_type.to_fallback_match_result())
            .unwrap_or_else(|| quote! { None });

        let resolve_arm = self.span_type.to_resolve_arm(state_ident, fallback_result);
        if let Some(end_frame) = next_span.map(|span| span.start_frame - 1) {
            let frames = range_to_tokens(&(self.start_frame..=end_frame));
            quote! {
                #frames => #resolve_arm
            }
        } else {
            let start_frame = self.start_frame;
            quote! { #start_frame..  => #resolve_arm }
        }
    }

    pub fn iter_variants(&self) -> FrameSpanIter<'_> {
        self.span_type.iter_variants()
    }
}

#[derive(Debug)]
pub struct StageState {
    type_ident: Ident,
    midboss_seq: Option<(u32, u32)>,
    boss_seq: Option<(u32, u32)>,
    second_half_start: Option<u32>,
    stage_seq: u32,
    has_nonspells: bool,
    frame_spans: Vec<FrameSpan>,
}

impl StageState {
    fn new(type_ident: Ident) -> Self {
        Self {
            midboss_seq: None,
            boss_seq: None,
            stage_seq: 0,
            has_nonspells: false,
            second_half_start: None,
            frame_spans: vec![FrameSpan {
                start_frame: 0,
                span_type: FrameSpanType::Single(LocationVariant::new_start(type_ident.clone())),
            }],
            type_ident,
        }
    }

    fn push_stage(
        &mut self,
        frame_number: u32,
        _err_span: Span,
        _def: &ast::SectionDef,
        name: Option<String>,
    ) -> Result<(), syn::Error> {
        self.frame_spans.push(FrameSpan {
            start_frame: frame_number,
            span_type: FrameSpanType::Single(LocationVariant::new_basic_section(
                self.type_ident.clone(),
                self.second_half_start,
                self.stage_seq,
                name,
            )),
        });

        self.stage_seq += 1;

        Ok(())
    }

    fn push_boss(
        &mut self,
        err_span: Span,
        def: &ast::BossDef,
        frame_number: u32,
        midboss: bool,
    ) -> Result<(), syn::Error> {
        use ast::BossPhaseDef;

        if midboss && self.boss_seq.is_some() {
            return Err(syn::Error::new(
                err_span,
                "cannot define midboss section after boss fight",
            ));
        }

        if self.second_half_start.is_none() {
            self.second_half_start = Some(self.stage_seq);
        }

        let seq_numbers = if midboss {
            self.midboss_seq.get_or_insert((0, 0))
        } else {
            self.boss_seq.get_or_insert((0, 0))
        };

        let mut phases = Vec::with_capacity(def.phases.len());
        for phase_def in &def.phases {
            match phase_def {
                BossPhaseDef::Nonspell { .. } => {
                    let phase = BossPhase::Nonspell {
                        variant: LocationVariant::new_boss_nonspell(
                            self.type_ident.clone(),
                            midboss,
                            seq_numbers.0,
                        ),
                    };
                    seq_numbers.0 += 1;
                    self.has_nonspells = true;
                    phases.push(phase);
                }
                BossPhaseDef::Spells { range, .. } => {
                    let spell_ids = range.parse_range()?;
                    let phase = BossPhase::Spells {
                        variant: LocationVariant::new_boss_spells(
                            self.type_ident.clone(),
                            midboss,
                            seq_numbers.1,
                            spell_ids.clone(),
                        ),
                        spell_ids,
                    };
                    seq_numbers.1 += 1;
                    phases.push(phase);
                }
                BossPhaseDef::LastSpell { ranges, .. } => {
                    for (idx, range) in ranges.iter().enumerate() {
                        let spell_ids = range.parse_range()?;
                        let seq = if ranges.len() > 1 {
                            Some(idx as u32)
                        } else {
                            None
                        };

                        phases.push(BossPhase::LastSpell {
                            variant: LocationVariant::new_boss_last_spell(
                                self.type_ident.clone(),
                                seq,
                                spell_ids.clone(),
                            ),
                            spell_ids,
                        })
                    }
                }
            };
        }

        self.frame_spans.push(FrameSpan {
            start_frame: frame_number,
            span_type: FrameSpanType::Boss(BossFight { midboss, phases }),
        });

        Ok(())
    }

    pub fn push_ast(
        &mut self,
        frame_number: u32,
        entry: &ast::SectionEntry,
    ) -> Result<(), syn::Error> {
        use ast::SectionDef;

        match &entry.def {
            SectionDef::Basic { name, .. } => self.push_stage(
                frame_number,
                entry.frame_number.span(),
                &entry.def,
                name.as_ref().map(|(_, s)| s.value()),
            ),
            SectionDef::Midboss { def, .. } => {
                self.push_boss(entry.frame_number.span(), def, frame_number, true)
            }
            SectionDef::Boss { def, .. } => {
                self.push_boss(entry.frame_number.span(), def, frame_number, false)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct StageLocations {
    game_type: Ident,
    type_ident: Ident,
    stage_ident: Ident,
    spell_id_ident: Ident,
    has_nonspells: bool,
    frame_spans: Vec<FrameSpan>,
}

impl StageLocations {
    pub fn from_ast(
        game_type: Ident,
        spell_id_ident: Ident,
        def: &ast::StageDef,
    ) -> Result<Self, syn::Error> {
        let type_ident = def
            .override_type_name
            .clone()
            .unwrap_or_else(|| format_ident!("Stage{}", &def.stage_id));
        let mut state = StageState::new(type_ident.clone());
        let mut entries: Vec<(u32, _)> = def
            .sections
            .iter()
            .map(|entry| {
                entry
                    .frame_number
                    .base10_parse()
                    .map(|frame| (frame, entry))
            })
            .collect::<Result<Vec<_>, _>>()?;

        entries.sort_by_key(|kv| kv.0);
        for (frame_number, entry) in entries {
            state.push_ast(frame_number, entry)?;
        }

        Ok(Self {
            game_type,
            type_ident,
            spell_id_ident,
            stage_ident: def.stage_id.clone(),
            has_nonspells: state.has_nonspells,
            frame_spans: state.frame_spans,
        })
    }

    pub fn iter_variants(&self) -> impl Iterator<Item = &LocationVariant> + '_ {
        self.frame_spans
            .iter()
            .flat_map(|span| span.iter_variants())
    }

    fn iter_match_patterns(
        &self,
        capture_spell_ids: bool,
    ) -> impl Iterator<Item = (&LocationVariant, Option<Ident>, TokenStream)> + '_ {
        let capture_name = if capture_spell_ids {
            Some(format_ident!("spell"))
        } else {
            None
        };

        self.iter_variants().map(move |variant| {
            let pattern = variant.match_pattern(capture_name.as_ref());
            (variant, pattern.0.cloned(), pattern.1)
        })
    }

    fn resolve_match_arms(&self, state_ident: &Ident) -> TokenStream {
        let mut ret = TokenStream::new();
        let mut fallback_span = None;
        let mut iter = self.frame_spans.iter().peekable();

        while let Some(frame_span) = iter.next() {
            if matches!(frame_span.span_type, FrameSpanType::Single(_)) {
                fallback_span = Some(frame_span);
            }

            let fallback = if matches!(
                frame_span.span_type,
                FrameSpanType::Boss(BossFight { midboss: false, .. })
            ) {
                None
            } else {
                fallback_span
            };

            ret.extend(frame_span.to_time_match_arm(state_ident, iter.peek().copied(), fallback));
        }

        ret
    }

    fn iter_spell_variants(
        &self,
    ) -> impl Iterator<Item = (&'_ LocationVariant, RangeInclusive<u32>)> + '_ {
        self.frame_spans
            .iter()
            .flat_map(|frame_span| {
                if let FrameSpanType::Boss(fight) = &frame_span.span_type {
                    &fight.phases[..]
                } else {
                    &[]
                }
            })
            .filter_map(|phase| {
                if let BossPhase::Spells { variant, spell_ids }
                | BossPhase::LastSpell { variant, spell_ids } = phase
                {
                    Some((variant, spell_ids.clone()))
                } else {
                    None
                }
            })
    }

    fn define_mapping_method<T, U, F>(
        &self,
        method_name: &'static str,
        capture_spell_ids: bool,
        is_const: bool,
        return_type: T,
        mut map_fn: F,
    ) -> TokenStream
    where
        T: ToTokens,
        U: ToTokens,
        F: FnMut(usize, &LocationVariant, Option<Ident>) -> U,
    {
        let method_name = Ident::new(method_name, self.stage_ident.span());
        let arms = self.iter_match_patterns(capture_spell_ids).enumerate().map(
            move |(idx, (variant, capture_name, pattern))| {
                let result = map_fn(idx, variant, capture_name);
                quote! { #pattern => #result }
            },
        );

        let const_kw = if is_const {
            Some(Token![const](self.stage_ident.span()))
        } else {
            None
        }
        .into_iter();

        quote! {
            pub #(#const_kw)* fn #method_name(self) -> #return_type {
                match self {
                    #(#arms),*
                }
            }
        }
    }

    fn iter_type(&self) -> Ident {
        format_ident!("{}Iter", &self.type_ident)
    }

    fn define_iter(&self) -> TokenStream {
        let self_type = &self.type_ident;
        let iter_type = self.iter_type();
        let spell_id_type = &self.spell_id_ident;
        let mut idx_arms = Vec::new();

        for variant in self.iter_variants() {
            let path = variant.full_path();
            if let Some(range) = variant.spell_range() {
                for spell_id in range.clone().map(|id| id as u16) {
                    idx_arms.push(
                        quote! { #path(crate::types::SpellCard::new(#spell_id_type::new(#spell_id).unwrap())) },
                    );
                }
            } else {
                idx_arms.push(path.into_token_stream())
            }
        }

        let n_arms = idx_arms.len() as u32;
        let idx_match_arms = idx_arms
            .into_iter()
            .enumerate()
            .map(|(idx, arm)| {
                let idx = idx as u32;
                quote! { #idx => #arm }
            })
            .chain(std::iter::once(quote! { #n_arms.. => unreachable!() }))
            .collect::<Vec<_>>();

        quote! {
            #[derive(Debug, Clone)]
            #[repr(transparent)]
            pub struct #iter_type(std::ops::Range<u32>);

            #[automatically_derived]
            impl #iter_type {
                pub const fn new() -> Self {
                    Self(0..#n_arms)
                }
            }

            #[automatically_derived]
            impl Iterator for #iter_type {
                type Item = #self_type;

                fn next(&mut self) -> Option<#self_type> {
                    self.0.next().map(|idx| match idx {
                        #(#idx_match_arms),*
                    })
                }

                #[inline]
                fn size_hint(&self) -> (usize, Option<usize>) {
                    self.0.size_hint()
                }
            }

            #[automatically_derived]
            impl DoubleEndedIterator for #iter_type {
                fn next_back(&mut self) -> Option<#self_type> {
                    self.0.next_back().map(|idx| match idx {
                        #(#idx_match_arms),*
                    })
                }
            }

            #[automatically_derived]
            impl ExactSizeIterator for #iter_type {
                #[inline]
                fn len(&self) -> usize {
                    self.0.len()
                }
            }

            #[automatically_derived]
            impl std::iter::FusedIterator for #iter_type { }

            #[automatically_derived]
            impl crate::types::AllIterable for #self_type {
                type IterAll = #iter_type;

                fn iter_all() -> #iter_type {
                    #iter_type::new()
                }
            }
        }
    }

    fn define_enum(&self) -> TokenStream {
        let type_name = &self.type_ident;
        let game = &self.game_type;
        let stage_name = self.stage_ident.to_string();
        let valid_indexes = range_to_tokens(&(0..=(self.iter_variants().count() as u64 - 1)));

        let variants = self.iter_variants().map(|variant| {
            let name = variant.variant_ident();
            let game = &self.game_type;
            if variant.needs_spell_id() {
                quote! { #name(crate::types::SpellCard<crate::#game>) }
            } else {
                quote! { #name }
            }
        });

        let name_method = self.define_mapping_method(
            "name",
            false,
            true,
            quote! { &'static str },
            |_, variant, _| variant.display_name().to_string(),
        );

        let index_method =
            self.define_mapping_method("index", false, true, quote! { u64 }, |idx, _, _| {
                idx as u64
            });

        let spell_method = self.define_mapping_method(
            "spell",
            true,
            true,
            quote! { Option<crate::types::SpellCard<#game>> },
            |_, _, capture| match capture {
                Some(ident) => quote! { Some(#ident) },
                None => quote! { None },
            },
        );

        let spell_to_location_map = self.iter_spell_variants().map(|(variant, spell_ids)| {
            let path = variant.full_path();
            let start = *spell_ids.start() as u16;
            let end = *spell_ids.end() as u16;
            quote! { #start..=#end => Some(#path(spell)), }
        });

        let mut rev_index_arms = Vec::new();
        for (idx, variant) in self.iter_variants().enumerate() {
            let idx = idx as u64;
            let path = variant.full_path();

            if let Some(range) = variant.spell_range().map(range_to_tokens) {
                let name = variant.display_name();

                rev_index_arms.push(quote! { (#idx, Some(spell_id @ #range)) => Ok(#path(crate::types::SpellCard::new(spell_id.try_into().unwrap()))) });
                rev_index_arms.push(quote! {
                    (#idx, Some(other_id)) => Err(crate::memory::InvalidLocationData::InvalidSpell {
                        stage: #stage_name,
                        loc_name: #name,
                        valid: #range
                    })
                });
                rev_index_arms.push(quote! {
                    (#idx, None) => Err(crate::memory::InvalidLocationData::MissingSpell {
                        stage: #stage_name,
                        loc_name: #name,
                        valid: #range
                    })
                })
            } else {
                rev_index_arms.push(quote! {
                    (#idx, _) => Ok(#path)
                })
            }
        }

        let display_map = self
            .iter_match_patterns(true)
            .map(|(variant, cap_ident, pattern)| {
                let name = variant.display_name();
                if variant.needs_spell_id() {
                    quote! {
                        #pattern => #cap_ident.name().fmt(f)
                    }
                } else {
                    quote! { #pattern => #name.fmt(f) }
                }
            });

        let mut is_boss_start_map = Vec::new();
        for frame_span in &self.frame_spans {
            match &frame_span.span_type {
                FrameSpanType::Boss(_) => {
                    is_boss_start_map.extend(frame_span.iter_variants().enumerate().map(
                        |(idx, variant)| {
                            let is_first = idx == 0;
                            let path = variant.full_path();
                            if variant.needs_spell_id() {
                                quote! { #path(_) => #is_first }
                            } else {
                                quote! { #path => #is_first }
                            }
                        },
                    ))
                }
                FrameSpanType::Single(variant) => {
                    let pattern = variant.match_pattern(None).1;
                    is_boss_start_map.push(quote! { #pattern => false })
                }
            }
        }

        let state_ident = format_ident!("state");
        let resolve_match_arms = self.resolve_match_arms(&state_ident);

        let last_variant_pattern = self
            .iter_variants()
            .last()
            .map(|variant| variant.match_pattern(None).1)
            .unwrap();

        let first_variant_path = self
            .iter_variants()
            .next()
            .map(LocationVariant::full_path)
            .unwrap();

        let resolve_bounds = if self.has_nonspells {
            quote! {
                T: crate::memory::traits::StageData<#game> + crate::memory::traits::ECLTimeline<#game>,
                T::BossState: crate::memory::traits::BossLifebars<#game>
            }
        } else {
            quote! {
                T: crate::memory::traits::StageData<#game>
            }
        };

        quote! {
            #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
            #[serde(tag = "type", content = "spell", rename_all = "snake_case")]
            pub enum #type_name {
                #(#variants),*
            }

            #[automatically_derived]
            impl #type_name {
                fn resolve<T>(#state_ident: &T) -> Option<Self>
                    where #resolve_bounds
                {
                    use crate::memory::traits::*;
                    match #state_ident.ecl_time() {
                        #resolve_match_arms
                    }
                }

                #name_method
                #index_method
                #spell_method

                pub(crate) fn from_index(index: u64, spell_id: Option<u32>) -> Result<Self, crate::memory::InvalidLocationData<#game>> {
                    match (index, spell_id) {
                        #(#rev_index_arms,)*
                        (index, _) => Err(crate::memory::InvalidLocationData::InvalidIndex {
                            stage: #stage_name,
                            index,
                            valid: #valid_indexes
                        })
                    }
                }

                pub const fn is_end(self) -> bool {
                    matches!(self, #last_variant_pattern)
                }

                pub const fn is_boss_start(self) -> bool {
                    match self {
                        #(#is_boss_start_map),*
                    }
                }

                pub const fn from_spell(spell: crate::types::SpellCard<#game>) -> Option<Self> {
                    use crate::types::SpellCard;
                    match spell.unwrap().unwrap() {
                        #(#spell_to_location_map)*
                        _ => None
                    }
                }
            }

            #[automatically_derived]
            impl std::fmt::Display for #type_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #(#display_map),*
                    }
                }
            }

            #[automatically_derived]
            impl Default for #type_name {
                fn default() -> Self {
                    #first_variant_path
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct GameLocations {
    type_ident: Ident,
    game_type: Ident,
    stage_type: Ident,
    stages: Vec<StageLocations>,
    exclude_stages: Vec<Ident>,
    resolve_visibility: Visibility,
}

impl GameLocations {
    pub fn from_ast(def: &ast::LocationsDef) -> Result<Self, syn::Error> {
        let type_ident = def.type_id.clone();
        let stage_type = def.stage_type.clone();
        let exclude_stages = def.exclude_stages.clone();
        let resolve_visibility = def.resolve_visibility.clone();

        def.stages
            .iter()
            .map(|stage_def| {
                StageLocations::from_ast(
                    def.game_type.clone(),
                    def.spell_id_type.clone(),
                    stage_def,
                )
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|stages| Self {
                type_ident,
                game_type: def.game_type.clone(),
                stage_type,
                stages,
                exclude_stages,
                resolve_visibility,
            })
    }

    fn has_nonspells(&self) -> bool {
        self.stages.iter().any(|stage| stage.has_nonspells)
    }

    fn iter_type(&self) -> Ident {
        format_ident!("Iter{}", &self.type_ident)
    }

    fn define_iter_all_method(&self) -> TokenStream {
        let self_type = &self.type_ident;
        let iter_type = self.iter_type();
        let inner_types = self.stages.iter().map(StageLocations::iter_type);

        let iter_init_exprs = self.stages.iter().map(|stage| {
            let type_ident = &stage.type_ident;
            quote! { #type_ident::iter_all() }
        });

        let iter_size_exprs = self.stages.iter().enumerate().map(|(idx, _)| {
            let idx = Index::from(idx);
            quote! { self.#idx.len() }
        });

        let iter_len_expr = quote! {
            #(#iter_size_exprs)+*
        };

        let fwd_expr = {
            let mut exprs = self.stages.iter().enumerate().map(|(idx, stage)| {
                let idx = Index::from(idx);
                let stage_id = &stage.stage_ident;
                quote! { self.#idx.next().map(#self_type::#stage_id) }
            });

            let first = exprs.next();
            first
                .map(|first| {
                    quote! {
                        #first
                        #(.or_else(|| #exprs))*
                    }
                })
                .unwrap_or_else(TokenStream::new)
        };

        let rev_expr = {
            let mut exprs = self.stages.iter().enumerate().rev().map(|(idx, stage)| {
                let idx = Index::from(idx);
                let stage_id = &stage.stage_ident;
                quote! { self.#idx.next_back().map(#self_type::#stage_id) }
            });

            let last = exprs.next();
            last.map(|last| {
                quote! {
                    #last
                    #(.or_else(|| #exprs))*
                }
            })
            .unwrap_or_else(TokenStream::new)
        };

        quote! {
            #[derive(Debug, Clone)]
            pub struct #iter_type(#(#inner_types),*);

            #[automatically_derived]
            impl Iterator for #iter_type {
                type Item = #self_type;

                fn next(&mut self) -> Option<#self_type> {
                    #fwd_expr
                }

                #[inline]
                fn size_hint(&self) -> (usize, Option<usize>) {
                    (self.len(), Some(self.len()))
                }
            }

            #[automatically_derived]
            impl DoubleEndedIterator for #iter_type {
                fn next_back(&mut self) -> Option<#self_type> {
                    #rev_expr
                }
            }

            #[automatically_derived]
            impl ExactSizeIterator for #iter_type {
                #[inline]
                fn len(&self) -> usize {
                    #iter_len_expr
                }
            }

            #[automatically_derived]
            impl std::iter::FusedIterator for #iter_type { }

            #[automatically_derived]
            impl crate::types::AllIterable for #self_type {
                type IterAll = #iter_type;

                fn iter_all() -> #iter_type {
                    #iter_type(#(#iter_init_exprs),*)
                }
            }
        }
    }

    fn impl_resolve(&self) -> TokenStream {
        let type_name = &self.type_ident;
        let game = &self.game_type;
        let stage_type = &self.stage_type;
        let resolve_vis = &self.resolve_visibility;

        let state_ident = format_ident!("stage_state");
        let resolve_match_arms = self.stages.iter().map(|stage| {
            let stage_type_ident = &stage.type_ident;
            let stage_id = &stage.stage_ident;

            quote! {
                #stage_type::#stage_id => #stage_type_ident::resolve(#state_ident).map(Self::#stage_id)
            }
        }).chain(self.exclude_stages.iter().map(|stage_id| {
            quote! { #stage_type::#stage_id => None }
        }));

        let resolve_bounds = if self.has_nonspells() {
            quote! {
                T: crate::memory::traits::RunData<#game>,
                T::StageState: crate::memory::traits::ECLTimeline<#game>,
                <T::StageState as crate::memory::traits::StageData<#game>>::BossState: crate::memory::traits::BossLifebars<#game>
            }
        } else {
            quote! {
                T: crate::memory::traits::RunData<#game>
            }
        };

        let location_resolve_method = if matches!(&self.resolve_visibility, Visibility::Public(_)) {
            quote! {
                #[automatically_derived]
                impl crate::memory::Location<#game> {
                    pub fn resolve<T>(state: &T) -> Option<Self>
                        where #resolve_bounds
                    {
                        #type_name::resolve(state).map(Self::new)
                    }
                }
            }
        } else {
            TokenStream::new()
        };

        quote! {
            #[automatically_derived]
            impl #type_name {
                #resolve_vis fn resolve<T>(state: &T) -> Option<Self>
                    where #resolve_bounds
                {
                    use crate::memory::traits::*;
                    let #state_ident = state.stage();
                    match #state_ident.stage_id().unwrap() {
                        #(#resolve_match_arms),*
                    }
                }
            }

            #location_resolve_method
        }
    }

    pub fn define_main_enum(&self) -> TokenStream {
        let type_name = &self.type_ident;
        let game = &self.game_type;
        let stage_type = &self.stage_type;

        let variants = self.stages.iter().map(|stage| {
            let stage_type_ident = &stage.type_ident;
            let stage_id = &stage.stage_ident;

            quote! {
                #stage_id(#stage_type_ident)
            }
        });

        let stage_match_arms = self
            .stages
            .iter()
            .map(|stage| {
                let stage_id = &stage.stage_ident;
                quote! {
                    Self::#stage_id(_) => crate::types::Stage::new(#stage_type::#stage_id)
                }
            })
            .collect::<Vec<_>>();

        let display_map = self.stages.iter().map(|stage| {
            let stage_id = &stage.stage_ident;

            quote! {
                Self::#stage_id(section) => if let Some(spell) = section.spell() {
                    spell.name().fmt(f)
                } else {
                    write!(f, "{} {}", &#stage_type::#stage_id, section)
                }
            }
        });

        let name_match_arms = self
            .stages
            .iter()
            .map(|stage| {
                let stage_id = &stage.stage_ident;

                quote! {
                    Self::#stage_id(section) => section.name()
                }
            })
            .collect::<Vec<_>>();

        let spell_match_arms = self
            .stages
            .iter()
            .map(|stage| {
                let stage_id = &stage.stage_ident;

                quote! {
                    Self::#stage_id(section) => section.spell()
                }
            })
            .collect::<Vec<_>>();

        let is_end_match_arms = self
            .stages
            .iter()
            .map(|stage| {
                let stage_id = &stage.stage_ident;

                quote! {
                    Self::#stage_id(section) => section.is_end()
                }
            })
            .collect::<Vec<_>>();

        let is_boss_start_match_arms = self
            .stages
            .iter()
            .map(|stage| {
                let stage_id = &stage.stage_ident;

                quote! {
                    Self::#stage_id(section) => section.is_boss_start()
                }
            })
            .collect::<Vec<_>>();

        let from_spell_match_arms = self
            .stages
            .iter()
            .flat_map(|stage| {
                stage.iter_spell_variants().map(|(variant, spell_ids)| {
                    let path = variant.full_path();
                    let start = *spell_ids.start() as u16;
                    let end = *spell_ids.end() as u16;
                    let stage_id = &stage.stage_ident;
                    quote! { #start..=#end => Some(Self::#stage_id(#path(spell))), }
                })
            })
            .collect::<Vec<_>>();

        let mut cur_idx = 0;
        let index_match_arms = self
            .stages
            .iter()
            .map(move |stage| {
                let stage_id = &stage.stage_ident;
                let offset = cur_idx as u64;
                cur_idx += stage.iter_variants().count();

                quote! {
                    Self::#stage_id(section) => section.index() + #offset
                }
            })
            .collect::<Vec<_>>();

        let from_index_match_arms = self.stages.iter().map(move |stage| {
            let stage_id = &stage.stage_ident;
            let stage_type_ident = &stage.type_ident;

            quote! {
                #stage_type::#stage_id => #stage_type_ident::from_index(index, spell_id).map(Self::#stage_id)
            }
        }).chain(self.exclude_stages.iter().map(|stage_id| {
            quote! { #stage_type::#stage_id => Err(crate::memory::InvalidLocationData::NoStageData { stage: #stage_type::#stage_id.name() }) }
        })).collect::<Vec<_>>();

        let to_index_match_arms = self.stages.iter().map(move |stage| {
            let stage_id = &stage.stage_ident;

            quote! {
                #type_name::#stage_id(section) => crate::memory::AnyLocation::new(<#game as crate::types::Game>::GAME_ID, #stage_type::#stage_id.into(), section.index(), section.spell().as_ref().map(crate::types::SpellCard::id))
            }
        }).collect::<Vec<_>>();

        let stage_start_locations = self.stages.iter().map(move |stage| {
            let stage_id = &stage.stage_ident;
            let first_variant = stage.iter_variants().next().unwrap().full_path();

            quote! {
                #stage_type::#stage_id => #type_name::#stage_id(Default::default())
            }
        });

        let default_expr = self.stages.iter().next().map(move |stage| {
            let stage_id = &stage.stage_ident;

            quote! {
                #type_name::#stage_id(Default::default())
            }
        });

        let resolve_impl = self.impl_resolve();

        quote! {
            #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
            #[serde(tag = "stage", content = "section", rename_all = "snake_case")]
            pub enum #type_name {
                #(#variants),*
            }

            #resolve_impl

            #[automatically_derived]
            impl #type_name {
                pub const fn name(self) -> &'static str {
                    match self {
                        #(#name_match_arms),*
                    }
                }

                pub const fn index(self) -> u64 {
                    match self {
                        #(#index_match_arms),*
                    }
                }

                pub const fn stage(self) -> crate::types::Stage<#game> {
                    match self {
                        #(#stage_match_arms),*
                    }
                }

                pub const fn spell(self) -> Option<crate::types::SpellCard<#game>> {
                    match self {
                        #(#spell_match_arms),*
                    }
                }

                pub const fn is_end(self) -> bool {
                    match self {
                        #(#is_end_match_arms),*
                    }
                }

                pub const fn is_boss_start(self) -> bool {
                    match self {
                        #(#is_boss_start_match_arms),*
                    }
                }

                pub const fn from_spell(spell: crate::types::SpellCard<#game>) -> Option<Self> {
                    match spell.unwrap().unwrap() {
                        #(#from_spell_match_arms)*
                        _ => None
                    }
                }
            }

            #[automatically_derived]
            impl TryFrom<crate::memory::AnyLocation> for #type_name {
                type Error = crate::memory::InvalidLocationData<#game>;

                fn try_from(value: crate::memory::AnyLocation) -> Result<Self, Self::Error> {
                    let stage = <#stage_type as crate::types::GameValue>::from_raw(value.stage(), value.game()).map_err(crate::memory::InvalidLocationData::InvalidStage)?;
                    let index = value.index();
                    let spell_id = value.spell();

                    match stage {
                        #(#from_index_match_arms),*
                    }
                }
            }

            #[automatically_derived]
            impl From<#type_name> for crate::memory::AnyLocation {
                fn from(value: #type_name) -> Self {
                    match value {
                        #(#to_index_match_arms),*
                    }
                }
            }

            #[automatically_derived]
            impl crate::memory::GameLocation<#game> for #type_name {
                fn name(&self) -> &'static str {
                    match self {
                        #(#name_match_arms),*
                    }
                }

                fn index(&self) -> u64 {
                    match self {
                        #(#index_match_arms),*
                    }
                }

                fn stage(&self) -> crate::types::Stage<#game> {
                    match self {
                        #(#stage_match_arms),*
                    }
                }

                fn spell(&self) -> Option<crate::types::SpellCard<#game>> {
                    match self {
                        #(#spell_match_arms),*
                    }
                }

                fn is_end(&self) -> bool {
                    match self {
                        #(#is_end_match_arms),*
                    }
                }

                fn is_boss_start(&self) -> bool {
                    match self {
                        #(#is_boss_start_match_arms),*
                    }
                }

                fn from_spell(spell: crate::types::SpellCard<#game>) -> Option<Self> {
                    match spell.unwrap().unwrap() {
                        #(#from_spell_match_arms)*
                        _ => None
                    }
                }
            }

            #[automatically_derived]
            impl crate::memory::HasLocations for #game {
                type Location = #type_name;

                fn stage_start_location(stage: #stage_type) -> #type_name {
                    match stage {
                        #(#stage_start_locations,)*
                        _ => unimplemented!("no locations defined for {stage}")
                    }
                }
            }

            #[automatically_derived]
            impl std::borrow::Borrow<#type_name> for crate::memory::Location<#game> {
                fn borrow(&self) -> &#type_name {
                    self.as_ref()
                }
            }

            #[automatically_derived]
            impl From<#type_name> for crate::memory::Location<#game> {
                fn from(value: #type_name) -> Self {
                    Self::new(value)
                }
            }

            #[automatically_derived]
            impl From<crate::memory::Location<#game>> for #type_name {
                fn from(value: crate::memory::Location<#game>) -> Self {
                    value.unwrap()
                }
            }

            #[automatically_derived]
            impl std::fmt::Display for #type_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #(#display_map),*
                    }
                }
            }

            #[automatically_derived]
            impl Default for #type_name {
                fn default() -> #type_name {
                    #default_expr
                }
            }
        }
    }

    pub fn define_sub_enums(&self) -> TokenStream {
        self.stages
            .iter()
            .map(|stage| stage.define_enum())
            .collect()
    }

    pub fn define_iter_module(&self) -> TokenStream {
        let contents = self
            .stages
            .iter()
            .map(StageLocations::define_iter)
            .chain(std::iter::once(self.define_iter_all_method()));
        quote! {
            pub mod iter {
                use super::*;
                #(#contents)*
            }
        }
    }

    pub fn to_definitions(&self) -> TokenStream {
        let mut ret = self.define_sub_enums();
        ret.extend(self.define_main_enum());
        ret.extend(self.define_iter_module());
        ret
    }
}
