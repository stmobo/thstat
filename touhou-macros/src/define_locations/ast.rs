use std::ops::RangeInclusive;

use syn::parse::{Lookahead1, Parse};
use syn::punctuated::Punctuated;
use syn::{braced, bracketed, parenthesized, token, Attribute, Ident, LitInt, LitStr, Token};

use crate::util;

pub mod kw {
    syn::custom_keyword!(Section);
    syn::custom_keyword!(Midboss);
    syn::custom_keyword!(Boss);
    syn::custom_keyword!(Nonspell);
    syn::custom_keyword!(Spells);
    syn::custom_keyword!(LastSpell);
}

#[derive(Debug)]
pub struct SpellRange {
    pub start: LitInt,
    _sep: Token![..=],
    pub end: LitInt,
}

impl SpellRange {
    pub fn parse_range(&self) -> Result<RangeInclusive<u32>, syn::Error> {
        let start: u32 = self.start.base10_parse()?;
        let end: u32 = self.end.base10_parse()?;

        if start >= end {
            Err(syn::Error::new(self.start.span(), "invalid range"))
        } else {
            Ok(start..=end)
        }
    }
}

impl Parse for SpellRange {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            start: input.parse()?,
            _sep: input.parse()?,
            end: input.parse()?,
        })
    }
}

#[derive(Debug)]
pub enum BossPhaseDef {
    Nonspell {
        _kw: kw::Nonspell,
    },
    Spells {
        key: kw::Spells,
        _paren: token::Paren,
        range: SpellRange,
    },
    LastSpell {
        key: kw::LastSpell,
        _paren: token::Paren,
        ranges: Punctuated<SpellRange, Token![,]>,
    },
}

impl BossPhaseDef {
    fn peek(lookahead: &Lookahead1) -> bool {
        lookahead.peek(kw::Nonspell) || lookahead.peek(kw::Spells) || lookahead.peek(kw::LastSpell)
    }
}

impl Parse for BossPhaseDef {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::Nonspell) {
            input.parse().map(|_kw| Self::Nonspell { _kw })
        } else if lookahead.peek(kw::Spells) {
            let content;

            Ok(Self::Spells {
                key: input.parse()?,
                _paren: parenthesized!(content in input),
                range: content.parse()?,
            })
        } else if lookahead.peek(kw::LastSpell) {
            let content;

            Ok(Self::LastSpell {
                key: input.parse()?,
                _paren: parenthesized!(content in input),
                ranges: content.parse_terminated(SpellRange::parse, Token![,])?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug)]
pub struct BossDef {
    _bracket: Option<token::Bracket>,
    pub phases: Punctuated<BossPhaseDef, Token![,]>,
}

impl Parse for BossDef {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(token::Bracket) {
            let content;
            Ok(Self {
                _bracket: Some(bracketed!(content in input)),
                phases: content.parse_terminated(BossPhaseDef::parse, Token![,])?,
            })
        } else if BossPhaseDef::peek(&lookahead) {
            let mut phases = Punctuated::new();
            phases.push_value(input.parse()?);
            Ok(Self {
                _bracket: None,
                phases,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug)]
pub enum SectionDef {
    Basic {
        section_kw: kw::Section,
        name: Option<(token::Paren, LitStr)>,
    },
    Midboss {
        section_kw: kw::Midboss,
        def: BossDef,
    },
    Boss {
        section_kw: kw::Boss,
        def: BossDef,
    },
}

impl Parse for SectionDef {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::Section) {
            let section_kw = input.parse()?;
            let lookahead = input.lookahead1();

            let name = if lookahead.peek(token::Paren) {
                let content;
                Some((parenthesized!(content in input), content.parse()?))
            } else {
                None
            };

            Ok(Self::Basic { section_kw, name })
        } else if lookahead.peek(kw::Midboss) {
            Ok(Self::Midboss {
                section_kw: input.parse()?,
                def: input.parse()?,
            })
        } else if lookahead.peek(kw::Boss) {
            Ok(Self::Boss {
                section_kw: input.parse()?,
                def: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug)]
pub struct SectionEntry {
    pub frame_number: LitInt,
    pub _arrow: Token![=>],
    pub def: SectionDef,
}

impl Parse for SectionEntry {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            frame_number: input.parse()?,
            _arrow: input.parse()?,
            def: input.parse()?,
        })
    }
}

#[derive(Debug)]
pub struct StageDef {
    pub override_type_name: Option<Ident>,
    pub stage_id: Ident,
    pub _colon: Token![:],
    pub _brace: token::Brace,
    pub sections: Punctuated<SectionEntry, Token![,]>,
}

impl Parse for StageDef {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;

        let attrs = input.call(Attribute::parse_outer)?;
        let override_type_name = util::parse_attribute_str("type_name", &attrs)?;

        Ok(Self {
            override_type_name,
            stage_id: input.parse()?,
            _colon: input.parse()?,
            _brace: braced!(content in input),
            sections: content.parse_terminated(SectionEntry::parse, Token![,])?,
        })
    }
}

struct ExcludeStages(Punctuated<Ident, Token![,]>);

impl Parse for ExcludeStages {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse_terminated(Ident::parse, Token![,]).map(Self)
    }
}

#[derive(Debug)]
pub struct LocationsDef {
    pub type_id: Ident,
    pub game_type: Ident,
    pub stage_type: Ident,
    pub exclude_stages: Vec<Ident>,
    _brace: token::Brace,
    pub stages: Punctuated<StageDef, Token![,]>,
}

impl Parse for LocationsDef {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;

        let attrs = input.call(Attribute::parse_outer)?;
        let stage_type = util::parse_attribute_str("stage_type", &attrs)?
            .ok_or_else(|| input.error("missing attribute 'stage_type'"))?;

        let game_type = util::parse_attribute_str("game", &attrs)?
            .ok_or_else(|| input.error("missing attribute 'game'"))?;

        let exclude_stages: ExcludeStages = util::parse_attribute_str("exclude_stages", &attrs)?
            .unwrap_or_else(|| ExcludeStages(Punctuated::new()));

        Ok(Self {
            type_id: input.parse()?,
            stage_type,
            game_type,
            exclude_stages: exclude_stages.0.into_iter().collect(),
            _brace: braced!(content in input),
            stages: content.parse_terminated(StageDef::parse, Token![,])?,
        })
    }
}
