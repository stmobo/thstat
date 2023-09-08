use touhou_macros::spellcards;

use super::{Difficulty, Stage, Touhou7};
use crate::types::{SpellCardInfo, SpellType};

pub(crate) const SPELL_CARDS: &[SpellCardInfo<Touhou7>; 141] = spellcards! {
    Game: Touhou7,
    S1: {
        Midboss: [
            {
                Hard: #0 "Frost Sign \"Frost Columns\"",
                Lunatic: #1 "Frost Sign \"Frost Columns -Lunatic-\"",
            }
        ],
        Boss: [
            {
                Easy: #2 "Cold Sign \"Lingering Cold -Easy-\"",
                Normal: #3 "Cold Sign \"Lingering Cold\"",
                Hard: #4 "Cold Sign \"Lingering Cold -Hard-\"",
                Lunatic: #5 "Cold Sign \"Lingering Cold -Lunatic-\"",
            },
            {
                Easy: #6 "Winter Sign \"Flower Wither Away -Easy-\"",
                Normal: #7 "Winter Sign \"Flower Wither Away\"",
                Hard: #8 "White Sign \"Undulation Ray\"",
                Lunatic: #9 "Mystic Sign \"Table-Turning\"",
            }
        ]
    },
    S2: {
        Midboss: [
            {
                Easy: #10 "Hermit Sign \"Fenghuang Egg -Easy-\"",
                Normal: #11 "Hermit Sign \"Fenghuang Egg\"",
                Hard: #12 "Hermit Sign \"Fenghuang's Spread Wings\"",
                Lunatic: #13 "Hermit Sign \"Fenghuang's Spread Wings -Lunatic-\"",
            }
        ],
        Boss: [
            {
                Easy: #14 "Shikigami Sign \"Soaring Seiman -Easy-\"",
                Normal: #15 "Shikigami Sign \"Soaring Seiman\"",
                Hard: #16 "Yin Yang \"Douman-Seiman\"",
                Lunatic: #17 "Yin Yang \"Seiman-Daimon\"",
            },
            {
                Easy: #18 "Heaven Sign \"Tianxian's Rumbling -Easy-\"",
                Normal: #19 "Heaven Sign \"Tianxian's Rumbling\"",
                Hard: #20 "Flight Sign \"Soaring Idaten\"",
                Lunatic: #21 "Servant Sign \"Gouhou-Tendou's Wild Dance\"",
            },
            {
                Easy: #22 "Hermit Sign \"Shikai Immortality -Easy-\"",
                Normal: #23 "Hermit Sign \"Shikai Immortality\"",
                Hard: #24 "Oni Sign \"Kimon Konjin\"",
                Lunatic: #25 "Direction Sign \"Kimontonkou\"",
            }
        ]
    },
    S3: {
        Midboss: [
            {
                Hard: #26 "Puppeteer Sign \"Maiden's Bunraku\"",
                Lunatic: #27 "Puppeteer Sign \"Maiden's Bunraku -Lunatic-\"",
            }
        ],
        Boss: [
            {
                Easy: #28 "Blue Sign \"Fraternal French Dolls -Easy-\"",
                Normal: #29 "Blue Sign \"Fraternal French Dolls\"",
                Hard: #30 "Blue Sign \"Fraternal French Dolls -Hard-\"",
                Lunatic: #31 "Blue Sign \"Fraternal Orléans Dolls\"",
            },
            {
                Easy: #32 "Scarlet Sign \"Red-Haired Dutch Dolls -Easy-\"",
                Normal: #33 "Scarlet Sign \"Red-Haired Dutch Dolls\"",
                Hard: #34 "White Sign \"Chalk-White Russian Dolls\"",
                Lunatic: #35 "White Sign \"Chalk-White Russian Dolls -Lunatic-\"",
            },
            {
                Easy: #36 "Darkness Sign \"Foggy London Dolls -Easy-\"",
                Normal: #37 "Darkness Sign \"Foggy London Dolls\"",
                Hard: #38 "Cycle Sign \"Samsaric Tibetan Dolls\"",
                Lunatic: #39 "Elegant Sign \"Spring Kyoto Dolls\"",
            },
            {
                Easy: #40 "Malediction \"Magically Luminous Shanghai Dolls -Easy-\"",
                Normal: #41 "Malediction \"Magically Luminous Shanghai Dolls\"",
                Hard: #42 "Malediction \"Magically Luminous Shanghai Dolls -Hard-\"",
                Lunatic: #43 "Malediction \"Hanged Hourai Dolls\"",
            }
        ]
    },
    S4: {
        Boss: [
            {
                Easy: #44 "Noisy Sign \"Phantom Dinning -Easy-\"",
                Normal: #45 "Noisy Sign \"Phantom Dinning\"",
                Hard: #46 "Noisy Sign \"Live Poltergeist\"",
                Lunatic: #47 "Noisy Sign \"Live Poltergeist -Lunatic-\"",
            },
            {
                Easy: #48 "String Performance \"Guarneri del Gesù -Easy-\"",
                Normal: #49 "String Performance \"Guarneri del Gesù\"",
                Hard: #50 "Divine Strings \"Stradivarius\"",
                Lunatic: #51 "Fake Strings \"Pseudo Stradivarius\"",

                Easy: #52 "Trumpet Spirit \"Hino Phantasm -Easy-\"",
                Normal: #53 "Trumpet Spirit \"Hino Phantasm\"",
                Hard: #54 "Nether Trumpet \"Ghost Clifford\"",
                Lunatic: #55 "Nether Trumpet \"Ghost Clifford -Lunatic-\"",

                Easy: #56 "Nether Keys \"Fazioli Nether Performance -Easy-\"",
                Normal: #57 "Nether Keys \"Fazioli Nether Performance\"",
                Hard: #58 "Key Spirit \"Bösendorfer Divine Performance\"",
                Lunatic: #59 "Key Spirit \"Bösendorfer Divine Performance -Lunatic-\"",
            },
            {
                Easy: #60 "Funeral Concert \"Prism Concerto -Easy-\"",
                Normal: #61 "Funeral Concert \"Prism Concerto\"",
                Hard: #62 "Noisy Funeral \"Stygian Riverside\"",
                Lunatic: #63 "Noisy Funeral \"Stygian Riverside -Lunatic-\"",
            },
            {
                Easy: #64 "Great Funeral Concert \"Spirit Wheel Concerto Grosso -Easy-\"",
                Normal: #65 "Great Funeral Concert \"Spirit Wheel Concerto Grosso\"",
                Hard: #66 "Great Funeral Concert \"Spirit Wheel Concerto Grosso: Revised\"",
                Lunatic: #67 "Great Funeral Concert \"Spirit Wheel Concerto Grosso: Wondrous\"",
            }
        ]
    },
    S5: {
        Midboss: [
            {
                Easy: #68 "Ghost Sword \"Fasting of the Young Preta -Easy-\"",
                Normal: #69 "Ghost Sword \"Fasting of the Young Preta\"",
                Hard: #70 "Preta Sword \"Scroll of the Preta Realm\"",
                Lunatic: #71 "Hungry King Sword \"Ten Kings' Retribution on the Preta\"",
            }
        ],
        Boss: [
            {
                Easy: #72 "Hell Realm Sword \"Two Hundred Yojana in One Slash -Easy-\"",
                Normal: #73 "Hell Realm Sword \"Two Hundred Yojana in One Slash\"",
                Hard: #74 "Hell Fire Sword \"Sudden Phantom Formation Slash of Karmic Wind\"",
                Lunatic: #75 "Hell God Sword \"Sudden Divine Severing of Karmic Wind\"",
            },
            {
                Easy: #76 "Animal Realm Sword \"Karmic Punishment of the Idle and Unfocused -Easy-\"",
                Normal: #77 "Animal Realm Sword \"Karmic Punishment of the Idle and Unfocused\"",
                Hard: #78 "Asura Sword \"Obsession with the Present World\"",
                Lunatic: #79 "Asura Sword \"Obsession with the Present World -Lunatic-\"",
            },
            {
                Easy: #80 "Human Realm Sword \"Fantasy of Entering Enlightenment -Easy-\"",
                Normal: #81 "Human Realm Sword \"Fantasy of Entering Enlightenment\"",
                Hard: #82 "Human Era Sword \"Great Enlightenment Appearing and Disappearing\"",
                Lunatic: #83 "Human God Sword \"Constancy of the Conventional Truth\"",
            },
            {
                Easy: #84 "Heaven Sword \"Five Signs of the Dying Deva -Easy-\"",
                Normal: #85 "Heaven Sword \"Five Signs of the Dying Deva\"",
                Hard: #86 "Deva Realm Sword \"Displeasure of the Seven Hakus\"",
                Lunatic: #87 "Heaven God Sword \"Three Kons, Seven Hakus\"",
            }
        ]
    },
    S6: {
        Midboss: [
            {
                Easy: #88 "Six Realms Sword \"A Single Thought and Infinite Kalpas -Easy-\"",
                Normal: #89 "Six Realms Sword \"A Single Thought and Infinite Kalpas\"",
                Hard: #90 "Six Realms Sword \"A Single Thought and Infinite Kalpas -Hard-\"",
                Lunatic: #91 "Six Realms Sword \"A Single Thought and Infinite Kalpas -Lunatic-\"",
            }
        ],
        Boss: [
            {
                Easy: #92 "Losing Hometown \"Death of One's Home -Wandering Soul-\"",
                Normal: #93 "Losing Hometown \"Death of One's Home -Past Sin-\"",
                Hard: #94 "Losing Hometown \"Death of One's Home -Trackless Path-\"",
                Lunatic: #95 "Losing Hometown \"Death of One's Home -Suicide-\"",
            },
            {
                Easy: #96 "Deadly Dance \"Law of Mortality -Bewilderment-\"",
                Normal: #97 "Deadly Dance \"Law of Mortality -Dead Butterfly-\"",
                Hard: #98 "Deadly Dance \"Law of Mortality -Poisonous Moth-\"",
                Lunatic: #99 "Deadly Dance \"Law of Mortality -Demon World-\"",
            },
            {
                Easy: #100 "Flowery Soul \"Ghost Butterfly\"",
                Normal: #101 "Flowery Soul \"Swallowtail Butterfly\"",
                Hard: #102 "Flowery Soul \"Deep-Rooted Butterfly\"",
                Lunatic: #103 "Flowery Soul \"Butterfly Delusion\"",
            },
            {
                Easy: #104 "Subtle Melody \"Repository of Hirokawa -False Spirit-\"",
                Normal: #105 "Subtle Melody \"Repository of Hirokawa -Dead Spirit-\"",
                Hard: #106 "Subtle Melody \"Repository of Hirokawa -Phantom Spirit-\"",
                Lunatic: #107 "Subtle Melody \"Repository of Hirokawa -Divine Spirit-\"",
            },
            {
                Easy: #108 "Cherry Blossom Sign \"Perfect Ink-Black Cherry Blossom -Seal-\"",
                Normal: #109 "Cherry Blossom Sign \"Perfect Ink-Black Cherry Blossom -Self-Loss-\"",
                Hard: #110 "Cherry Blossom Sign \"Perfect Ink-Black Cherry Blossom -Spring Sleep-\"",
                Lunatic: #111 "Cherry Blossom Sign \"Perfect Ink-Black Cherry Blossom -Bloom-\"",
            },
            {
                Easy: #112 "\"Resurrection Butterfly -10% Reflowering-\"",
                Normal: #113 "\"Resurrection Butterfly -30% Reflowering-\"",
                Hard: #114 "\"Resurrection Butterfly -50% Reflowering-\"",
                Lunatic: #115 "\"Resurrection Butterfly -80% Reflowering-\"",
            }
        ]
    },
    Extra: {
        Midboss: [
            #116 "Oni Sign \"Blue Oni, Red Oni\"",
            #117 "Kishin \"Soaring Bishamonten\"",
        ],
        Boss: [
            #118 "Shikigami \"Senko Thoughtful Meditation\"",
            #119 "Shikigami \"Banquet of the Twelve General Gods\"",
            #120 "Shiki Brilliance \"Kitsune-Tanuki Youkai Laser\"",
            #121 "Shiki Brilliance \"Charming Siege from All Sides\"",
            #122 "Shiki Brilliance \"Princess Tenko -Illusion-\"",
            #123 "Shiki Shot \"Ultimate Buddhist\"",
            #124 "Shiki Shot \"Unilateral Contact\"",
            #125 "Shikigami \"Chen\"",
            #126 "\"Kokkuri-san's Contract\"",
            #127 "Illusion God \"Descent of Izuna Gongen\""
        ]
    },
    Phantasm: {
        Midboss: [
            #128 "Shikigami \"Protection of Zenki and Goki\"",
            #129 "Shikigami \"Channeling Dakiniten\"",
        ],
        Boss: [
            #130 "Barrier \"Curse of Dreams and Reality\"",
            #131 "Barrier \"Balance of Motion and Stillness\"",
            #132 "Barrier \"Mesh of Light and Darkness\"",
            #133 "Evil Spirits \"Dreamland of Straight and Curve\"",
            #134 "Evil Spirits \"Yukari Yakumo's Spiriting Away\"",
            #135 "Evil Spirits \"Bewitching Butterfly Living in the Zen Temple\"",
            #136 "Sinister Spirits \"Double Black Death Butterfly\"",
            #137 "Shikigami \"Ran Yakumo\"",
            #138 "\"Boundary of Humans and Youkai\"",
            #139 "Barrier \"Boundary of Life and Death\"",
            #140 "Yukari's Arcanum \"Danmaku Barrier\""
        ]
    }
};
