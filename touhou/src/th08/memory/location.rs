use touhou_macros::define_locations;

use crate::th08::{SpellId, Stage, Touhou8};

define_locations! {
    #[game(Touhou8)]
    #[exclude_stages = "Extra, LastWord"]
    #[resolve_visibility(pub(self))]
    Location {
        One: {
            340 => Section,
            2875 => Section,
            2935 => Midboss [
                Nonspell,
                Spells(1..=2)
            ],
            4175 => Boss [
                Nonspell,
                Spells(3..=6),
                Nonspell,
                Spells(7..=10),
                LastSpell(11..=13)
            ]
        },
        Two: {
            400 => Section,
            990 => Section,
            2350 => Section,
            3350 => Section,
            4870 => Boss [
                Nonspell,
                Spells(14..=17),
                Nonspell,
                Spells(18..=21),
                Nonspell,
                Spells(22..=25),
                Spells(26..=29),
                LastSpell(30..=32)
            ]
        },
        Three: {
            340 => Section,
            1220 => Section,
            3080 => Midboss [
                Nonspell,
                Spells(33..=36)
            ],
            3103 => Section,
            4663 => Boss [
                Nonspell,
                Spells(37..=39),
                Spells(40..=43),
                Nonspell,
                Spells(44..=47),
                Spells(48..=51),
                LastSpell(52..=54)
            ]
        },
        FourA: {
            340 => Section,
            1180 => Section,
            2240 => Section,
            3180 => Section,
            4903 => Section,
            4962 => Boss [
                Nonspell,
                Spells(55..=58),
                Nonspell,
                Spells(59..=62)
            ],
            5363 => Section,
            6923 => Boss [
                Nonspell,
                Spells(63..=66),
                Nonspell,
                Spells(67..=70),
                Spells(71..=74),
                LastSpell(75..=77)
            ]
        },
        FourB: {
            340 => Section,
            1180 => Section,
            2240 => Section,
            3180 => Section,
            4903 => Section,
            4962 => Boss [
                Nonspell,
                Spells(78..=81),
                Nonspell,
                Spells(82..=85)
            ],
            5363 => Section,
            6923 => Boss [
                Nonspell,
                Spells(86..=89),
                Nonspell,
                Spells(90..=93),
                Spells(94..=97),
                LastSpell(98..=100)
            ]
        },
        Five: {
            200 => Section,
            1040 => Section,
            2990 => Section,
            4530 => Midboss [
                Nonspell,
                Nonspell
            ],
            4651 => Section,
            5971 => Section,
            7481 => Boss [
                Nonspell,
                Spells(101..=104),
                Nonspell,
                Spells(105..=108),
                Nonspell,
                Spells(109..=112),
                Spells(113..=116),
                LastSpell(117..=119)
            ]
        },
        FinalA: {
            200 => Section,
            1140 => Section,
            3400 => Midboss [
                Nonspell,
                Spells(120..=123)
            ],
            4022 => Boss [
                Nonspell,
                Spells(124..=127),
                Nonspell,
                Spells(128..=131),
                Nonspell,
                Spells(132..=135),
                Nonspell,
                Spells(136..=139),
                Spells(140..=143),
                LastSpell(144..=147)
            ]
        },
        FinalB: {
            200 => Section,
            1140 => Section,
            3490 => Midboss [
                Nonspell,
                Spells(148..=151)
            ],
            4022 => Boss [
                Nonspell,
                Spells(152..=155),
                Nonspell,
                Spells(156..=159),
                Nonspell,
                Spells(160..=163),
                Nonspell,
                Spells(164..=167),
                Spells(168..=171),
                LastSpell(
                    172..=175,
                    176..=179,
                    180..=183,
                    184..=187,
                    188..=191
                )
            ]
        }
    }
}

pub(super) fn resolve(state: &super::state::RunState) -> Option<Location> {
    use crate::memory::traits::StageData;

    let stage = state.stage();
    let adj = if stage.stage() == Stage::Two && stage.frame() >= 4870 {
        stage
            .active_boss()
            .and_then(|boss| {
                if boss.active_spell().is_none() {
                    // note: 'nonspell 1' is technically a midnon with no other lifebars,
                    // so we need to distinguish between it and the last boss non.
                    //
                    // therefore, massive hack: check the damage multiplier to
                    // see if the midnon or the boss non is active. the midnon has a higher damage multiplier of 0.125,
                    // while the boss non is something like 0.09 or so?
                    match boss.remaining_lifebars() {
                        0 => {
                            if boss.damage_multiplier() >= 0.125 {
                                Some(StageTwo::BossNonspell1)
                            } else {
                                Some(StageTwo::BossNonspell3)
                            }
                        }
                        1 => Some(StageTwo::BossNonspell2),
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .map(Location::Two)
    } else {
        None
    };

    adj.or_else(|| Location::resolve(state))
}
