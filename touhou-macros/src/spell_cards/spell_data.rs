use std::fmt::Display;

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, TokenStreamExt};
use syn::parse::{Lookahead1, Parse, ParseStream};
use syn::{Ident, Result, Token};

macro_rules! parse_from_keywords {
    ([ $first:ident $(, $keyword:ident)*$(,)? ] => $parsed:ty) => {
        #[automatically_derived]
        impl Parse for $parsed {
            fn parse(input: ParseStream) -> Result<Self> {
                let lookahead = input.lookahead1();

                if lookahead.peek(kw::$first) {
                    input.parse().map(Self::$first)
                } $(
                    else if lookahead.peek(kw::$keyword) {
                        input.parse().map(Self::$keyword)
                    }
                )* else {
                    Err(lookahead.error())
                }
            }
        }

        #[automatically_derived]
        impl $parsed {
            pub fn span(&self) -> Span {
                match self {
                    Self::$first(inner) => inner.span,
                    $(
                        Self::$keyword(inner) => inner.span
                    ),*
                }
            }

            pub fn peek(lookahead: &Lookahead1) -> bool {
                lookahead.peek(kw::$first) $(|| lookahead.peek(kw::$keyword))*
            }
        }
    };
}

pub(crate) use parse_from_keywords;

pub mod kw {
    syn::custom_keyword!(S1);
    syn::custom_keyword!(S2);
    syn::custom_keyword!(S3);
    syn::custom_keyword!(S4);
    syn::custom_keyword!(S4A);
    syn::custom_keyword!(S4B);
    syn::custom_keyword!(S5);
    syn::custom_keyword!(S6);
    syn::custom_keyword!(S6A);
    syn::custom_keyword!(S6B);
    syn::custom_keyword!(Extra);
    syn::custom_keyword!(Phantasm);
    syn::custom_keyword!(Easy);
    syn::custom_keyword!(Normal);
    syn::custom_keyword!(Hard);
    syn::custom_keyword!(Lunatic);
    syn::custom_keyword!(LastWord);
    syn::custom_keyword!(Midboss);
    syn::custom_keyword!(Boss);
    syn::custom_keyword!(LastSpell);
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum StageSpellType {
    Midboss(kw::Midboss),
    Boss(kw::Boss),
    LastSpell(kw::LastSpell),
}

parse_from_keywords!(
    [Midboss, Boss, LastSpell] => StageSpellType
);

impl From<StageSpellType> for Ident {
    fn from(value: StageSpellType) -> Self {
        match value {
            StageSpellType::Midboss(t) => Ident::new("Midboss", t.span),
            StageSpellType::Boss(t) => Ident::new("Boss", t.span),
            StageSpellType::LastSpell(t) => Ident::new("LastSpell", t.span),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum SpellType {
    StageSpell(StageSpellType),
    LastWord(kw::LastWord),
}

impl SpellType {
    pub fn span(&self) -> Span {
        match self {
            Self::StageSpell(inner) => inner.span(),
            Self::LastWord(inner) => inner.span,
        }
    }
}

impl From<StageSpellType> for SpellType {
    fn from(value: StageSpellType) -> Self {
        Self::StageSpell(value)
    }
}

impl From<kw::LastWord> for SpellType {
    fn from(value: kw::LastWord) -> Self {
        Self::LastWord(value)
    }
}

impl From<SpellType> for Ident {
    fn from(value: SpellType) -> Self {
        match value {
            SpellType::StageSpell(spell_type) => spell_type.into(),
            SpellType::LastWord(t) => Ident::new("LastWord", t.span),
        }
    }
}

impl ToTokens for SpellType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let span = self.span();
        let ident: Ident = (*self).into();

        tokens.append(Ident::new("crate", span));
        Token![::](span).to_tokens(tokens);
        tokens.append(Ident::new("types", span));
        Token![::](span).to_tokens(tokens);
        tokens.append(Ident::new("SpellType", span));
        Token![::](span).to_tokens(tokens);
        ident.to_tokens(tokens);
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum MainStage {
    S1(kw::S1),
    S2(kw::S2),
    S3(kw::S3),
    S4(kw::S4),
    S4A(kw::S4A),
    S4B(kw::S4B),
    S5(kw::S5),
    S6(kw::S6),
    S6A(kw::S6A),
    S6B(kw::S6B),
}

parse_from_keywords!(
    [S1, S2, S3, S4, S4A, S4B, S5, S6, S6A, S6B] => MainStage
);

impl MainStage {
    pub fn name(&self) -> &'static str {
        match self {
            Self::S1(_) => "1",
            Self::S2(_) => "2",
            Self::S3(_) => "3",
            Self::S4(_) => "4",
            Self::S4A(_) => "4A",
            Self::S4B(_) => "4B",
            Self::S5(_) => "5",
            Self::S6(_) => "6",
            Self::S6A(_) => "6A",
            Self::S6B(_) => "6B",
        }
    }
}

impl From<MainStage> for Ident {
    fn from(value: MainStage) -> Self {
        match value {
            MainStage::S1(t) => Ident::new("One", t.span),
            MainStage::S2(t) => Ident::new("Two", t.span),
            MainStage::S3(t) => Ident::new("Three", t.span),
            MainStage::S4(t) => Ident::new("Four", t.span),
            MainStage::S4A(t) => Ident::new("FourA", t.span),
            MainStage::S4B(t) => Ident::new("FourB", t.span),
            MainStage::S5(t) => Ident::new("Five", t.span),
            MainStage::S6(t) => Ident::new("Six", t.span),
            MainStage::S6A(t) => Ident::new("FinalA", t.span),
            MainStage::S6B(t) => Ident::new("FinalB", t.span),
        }
    }
}

impl ToTokens for MainStage {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident: Ident = (*self).into();
        ident.to_tokens(tokens);
    }
}

impl Display for MainStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum ExtraStage {
    Extra(kw::Extra),
    Phantasm(kw::Phantasm),
}

parse_from_keywords!(
    [Extra, Phantasm] => ExtraStage
);

impl ExtraStage {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Extra(_) => "Extra",
            Self::Phantasm(_) => "Phantasm",
        }
    }
}

impl From<ExtraStage> for Ident {
    fn from(value: ExtraStage) -> Self {
        match value {
            ExtraStage::Extra(t) => Ident::new("Extra", t.span),
            ExtraStage::Phantasm(t) => Ident::new("Phantasm", t.span),
        }
    }
}

impl ToTokens for ExtraStage {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident: Ident = (*self).into();
        ident.to_tokens(tokens);
    }
}

impl Display for ExtraStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Stage {
    Main(MainStage),
    Extra(ExtraStage),
    LastWord(kw::LastWord),
}

impl Stage {
    pub fn span(&self) -> Span {
        match self {
            Self::Main(stage) => stage.span(),
            Self::Extra(stage) => stage.span(),
            Self::LastWord(inner) => inner.span,
        }
    }
}

impl From<Stage> for Ident {
    fn from(value: Stage) -> Self {
        match value {
            Stage::Main(stage) => stage.into(),
            Stage::Extra(stage) => stage.into(),
            Stage::LastWord(inner) => Ident::new("LastWord", inner.span),
        }
    }
}

impl Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Main(stage) => write!(f, "Stage {}", stage),
            Self::Extra(stage) => write!(f, "{} Stage", stage),
            Self::LastWord(_) => f.write_str("Last Word"),
        }
    }
}

impl Parse for Stage {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if MainStage::peek(&lookahead) {
            input.parse().map(Self::Main)
        } else if ExtraStage::peek(&lookahead) {
            input.parse().map(Self::Extra)
        } else if lookahead.peek(kw::LastWord) {
            input.parse().map(Self::LastWord)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for Stage {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let span = self.span();
        let ident: Ident = (*self).into();

        tokens.append(Ident::new("Stage", span));
        Token![::](span).to_tokens(tokens);
        ident.to_tokens(tokens);
    }
}

impl From<MainStage> for Stage {
    fn from(value: MainStage) -> Self {
        Self::Main(value)
    }
}

impl From<ExtraStage> for Stage {
    fn from(value: ExtraStage) -> Self {
        Self::Extra(value)
    }
}

impl From<kw::LastWord> for Stage {
    fn from(value: kw::LastWord) -> Self {
        Self::LastWord(value)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum MainDifficulty {
    Easy(kw::Easy),
    Normal(kw::Normal),
    Hard(kw::Hard),
    Lunatic(kw::Lunatic),
}

impl MainDifficulty {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Easy(_) => "Easy",
            Self::Normal(_) => "Normal",
            Self::Hard(_) => "Hard",
            Self::Lunatic(_) => "Lunatic",
        }
    }
}

parse_from_keywords!(
    [Easy, Normal, Hard, Lunatic] => MainDifficulty
);

impl From<MainDifficulty> for Ident {
    fn from(value: MainDifficulty) -> Self {
        match value {
            MainDifficulty::Easy(t) => Ident::new("Easy", t.span),
            MainDifficulty::Normal(t) => Ident::new("Normal", t.span),
            MainDifficulty::Hard(t) => Ident::new("Hard", t.span),
            MainDifficulty::Lunatic(t) => Ident::new("Lunatic", t.span),
        }
    }
}

impl ToTokens for MainDifficulty {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident: Ident = (*self).into();
        ident.to_tokens(tokens);
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Difficulty {
    Main(MainDifficulty),
    Extra(ExtraStage),
    LastWord(kw::LastWord),
}

impl Difficulty {
    pub fn span(&self) -> Span {
        match self {
            Self::Main(inner) => inner.span(),
            Self::Extra(inner) => inner.span(),
            Self::LastWord(inner) => inner.span,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Main(inner) => inner.name(),
            Self::Extra(inner) => inner.name(),
            Self::LastWord(_) => "Last Word",
        }
    }
}

impl From<Difficulty> for Ident {
    fn from(value: Difficulty) -> Self {
        match value {
            Difficulty::Main(inner) => inner.into(),
            Difficulty::Extra(inner) => inner.into(),
            Difficulty::LastWord(t) => Ident::new("LastWord", t.span),
        }
    }
}

impl ToTokens for Difficulty {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let span = self.span();
        let ident: Ident = (*self).into();

        tokens.append(Ident::new("Difficulty", span));
        Token![::](span).to_tokens(tokens);
        ident.to_tokens(tokens);
    }
}

impl From<MainDifficulty> for Difficulty {
    fn from(value: MainDifficulty) -> Self {
        Self::Main(value)
    }
}

impl From<ExtraStage> for Difficulty {
    fn from(value: ExtraStage) -> Self {
        Self::Extra(value)
    }
}

impl From<kw::LastWord> for Difficulty {
    fn from(value: kw::LastWord) -> Self {
        Self::LastWord(value)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum SpellLocation {
    Main {
        stage: MainStage,
        difficulty: MainDifficulty,
        spell_type: StageSpellType,
    },
    Extra {
        stage: ExtraStage,
        spell_type: StageSpellType,
    },
    LastWord(kw::LastWord),
}

impl SpellLocation {
    pub fn difficulty_span(&self) -> Span {
        match self {
            Self::Main { difficulty, .. } => difficulty.span(),
            Self::Extra { stage, .. } => stage.span(),
            Self::LastWord(lw) => lw.span,
        }
    }
}

impl From<kw::LastWord> for SpellLocation {
    fn from(value: kw::LastWord) -> Self {
        Self::LastWord(value)
    }
}

impl From<SpellLocation> for Stage {
    fn from(value: SpellLocation) -> Self {
        match value {
            SpellLocation::Main { stage, .. } => stage.into(),
            SpellLocation::Extra { stage, .. } => stage.into(),
            SpellLocation::LastWord(inner) => inner.into(),
        }
    }
}

impl From<SpellLocation> for Difficulty {
    fn from(value: SpellLocation) -> Self {
        match value {
            SpellLocation::Main { difficulty, .. } => difficulty.into(),
            SpellLocation::Extra { stage, .. } => stage.into(),
            SpellLocation::LastWord(inner) => inner.into(),
        }
    }
}

impl From<SpellLocation> for SpellType {
    fn from(value: SpellLocation) -> Self {
        match value {
            SpellLocation::Main { spell_type, .. } | SpellLocation::Extra { spell_type, .. } => {
                spell_type.into()
            }
            SpellLocation::LastWord(inner) => inner.into(),
        }
    }
}
