use std::fmt::Display;
use std::num::NonZeroU16;

use touhou_macros::spellcards;

use super::{Difficulty, Stage, Touhou10};
use crate::types::{GameId, GameValue, InvalidCardId, SpellCardInfo, SpellType};

const SPELL_CARDS: &[SpellCardInfo<Touhou10>; 110] = spellcards! {
    Game: Touhou10,
    S1: {
        Midboss: [
            Hard | Lunatic: #1 "Leaf Sign \"Falling Leaves of Madness\"",
        ],
        Boss: [
            {
                Easy | Normal: #3 "Fall Sign \"Autumn Sky\"",
                Hard | Lunatic: #5 "Fall Sign \"The Fall Sky and a Maiden's Heart\"",
            },
            {
                Easy | Normal: #7 "Plenty Sign \"Owotoshi Harvester\"",
                Hard | Lunatic: #9 "Bumper Crop \"Promise of the Wheat God\"",
            }
        ],
    },
    S2: {
        Midboss: [
            {
                Easy | Normal: #11 "Misfortune Sign \"Bad Fortune\"",
                Hard | Lunatic: #13 "Misfortune Sign \"Biorhythm of the Misfortune God\"",
            }
        ],
        Boss: [
            {
                Easy | Normal: #15 "Flawed Sign \"Broken Amulet\"",
                Hard | Lunatic: #17 "Scar \"Broken Charm of Protection\"",
            },
            {
                Easy | Normal: #19 "Evil Spirit \"Misfortune's Wheel\"",
                Hard | Lunatic: #21 "Tragic Fate \"Old Lady Ohgane's Fire\"",
            },
            {
                Easy | Normal: #23 "Wound Sign \"Pain Flow\"",
                Hard | Lunatic: #25 "Wound Sign \"Exiled Doll\"",
            }
        ]
    },
    S3: {
        Midboss: [
            {
                Easy | Normal: #27 "Optics \"Optical Camouflage\"",
                Hard | Lunatic: #29 "Optics \"Hydro Camouflage\"",
            }
        ],
        Boss: [
            {
                Easy | Normal: #31 "Flood \"Ooze Flooding\"",
                Hard: #33 "Flood \"Diluvial Mere\"",
                Lunatic: #34 "Drown \"Trauma in the Glimmering Depths\"",
            },
            {
                Easy | Normal: #35 "Water Sign \"Kappa's Pororoca\"",
                Hard: #37 "Water Sign \"Kappa's Flash Flood\"",
                Lunatic: #38 "Water Sign \"Kappa's Great Illusionary Waterfall\"",
            },
            {
                Easy | Normal: #39 "Kappa \"Monster Cucumber\"",
                Hard: #41 "Kappa \"Exteeeending Aaaaarm\"",
                Lunatic: #42 "Kappa \"Spin the Cephalic Plate\"",
            }
        ]
    },
    S4: {
        Boss: [
            {
                Easy | Normal: #43 "Crossroad Sign \"Crossroads of Heaven\"",
                Hard | Lunatic: #45 "Crossroad Sign \"Saruta Cross\"",
            },
            {
                Easy | Normal: #47 "Wind God \"Wind God's Leaf-Veiling\"",
                Hard: #49 "Wind God \"Tengu's Fall Wind\"",
                Lunatic: #50 "Wind God \"Storm Day\"",
            },
            {
                Normal | Hard: #51 "\"Illusionary Dominance\"",
                Lunatic: #53 "\"Peerless Wind God\"",
            },
            {
                Easy | Normal: #54 "Blockade Sign \"Mountain God's Procession\"",
                Hard: #56 "Blockade Sign \"Advent of the Divine Grandson\"",
                Lunatic: #57 "Blockade Sign \"Terukuni Shining Through Heaven and Earth\"",
            }
        ]
    },
    S5: {
        Midboss: [
            {
                Easy | Normal: #58 "Esoterica \"Gray Thaumaturgy\"",
                Hard: #60 "Esoterica \"Forgotten Ritual\"",
                Lunatic: #61 "Esoterica \"Secretly Inherited Art of Danmaku\"",
            }
        ],
        Boss: [
            {
                Easy | Normal: #62 "Miracle \"Daytime Guest Stars\"",
                Hard: #64 "Miracle \"Night with Bright Guest Stars\"",
                Lunatic: #65 "Miracle \"Night with Overly Bright Guest Stars\"",
            },
            {
                Easy | Normal: #66 "Sea Opening \"The Day the Sea Split\"",
                Hard | Lunatic: #68 "Sea Opening \"Moses' Miracle\"",
            },
            {
                Easy | Normal: #70 "Preparation \"Star Ritual to Call the Godly Winds\"",
                Hard | Lunatic: #72 "Preparation \"Summon Takeminakata\"",
            },
            {
                Easy | Normal: #74 "Miracle \"God's Wind\"",
                Hard | Lunatic: #76 "Great Miracle \"Yasaka's Divine Wind\"",
            }
        ]
    },
    S6: {
        Boss: [
            {
                Easy | Normal: #78 "Divine Festival \"Expanded Onbashira\"",
                Hard | Lunatic: #80 "Weird Festival \"Medoteko Boisterous Dance\"",
            },
            {
                Easy | Normal: #82 "Rice Porridge in Tube \"God's Rice Porridge\"",
                Hard: #84 "Forgotten Grain \"Unremembered Crop\"",
                Lunatic: #85 "Divine Grain \"Divining Crop\"",
            },
            {
                Easy | Normal: #86 "Sacrifice Sign \"Misayama Hunting Shrine Ritual\"",
                Hard: #88 "Mystery \"Kuzui Clear Water\"",
                Lunatic: #89 "Mystery \"Yamato Torus\"",
            },
            {
                Easy | Normal: #90 "Heaven's Stream \"Miracle of Otensui\"",
                Hard | Lunatic: #92 "Heaven's Dragon \"Source of Rains\"",
            },
            {
                Easy | Normal: #94 "\"Mountain of Faith\"",
                Hard | Lunatic: #96 "\"Divine Virtues of Wind God\""
            }
        ]
    },
    Extra: {
        Midboss: [
            #98 "God Sign \"Beautiful Spring like Suiga\"",
            #99 "God Sign \"Ancient Fate Linked by Cedars\"",
            #100 "God Sign \"Omiwatari that God Walked\""
        ],
        Boss: [
            #101 "Party Start \"Two Bows, Two Claps, and One Bow\"",
            #102 "Native God \"Lord Long-Arm and Lord Long-Leg\"",
            #103 "Divine Tool \"Moriya's Iron Ring\"",
            #104 "Spring Sign \"Jade of the Horrid River\"",
            #105 "Frog Hunt \"The Snake Eats the Croaking Frog\"",
            #106 "Native God \"Seven Stones and Seven Trees\"",
            #107 "Native God \"Froggy Braves the Wind and Rain\"",
            #108 "Native God \"Red Frogs of Houei Four\"",
            #109 "\"Suwa War ~ Native Myth vs. Central Myth\"",
            #110 "Scourge Sign \"Mishaguji-sama\""
        ]
    }
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpellId(NonZeroU16);

impl SpellId {
    pub fn card_info(&self) -> &'static SpellCardInfo<Touhou10> {
        &SPELL_CARDS[(self.0.get() - 1) as usize]
    }
}

impl From<SpellId> for u32 {
    fn from(value: SpellId) -> Self {
        value.0.get() as u32
    }
}

impl TryFrom<u32> for SpellId {
    type Error = InvalidCardId;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if let Ok(Some(value)) = <u16 as TryFrom<u32>>::try_from(value).map(NonZeroU16::new) {
            if value.get() <= (SPELL_CARDS.len() as u16) {
                return Ok(Self(value));
            }
        }

        Err(InvalidCardId::InvalidCard(
            GameId::MoF,
            value,
            SPELL_CARDS.len() as u32,
        ))
    }
}

impl Display for SpellId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl GameValue for SpellId {
    type RawValue = u32;
    type ConversionError = InvalidCardId;

    fn game_id(&self) -> GameId {
        GameId::MoF
    }

    fn raw_id(&self) -> u32 {
        (*self).into()
    }

    fn from_raw(id: u32, game: GameId) -> Result<Self, InvalidCardId> {
        if game == GameId::MoF {
            id.try_into()
        } else {
            Err(InvalidCardId::UnexpectedGameId(game, GameId::MoF))
        }
    }

    fn name(&self) -> &'static str {
        self.card_info().name
    }
}
