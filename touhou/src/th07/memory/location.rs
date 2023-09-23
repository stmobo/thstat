use touhou_macros::define_locations;

use crate::th07::{Stage, Touhou7};

define_locations! {
    #[stage_type = "Stage"]
    #[game = "Touhou7"]
    #[exclude_stages = "Extra, Phantasm"]
    Location {
        One: {
            540 => Section,
            1341 => Section,
            2656 => Midboss [
                Nonspell,
                Spells(1..=2)
            ],
            3107 => Section,
            5402 => Boss [
                Nonspell,
                Spells(3..=6),
                Nonspell,
                Spells(7..=10)
            ]
        },
        Two: {
            390 => Section,
            2826 => Midboss [
                Nonspell,
                Spells(11..=14)
            ],
            3366 => Section,
            7647 => Boss [
                Nonspell,
                Spells(15..=18),
                Nonspell,
                Spells(19..=22),
                Spells(23..=28)
            ]
        },
        Three: {
            390 => Section,
            821 => Midboss Nonspell,
            854 => Section,
            1805 => Section,
            1858 => Midboss [
                Nonspell,
                Spells(27..=28)
            ],
            3393 => Boss [
                Nonspell,
                Spells(29..=32),
                Nonspell,
                Spells(33..=36),
                Nonspell,
                Spells(37..=40),
                Spells(41..=44)
            ]
        },
        Four: {
            80 => Section,
            1948 => Section,
            3028 => Section,
            4288 => Section,
            7122 => Midboss Nonspell,
            7964 => Section,
            10136 => Section,
            11396 => Section,
            13166 => Section,
            14826 => Boss [
                Nonspell,
                Nonspell,
                Spells(45..=48),
                Nonspell,
                Spells(49..=60),
                Spells(61..=64),
                Spells(65..=68)
            ]
        },
        Five: {
            440 => Section,
            840 => Section,
            2550 => Section,
            4820 => Midboss [
                Nonspell,
                Spells(69..=72)
            ],
            4883 => Section,
            6113 => Boss [
                Nonspell,
                Spells(73..=76),
                Nonspell,
                Spells(77..=80),
                Spells(81..=84),
                Spells(85..=88)
            ]
        },
        Six: {
            660 => Section,
            1180 => Section("Spam"),
            1914 => Midboss [
                Nonspell,
                Spells(89..=92)
            ],
            2518 => Boss [
                Nonspell,
                Spells(93..=96),
                Nonspell,
                Spells(97..=100),
                Nonspell,
                Spells(101..=104),
                Nonspell,
                Spells(105..=108),
                Spells(109..=112),
                Spells(113..=115)
            ]
        }
    }
}
