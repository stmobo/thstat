use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitInt, LitStr, Result};

macro_rules! parse_from_keywords {
    ([ $first:ident $(, $keyword:ident)*$(,)? ] => $parsed:ty) => {
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
    };
}

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
}

#[derive(Clone)]
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

impl MainStage {
    pub fn into_spell_location(
        self,
        difficulty: MainDifficulty,
        midboss: Option<kw::Midboss>,
    ) -> SpellLocation {
        match self {
            MainStage::S1(stage_token) => SpellLocation::S1 {
                stage_token,
                difficulty,
                midboss,
            },
            MainStage::S2(stage_token) => SpellLocation::S2 {
                stage_token,
                difficulty,
                midboss,
            },
            MainStage::S3(stage_token) => SpellLocation::S3 {
                stage_token,
                difficulty,
                midboss,
            },
            MainStage::S4(stage_token) => SpellLocation::S4 {
                stage_token,
                difficulty,
                midboss,
            },
            MainStage::S4A(stage_token) => SpellLocation::S4A {
                stage_token,
                difficulty,
                midboss,
            },
            MainStage::S4B(stage_token) => SpellLocation::S4B {
                stage_token,
                difficulty,
                midboss,
            },
            MainStage::S5(stage_token) => SpellLocation::S5 {
                stage_token,
                difficulty,
                midboss,
            },
            MainStage::S6(stage_token) => SpellLocation::S6 {
                stage_token,
                difficulty,
                midboss,
            },
            MainStage::S6A(stage_token) => SpellLocation::S6A {
                stage_token,
                difficulty,
                midboss,
            },
            MainStage::S6B(stage_token) => SpellLocation::S6B {
                stage_token,
                difficulty,
                midboss,
            },
        }
    }
}

parse_from_keywords!(
    [S1, S2, S3, S4, S4A, S4B, S5, S6, S6A, S6B] => MainStage
);

#[derive(Clone)]
pub enum Stage {
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
    Extra(kw::Extra),
    Phantasm(kw::Phantasm),
    LastWord(kw::LastWord),
}

parse_from_keywords!(
    [S1, S2, S3, S4, S4A, S4B, S5, S6, S6A, S6B, Extra, Phantasm, LastWord] => Stage
);

impl Stage {
    pub fn into_main_stage(self) -> std::result::Result<MainStage, Self> {
        match self {
            Self::S1(token) => Ok(MainStage::S1(token)),
            Self::S2(token) => Ok(MainStage::S2(token)),
            Self::S3(token) => Ok(MainStage::S3(token)),
            Self::S4(token) => Ok(MainStage::S4(token)),
            Self::S4A(token) => Ok(MainStage::S4A(token)),
            Self::S4B(token) => Ok(MainStage::S4B(token)),
            Self::S5(token) => Ok(MainStage::S5(token)),
            Self::S6(token) => Ok(MainStage::S6(token)),
            Self::S6A(token) => Ok(MainStage::S6A(token)),
            Self::S6B(token) => Ok(MainStage::S6B(token)),
            other => Err(other),
        }
    }

    pub fn can_have_midboss(&self) -> bool {
        !matches!(self, Self::LastWord(_))
    }
}

#[derive(Clone)]
pub enum MainDifficulty {
    Easy(kw::Easy),
    Normal(kw::Normal),
    Hard(kw::Hard),
    Lunatic(kw::Lunatic),
}

impl MainDifficulty {
    pub fn to_ident_tokens(&self) -> TokenStream {
        match self {
            Self::Easy(_) => quote!(Difficulty::Easy),
            Self::Normal(_) => quote!(Difficulty::Normal),
            Self::Hard(_) => quote!(Difficulty::Hard),
            Self::Lunatic(_) => quote!(Difficulty::Lunatic),
        }
    }

    pub fn difficulty_name(&self) -> &'static str {
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

#[allow(dead_code)]
#[derive(Clone)]
pub enum SpellLocation {
    S1 {
        stage_token: kw::S1,
        difficulty: MainDifficulty,
        midboss: Option<kw::Midboss>,
    },
    S2 {
        stage_token: kw::S2,
        difficulty: MainDifficulty,
        midboss: Option<kw::Midboss>,
    },
    S3 {
        stage_token: kw::S3,
        difficulty: MainDifficulty,
        midboss: Option<kw::Midboss>,
    },
    S4 {
        stage_token: kw::S4,
        difficulty: MainDifficulty,
        midboss: Option<kw::Midboss>,
    },
    S4A {
        stage_token: kw::S4A,
        difficulty: MainDifficulty,
        midboss: Option<kw::Midboss>,
    },
    S4B {
        stage_token: kw::S4B,
        difficulty: MainDifficulty,
        midboss: Option<kw::Midboss>,
    },
    S5 {
        stage_token: kw::S5,
        difficulty: MainDifficulty,
        midboss: Option<kw::Midboss>,
    },
    S6 {
        stage_token: kw::S6,
        difficulty: MainDifficulty,
        midboss: Option<kw::Midboss>,
    },
    S6A {
        stage_token: kw::S6A,
        difficulty: MainDifficulty,
        midboss: Option<kw::Midboss>,
    },
    S6B {
        stage_token: kw::S6B,
        difficulty: MainDifficulty,
        midboss: Option<kw::Midboss>,
    },
    Extra {
        stage_token: kw::Extra,
        midboss: Option<kw::Midboss>,
    },
    Phantasm {
        stage_token: kw::Phantasm,
        midboss: Option<kw::Midboss>,
    },
    LastWord {
        stage_token: kw::LastWord,
    },
}

impl SpellLocation {
    pub fn to_stage_tokens(&self) -> TokenStream {
        match self {
            Self::S1 { .. } => quote!(Stage::One),
            Self::S2 { .. } => quote!(Stage::Two),
            Self::S3 { .. } => quote!(Stage::Three),
            Self::S4 { .. } => quote!(Stage::Four),
            Self::S4A { .. } => quote!(Stage::FourA),
            Self::S4B { .. } => quote!(Stage::FourB),
            Self::S5 { .. } => quote!(Stage::Five),
            Self::S6 { .. } => quote!(Stage::Six),
            Self::S6A { .. } => quote!(Stage::FinalA),
            Self::S6B { .. } => quote!(Stage::FinalB),
            Self::Extra { .. } => quote!(Stage::Extra),
            Self::Phantasm { .. } => quote!(Stage::Phantasm),
            Self::LastWord { .. } => quote!(Stage::LastWord),
        }
    }

    pub fn to_difficulty_tokens(&self) -> TokenStream {
        match self {
            Self::S1 { difficulty, .. }
            | Self::S2 { difficulty, .. }
            | Self::S3 { difficulty, .. }
            | Self::S4 { difficulty, .. }
            | Self::S4A { difficulty, .. }
            | Self::S4B { difficulty, .. }
            | Self::S5 { difficulty, .. }
            | Self::S6 { difficulty, .. }
            | Self::S6A { difficulty, .. }
            | Self::S6B { difficulty, .. } => difficulty.to_ident_tokens(),
            Self::Extra { .. } => quote!(Difficulty::Extra),
            Self::Phantasm { .. } => quote!(Difficulty::Phantasm),
            Self::LastWord { .. } => quote!(Difficulty::LastWord),
        }
    }

    pub fn difficulty_name(&self) -> &'static str {
        match self {
            Self::S1 { difficulty, .. }
            | Self::S2 { difficulty, .. }
            | Self::S3 { difficulty, .. }
            | Self::S4 { difficulty, .. }
            | Self::S4A { difficulty, .. }
            | Self::S4B { difficulty, .. }
            | Self::S5 { difficulty, .. }
            | Self::S6 { difficulty, .. }
            | Self::S6A { difficulty, .. }
            | Self::S6B { difficulty, .. } => difficulty.difficulty_name(),
            Self::Extra { .. } => "Extra",
            Self::Phantasm { .. } => "Phantasm",
            Self::LastWord { .. } => "Last Word",
        }
    }

    pub fn is_midboss(&self) -> bool {
        match self {
            Self::S1 { midboss, .. }
            | Self::S2 { midboss, .. }
            | Self::S3 { midboss, .. }
            | Self::S4 { midboss, .. }
            | Self::S4A { midboss, .. }
            | Self::S4B { midboss, .. }
            | Self::S5 { midboss, .. }
            | Self::S6 { midboss, .. }
            | Self::S6A { midboss, .. }
            | Self::S6B { midboss, .. }
            | Self::Extra { midboss, .. }
            | Self::Phantasm { midboss, .. } => midboss.is_some(),
            Self::LastWord { .. } => false,
        }
    }
}

impl Parse for SpellLocation {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::S1) {
            if input.peek2(kw::Midboss) {
                Ok(Self::S1 {
                    stage_token: input.parse()?,
                    midboss: input.parse()?,
                    difficulty: input.parse()?,
                })
            } else {
                Ok(Self::S1 {
                    stage_token: input.parse()?,
                    difficulty: input.parse()?,
                    midboss: input.parse()?,
                })
            }
        } else if lookahead.peek(kw::S2) {
            if input.peek2(kw::Midboss) {
                Ok(Self::S2 {
                    stage_token: input.parse()?,
                    midboss: input.parse()?,
                    difficulty: input.parse()?,
                })
            } else {
                Ok(Self::S2 {
                    stage_token: input.parse()?,
                    difficulty: input.parse()?,
                    midboss: input.parse()?,
                })
            }
        } else if lookahead.peek(kw::S3) {
            if input.peek2(kw::Midboss) {
                Ok(Self::S3 {
                    stage_token: input.parse()?,
                    midboss: input.parse()?,
                    difficulty: input.parse()?,
                })
            } else {
                Ok(Self::S3 {
                    stage_token: input.parse()?,
                    difficulty: input.parse()?,
                    midboss: input.parse()?,
                })
            }
        } else if lookahead.peek(kw::S4) {
            if input.peek2(kw::Midboss) {
                Ok(Self::S4 {
                    stage_token: input.parse()?,
                    midboss: input.parse()?,
                    difficulty: input.parse()?,
                })
            } else {
                Ok(Self::S4 {
                    stage_token: input.parse()?,
                    difficulty: input.parse()?,
                    midboss: input.parse()?,
                })
            }
        } else if lookahead.peek(kw::S4A) {
            if input.peek2(kw::Midboss) {
                Ok(Self::S4A {
                    stage_token: input.parse()?,
                    midboss: input.parse()?,
                    difficulty: input.parse()?,
                })
            } else {
                Ok(Self::S4A {
                    stage_token: input.parse()?,
                    difficulty: input.parse()?,
                    midboss: input.parse()?,
                })
            }
        } else if lookahead.peek(kw::S4B) {
            if input.peek2(kw::Midboss) {
                Ok(Self::S4B {
                    stage_token: input.parse()?,
                    midboss: input.parse()?,
                    difficulty: input.parse()?,
                })
            } else {
                Ok(Self::S4B {
                    stage_token: input.parse()?,
                    difficulty: input.parse()?,
                    midboss: input.parse()?,
                })
            }
        } else if lookahead.peek(kw::S5) {
            if input.peek2(kw::Midboss) {
                Ok(Self::S5 {
                    stage_token: input.parse()?,
                    midboss: input.parse()?,
                    difficulty: input.parse()?,
                })
            } else {
                Ok(Self::S5 {
                    stage_token: input.parse()?,
                    difficulty: input.parse()?,
                    midboss: input.parse()?,
                })
            }
        } else if lookahead.peek(kw::S6) {
            if input.peek2(kw::Midboss) {
                Ok(Self::S6 {
                    stage_token: input.parse()?,
                    midboss: input.parse()?,
                    difficulty: input.parse()?,
                })
            } else {
                Ok(Self::S6 {
                    stage_token: input.parse()?,
                    difficulty: input.parse()?,
                    midboss: input.parse()?,
                })
            }
        } else if lookahead.peek(kw::S6A) {
            if input.peek2(kw::Midboss) {
                Ok(Self::S6A {
                    stage_token: input.parse()?,
                    midboss: input.parse()?,
                    difficulty: input.parse()?,
                })
            } else {
                Ok(Self::S6A {
                    stage_token: input.parse()?,
                    difficulty: input.parse()?,
                    midboss: input.parse()?,
                })
            }
        } else if lookahead.peek(kw::S6B) {
            if input.peek2(kw::Midboss) {
                Ok(Self::S6B {
                    stage_token: input.parse()?,
                    midboss: input.parse()?,
                    difficulty: input.parse()?,
                })
            } else {
                Ok(Self::S6B {
                    stage_token: input.parse()?,
                    difficulty: input.parse()?,
                    midboss: input.parse()?,
                })
            }
        } else if lookahead.peek(kw::Extra) {
            Ok(Self::Extra {
                stage_token: input.parse()?,
                midboss: input.parse()?,
            })
        } else if lookahead.peek(kw::Phantasm) {
            Ok(Self::Phantasm {
                stage_token: input.parse()?,
                midboss: input.parse()?,
            })
        } else if lookahead.peek(kw::LastWord) {
            Ok(Self::LastWord {
                stage_token: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

pub struct SpellEntry {
    location: SpellLocation,
    id: (LitInt, u32),
    name: (LitStr, String),
}

impl SpellEntry {
    pub fn new(location: SpellLocation, id: LitInt, name: LitStr, offset: u32) -> Self {
        let parsed_id: u32 = id.base10_parse().unwrap();
        let parsed_name = name.value();

        Self {
            location,
            id: (id, parsed_id + offset),
            name: (name, parsed_name),
        }
    }

    pub fn id(&self) -> u32 {
        self.id.1
    }

    pub fn name(&self) -> &str {
        &self.name.1
    }

    pub fn id_span(&self) -> &LitInt {
        &self.id.0
    }

    pub fn spell_def_tokens(
        self,
        game_ident: &Ident,
        duplicate_names: &HashMap<String, bool>,
    ) -> TokenStream {
        let difficulty = self.location.to_difficulty_tokens();
        let stage = self.location.to_stage_tokens();
        let is_midboss = self.location.is_midboss();

        let name = if duplicate_names.get(self.name()).copied().unwrap_or(false) {
            format!("{} ({})", self.name(), self.location.difficulty_name())
        } else {
            self.name.1.clone()
        };

        quote! {
            SpellCardInfo::<#game_ident> {
                name: #name,
                difficulty: #difficulty,
                stage: #stage,
                is_midboss: #is_midboss
            }
        }
    }
}
