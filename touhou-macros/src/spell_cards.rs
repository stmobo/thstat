use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parenthesized, token, LitInt, LitStr, Result, Token};

mod kw {
    syn::custom_keyword!(S1);
    syn::custom_keyword!(S2);
    syn::custom_keyword!(S3);
    syn::custom_keyword!(S4);
    syn::custom_keyword!(S5);
    syn::custom_keyword!(S6);
    syn::custom_keyword!(Extra);
    syn::custom_keyword!(Phantasm);
    syn::custom_keyword!(Easy);
    syn::custom_keyword!(Normal);
    syn::custom_keyword!(Hard);
    syn::custom_keyword!(Lunatic);
    syn::custom_keyword!(Midboss);
}

enum MainDifficulty {
    Easy(kw::Easy),
    Normal(kw::Normal),
    Hard(kw::Hard),
    Lunatic(kw::Lunatic),
}

impl MainDifficulty {
    fn to_ident_tokens(&self) -> TokenStream {
        match self {
            Self::Easy(_) => quote!(Difficulty::Easy),
            Self::Normal(_) => quote!(Difficulty::Normal),
            Self::Hard(_) => quote!(Difficulty::Hard),
            Self::Lunatic(_) => quote!(Difficulty::Lunatic),
        }
    }

    fn difficulty_name(&self) -> &'static str {
        match self {
            Self::Easy(_) => "Easy",
            Self::Normal(_) => "Normal",
            Self::Hard(_) => "Hard",
            Self::Lunatic(_) => "Lunatic",
        }
    }
}

impl Parse for MainDifficulty {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::Easy) {
            input.parse().map(Self::Easy)
        } else if lookahead.peek(kw::Normal) {
            input.parse().map(Self::Normal)
        } else if lookahead.peek(kw::Hard) {
            input.parse().map(Self::Hard)
        } else if lookahead.peek(kw::Lunatic) {
            input.parse().map(Self::Lunatic)
        } else {
            Err(lookahead.error())
        }
    }
}

#[allow(dead_code)]
enum SpellLocation {
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
    Extra {
        stage_token: kw::Extra,
        midboss: Option<kw::Midboss>,
    },
    Phantasm {
        stage_token: kw::Phantasm,
        midboss: Option<kw::Midboss>,
    },
}

impl SpellLocation {
    fn to_stage_tokens(&self) -> TokenStream {
        match self {
            Self::S1 { .. } => quote!(Stage::One),
            Self::S2 { .. } => quote!(Stage::Two),
            Self::S3 { .. } => quote!(Stage::Three),
            Self::S4 { .. } => quote!(Stage::Four),
            Self::S5 { .. } => quote!(Stage::Five),
            Self::S6 { .. } => quote!(Stage::Six),
            Self::Extra { .. } => quote!(Stage::Extra),
            Self::Phantasm { .. } => quote!(Stage::Phantasm),
        }
    }

    fn to_difficulty_tokens(&self) -> TokenStream {
        match self {
            Self::S1 { difficulty, .. }
            | Self::S2 { difficulty, .. }
            | Self::S3 { difficulty, .. }
            | Self::S4 { difficulty, .. }
            | Self::S5 { difficulty, .. }
            | Self::S6 { difficulty, .. } => difficulty.to_ident_tokens(),
            Self::Extra { .. } => quote!(Difficulty::Extra),
            Self::Phantasm { .. } => quote!(Difficulty::Phantasm),
        }
    }

    fn difficulty_name(&self) -> &'static str {
        match self {
            Self::S1 { difficulty, .. }
            | Self::S2 { difficulty, .. }
            | Self::S3 { difficulty, .. }
            | Self::S4 { difficulty, .. }
            | Self::S5 { difficulty, .. }
            | Self::S6 { difficulty, .. } => difficulty.difficulty_name(),
            Self::Extra { .. } => "Extra",
            Self::Phantasm { .. } => "Phantasm",
        }
    }

    fn is_midboss(&self) -> bool {
        match self {
            Self::S1 { midboss, .. }
            | Self::S2 { midboss, .. }
            | Self::S3 { midboss, .. }
            | Self::S4 { midboss, .. }
            | Self::S5 { midboss, .. }
            | Self::S6 { midboss, .. }
            | Self::Extra { midboss, .. }
            | Self::Phantasm { midboss, .. } => midboss.is_some(),
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
        } else {
            Err(lookahead.error())
        }
    }
}

#[allow(dead_code)]
struct SpellEntry {
    spell_number: LitInt,
    paren_token: token::Paren,
    location: SpellLocation,
    colon_token: Token![:],
    spell_name: LitStr,
}

impl SpellEntry {
    fn spell_def_tokens(&self, is_duplicate: bool) -> TokenStream {
        let difficulty = self.location.to_difficulty_tokens();
        let stage = self.location.to_stage_tokens();
        let is_midboss = self.location.is_midboss();
        let name = if is_duplicate {
            format!(
                "{} ({})",
                self.spell_name.value(),
                self.location.difficulty_name()
            )
        } else {
            self.spell_name.value()
        };

        quote! {
            SpellCardInfo {
                name: #name,
                difficulty: #difficulty,
                stage: #stage,
                is_midboss: #is_midboss
            }
        }
    }
}

impl Parse for SpellEntry {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            spell_number: input.parse()?,
            paren_token: parenthesized!(content in input),
            location: content.parse()?,
            colon_token: input.parse()?,
            spell_name: input.parse()?,
        })
    }
}

pub struct SpellList(Punctuated<SpellEntry, Token![,]>);

impl SpellList {
    pub fn to_list_tokens(&self) -> TokenStream {
        let mut entries = Vec::with_capacity(self.0.len());
        let mut is_duplicate_name = HashMap::new();

        for (i, entry) in self.0.iter().enumerate() {
            match entry.spell_number.base10_parse::<u32>() {
                Ok(id) => {
                    if let Some((prev, _)) = entries.last() {
                        if (i > 0) && (id != (*prev + 1)) {
                            entry
                                .spell_number
                                .span()
                                .unwrap()
                                .warning("Spell numbers are not consecutive")
                                .emit();
                        }
                    }

                    let name = entry.spell_name.value();
                    match is_duplicate_name.entry(name) {
                        Entry::Occupied(mut entry) => {
                            entry.insert(true);
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(false);
                        }
                    };

                    entries.push((id, entry));
                }
                Err(e) => {
                    entry.spell_number.span().unwrap().error(e.to_string());
                    return TokenStream::new();
                }
            }
        }

        entries.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        let first_id = entries.first().unwrap().0;
        let mut out_tokens = Vec::with_capacity(entries.len());

        for (i, (id, entry)) in entries.into_iter().enumerate() {
            let expected_id = (i as u32) + first_id;
            if id != expected_id {
                let msg = format!("duplicate or missing spell ID {}", expected_id);
                return quote! { compile_error!(#msg) };
            }

            let name = entry.spell_name.value();
            out_tokens.push(
                entry.spell_def_tokens(is_duplicate_name.get(&name).copied().unwrap_or(false)),
            );
        }

        quote! { &[ #(#out_tokens),* ] }
    }
}

impl Parse for SpellList {
    fn parse(input: ParseStream) -> Result<Self> {
        input
            .parse_terminated(SpellEntry::parse, Token![,])
            .map(Self)
    }
}
