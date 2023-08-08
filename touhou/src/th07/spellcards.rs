use touhou_macros::spellcards;

use crate::types::{Difficulty, SpellCardInfo, Stage};

pub(crate) const SPELL_CARDS: &[SpellCardInfo] = spellcards! {
    0 (S1 Hard Midboss): "Frost Sign \"Frost Columns\"",
    1 (S1 Lunatic Midboss): "Frost Sign \"Frost Columns -Lunatic-\"",

    2 (S1 Easy): "Cold Sign \"Lingering Cold -Easy-\"",
    3 (S1 Normal): "Cold Sign \"Lingering Cold\"",
    4 (S1 Hard): "Cold Sign \"Lingering Cold -Hard-\"",
    5 (S1 Lunatic): "Cold Sign \"Lingering Cold -Lunatic-\"",

    6 (S1 Easy): "Winter Sign \"Flower Wither Away -Easy-\"",
    7 (S1 Normal): "Winter Sign \"Flower Wither Away\"",
    8 (S1 Hard): "White Sign \"Undulation Ray\"",
    9 (S1 Lunatic): "Mystic Sign \"Table-Turning\"",

    10 (S2 Easy Midboss): "Hermit Sign \"Fenghuang Egg -Easy-\"",
    11 (S2 Normal Midboss): "Hermit Sign \"Fenghuang Egg\"",
    12 (S2 Hard Midboss): "Hermit Sign \"Fenghuang's Spread Wings\"",
    13 (S2 Lunatic Midboss): "Hermit Sign \"Fenghuang's Spread Wings -Lunatic-\"",

    14 (S2 Easy): "Shikigami Sign \"Soaring Seiman -Easy-\"",
    15 (S2 Normal): "Shikigami Sign \"Soaring Seiman\"",
    16 (S2 Hard): "Yin Yang \"Douman-Seiman\"",
    17 (S2 Lunatic): "Yin Yang \"Seiman-Daimon\"",

    18 (S2 Easy): "Heaven Sign \"Tianxian's Rumbling -Easy-\"",
    19 (S2 Normal): "Heaven Sign \"Tianxian's Rumbling\"",
    20 (S2 Hard): "Flight Sign \"Soaring Idaten\"",
    21 (S2 Lunatic): "Servant Sign \"Gouhou-Tendou's Wild Dance\"",

    22 (S2 Easy): "Hermit Sign \"Shikai Immortality -Easy-\"",
    23 (S2 Normal): "Hermit Sign \"Shikai Immortality\"",
    24 (S2 Hard): "Oni Sign \"Kimon Konjin\"",
    25 (S2 Lunatic): "Direction Sign \"Kimontonkou\"",

    26 (S3 Hard Midboss): "Puppeteer Sign \"Maiden's Bunraku\"",
    27 (S3 Lunatic Midboss): "Puppeteer Sign \"Maiden's Bunraku -Lunatic-\"",

    28 (S3 Easy): "Blue Sign \"Fraternal French Dolls -Easy-\"",
    29 (S3 Normal): "Blue Sign \"Fraternal French Dolls\"",
    30 (S3 Hard): "Blue Sign \"Fraternal French Dolls -Hard-\"",
    31 (S3 Lunatic): "Blue Sign \"Fraternal Orléans Dolls\"",

    32 (S3 Easy): "Scarlet Sign \"Red-Haired Dutch Dolls -Easy-\"",
    33 (S3 Normal): "Scarlet Sign \"Red-Haired Dutch Dolls\"",
    34 (S3 Hard): "White Sign \"Chalk-White Russian Dolls\"",
    35 (S3 Lunatic): "White Sign \"Chalk-White Russian Dolls -Lunatic-\"",

    36 (S3 Easy): "Darkness Sign \"Foggy London Dolls -Easy-\"",
    37 (S3 Normal): "Darkness Sign \"Foggy London Dolls\"",
    38 (S3 Hard): "Cycle Sign \"Samsaric Tibetan Dolls\"",
    39 (S3 Lunatic): "Elegant Sign \"Spring Kyoto Dolls\"",

    40 (S3 Easy): "Malediction \"Magically Luminous Shanghai Dolls -Easy-\"",
    41 (S3 Normal): "Malediction \"Magically Luminous Shanghai Dolls\"",
    42 (S3 Hard): "Malediction \"Magically Luminous Shanghai Dolls -Hard-\"",
    43 (S3 Lunatic): "Malediction \"Hanged Hourai Dolls\"",

    44 (S4 Easy): "Noisy Sign \"Phantom Dinning -Easy-\"",
    45 (S4 Normal): "Noisy Sign \"Phantom Dinning\"",
    46 (S4 Hard): "Noisy Sign \"Live Poltergeist\"",
    47 (S4 Lunatic): "Noisy Sign \"Live Poltergeist -Lunatic-\"",

    48 (S4 Easy): "String Performance \"Guarneri del Gesù -Easy-\"",
    49 (S4 Normal): "String Performance \"Guarneri del Gesù\"",
    50 (S4 Hard): "Divine Strings \"Stradivarius\"",
    51 (S4 Lunatic): "Fake Strings \"Pseudo Stradivarius\"",

    52 (S4 Easy): "Trumpet Spirit \"Hino Phantasm -Easy-\"",
    53 (S4 Normal): "Trumpet Spirit \"Hino Phantasm\"",
    54 (S4 Hard): "Nether Trumpet \"Ghost Clifford\"",
    55 (S4 Lunatic): "Nether Trumpet \"Ghost Clifford -Lunatic-\"",

    56 (S4 Easy): "Nether Keys \"Fazioli Nether Performance -Easy-\"",
    57 (S4 Normal): "Nether Keys \"Fazioli Nether Performance\"",
    58 (S4 Hard): "Key Spirit \"Bösendorfer Divine Performance\"",
    59 (S4 Lunatic): "Key Spirit \"Bösendorfer Divine Performance -Lunatic-\"",

    60 (S4 Easy): "Funeral Concert \"Prism Concerto -Easy-\"",
    61 (S4 Normal): "Funeral Concert \"Prism Concerto\"",
    62 (S4 Hard): "Noisy Funeral \"Stygian Riverside\"",
    63 (S4 Lunatic): "Noisy Funeral \"Stygian Riverside -Lunatic-\"",

    64 (S4 Easy): "Great Funeral Concert \"Spirit Wheel Concerto Grosso -Easy-\"",
    65 (S4 Normal): "Great Funeral Concert \"Spirit Wheel Concerto Grosso\"",
    66 (S4 Hard): "Great Funeral Concert \"Spirit Wheel Concerto Grosso: Revised\"",
    67 (S4 Lunatic): "Great Funeral Concert \"Spirit Wheel Concerto Grosso: Wondrous\"",

    68 (S5 Easy Midboss): "Ghost Sword \"Fasting of the Young Preta -Easy-\"",
    69 (S5 Normal Midboss): "Ghost Sword \"Fasting of the Young Preta\"",
    70 (S5 Hard Midboss): "Preta Sword \"Scroll of the Preta Realm\"",
    71 (S5 Lunatic Midboss): "Hungry King Sword \"Ten Kings' Retribution on the Preta\"",

    72 (S5 Easy): "Hell Realm Sword \"Two Hundred Yojana in One Slash -Easy-\"",
    73 (S5 Normal): "Hell Realm Sword \"Two Hundred Yojana in One Slash\"",
    74 (S5 Hard): "Hell Fire Sword \"Sudden Phantom Formation Slash of Karmic Wind\"",
    75 (S5 Lunatic): "Hell God Sword \"Sudden Divine Severing of Karmic Wind\"",

    76 (S5 Easy): "Animal Realm Sword \"Karmic Punishment of the Idle and Unfocused -Easy-\"",
    77 (S5 Normal): "Animal Realm Sword \"Karmic Punishment of the Idle and Unfocused\"",
    78 (S5 Hard): "Asura Sword \"Obsession with the Present World\"",
    79 (S5 Lunatic): "Asura Sword \"Obsession with the Present World -Lunatic-\"",

    80 (S5 Easy): "Human Realm Sword \"Fantasy of Entering Enlightenment -Easy-\"",
    81 (S5 Normal): "Human Realm Sword \"Fantasy of Entering Enlightenment\"",
    82 (S5 Hard): "Human Era Sword \"Great Enlightenment Appearing and Disappearing\"",
    83 (S5 Lunatic): "Human God Sword \"Constancy of the Conventional Truth\"",

    84 (S5 Easy): "Heaven Sword \"Five Signs of the Dying Deva -Easy-\"",
    85 (S5 Normal): "Heaven Sword \"Five Signs of the Dying Deva\"",
    86 (S5 Hard): "Deva Realm Sword \"Displeasure of the Seven Hakus\"",
    87 (S5 Lunatic): "Heaven God Sword \"Three Kons, Seven Hakus\"",

    88 (S6 Easy Midboss): "Six Realms Sword \"A Single Thought and Infinite Kalpas -Easy-\"",
    89 (S6 Normal Midboss): "Six Realms Sword \"A Single Thought and Infinite Kalpas\"",
    90 (S6 Hard Midboss): "Six Realms Sword \"A Single Thought and Infinite Kalpas -Hard-\"",
    91 (S6 Lunatic Midboss): "Six Realms Sword \"A Single Thought and Infinite Kalpas -Lunatic-\"",

    92 (S6 Easy): "Losing Hometown \"Death of One's Home -Wandering Soul-\"",
    93 (S6 Normal): "Losing Hometown \"Death of One's Home -Past Sin-\"",
    94 (S6 Hard): "Losing Hometown \"Death of One's Home -Trackless Path-\"",
    95 (S6 Lunatic): "Losing Hometown \"Death of One's Home -Suicide-\"",

    96 (S6 Easy): "Deadly Dance \"Law of Mortality -Bewilderment-\"",
    97 (S6 Normal): "Deadly Dance \"Law of Mortality -Dead Butterfly-\"",
    98 (S6 Hard): "Deadly Dance \"Law of Mortality -Poisonous Moth-\"",
    99 (S6 Lunatic): "Deadly Dance \"Law of Mortality -Demon World-\"",

    100 (S6 Easy): "Flowery Soul \"Ghost Butterfly\"",
    101 (S6 Normal): "Flowery Soul \"Swallowtail Butterfly\"",
    102 (S6 Hard): "Flowery Soul \"Deep-Rooted Butterfly\"",
    103 (S6 Lunatic): "Flowery Soul \"Butterfly Delusion\"",

    104 (S6 Easy): "Subtle Melody \"Repository of Hirokawa -False Spirit-\"",
    105 (S6 Normal): "Subtle Melody \"Repository of Hirokawa -Dead Spirit-\"",
    106 (S6 Hard): "Subtle Melody \"Repository of Hirokawa -Phantom Spirit-\"",
    107 (S6 Lunatic): "Subtle Melody \"Repository of Hirokawa -Divine Spirit-\"",

    108 (S6 Easy): "Cherry Blossom Sign \"Perfect Ink-Black Cherry Blossom -Seal-\"",
    109 (S6 Normal): "Cherry Blossom Sign \"Perfect Ink-Black Cherry Blossom -Self-Loss-\"",
    110 (S6 Hard): "Cherry Blossom Sign \"Perfect Ink-Black Cherry Blossom -Spring Sleep-\"",
    111 (S6 Lunatic): "Cherry Blossom Sign \"Perfect Ink-Black Cherry Blossom -Bloom-\"",

    112 (S6 Easy): "\"Resurrection Butterfly -10% Reflowering-\"",
    113 (S6 Normal): "\"Resurrection Butterfly -30% Reflowering-\"",
    114 (S6 Hard): "\"Resurrection Butterfly -50% Reflowering-\"",
    115 (S6 Lunatic): "\"Resurrection Butterfly -80% Reflowering-\"",

    116 (Extra Midboss): "Oni Sign \"Blue Oni, Red Oni\"",
    117 (Extra Midboss): "Kishin \"Soaring Bishamonten\"",

    118 (Extra): "Shikigami \"Senko Thoughtful Meditation\"",
    119 (Extra): "Shikigami \"Banquet of the Twelve General Gods\"",
    120 (Extra): "Shiki Brilliance \"Kitsune-Tanuki Youkai Laser\"",
    121 (Extra): "Shiki Brilliance \"Charming Siege from All Sides\"",
    122 (Extra): "Shiki Brilliance \"Princess Tenko -Illusion-\"",
    123 (Extra): "Shiki Shot \"Ultimate Buddhist\"",
    124 (Extra): "Shiki Shot \"Unilateral Contact\"",
    125 (Extra): "Shikigami \"Chen\"",
    126 (Extra): "\"Kokkuri-san's Contract\"",
    127 (Extra): "Illusion God \"Descent of Izuna Gongen\"",

    128 (Phantasm Midboss): "Shikigami \"Protection of Zenki and Goki\"",
    129 (Phantasm Midboss): "Shikigami \"Channeling Dakiniten\"",

    130 (Phantasm): "Barrier \"Curse of Dreams and Reality\"",
    131 (Phantasm): "Barrier \"Balance of Motion and Stillness\"",
    132 (Phantasm): "Barrier \"Mesh of Light and Darkness\"",
    133 (Phantasm): "Evil Spirits \"Dreamland of Straight and Curve\"",
    134 (Phantasm): "Evil Spirits \"Yukari Yakumo's Spiriting Away\"",
    135 (Phantasm): "Evil Spirits \"Bewitching Butterfly Living in the Zen Temple\"",
    136 (Phantasm): "Sinister Spirits \"Double Black Death Butterfly\"",
    137 (Phantasm): "Shikigami \"Ran Yakumo\"",
    138 (Phantasm): "\"Boundary of Humans and Youkai\"",
    139 (Phantasm): "Barrier \"Boundary of Life and Death\"",
    140 (Phantasm): "Yukari's Arcanum \"Danmaku Barrier\""
};

// spellcard_data! {
//     n: 141,
//     One: {
//         midboss: {
//             [
//                 "Frost Sign \"Frost Columns\"",
//                 "Frost Sign \"Frost Columns -Lunatic-\""
//             ]
//         },
//         boss: {
//             [
//                 "Cold Sign \"Lingering Cold -Easy-\"",
//                 "Cold Sign \"Lingering Cold\"",
//                 "Cold Sign \"Lingering Cold -Hard-\"",
//                 "Cold Sign \"Lingering Cold -Lunatic-\""
//             ],
//             [
//                 "Winter Sign \"Flower Wither Away -Easy-\"",
//                 "Winter Sign \"Flower Wither Away\"",
//                 "White Sign \"Undulation Ray\"",
//                 "Mystic Sign \"Table-Turning\""
//             ]
//         }
//     },
//     Two: {
//         midboss: {
//             [
//                 (
//                     "Hermit Sign \"Fenghuang Egg -Easy-\"",
//                     "Hermit Sign \"Fenghuang Egg\""
//                 ),
//                 "Hermit Sign \"Fenghuang's Spread Wings\"",
//                 "Hermit Sign \"Fenghuang's Spread Wings -Lunatic-\""
//             ]
//         },
//         boss: {
//             [
//                 "Shikigami Sign \"Soaring Seiman -Easy-\"",
//                 "Shikigami Sign \"Soaring Seiman\"",
//                 "Yin Yang \"Douman-Seiman\"",
//                 "Yin Yang \"Seiman-Daimon\""
//             ],
//             [
//                 "Heaven Sign \"Tianxian's Rumbling -Easy-\"",
//                 "Heaven Sign \"Tianxian's Rumbling\"",
//                 "Flight Sign \"Soaring Idaten\"",
//                 "Servant Sign \"Gouhou-Tendou's Wild Dance\""
//             ],
//             [
//                 "Hermit Sign \"Shikai Immortality -Easy-\"",
//                 "Hermit Sign \"Shikai Immortality\"",
//                 "Oni Sign \"Kimon Konjin\"",
//                 "Direction Sign \"Kimontonkou\""
//             ]
//         }
//     },
//     Three: {
//         midboss: {
//             [
//                 "Puppeteer Sign \"Maiden's Bunraku\"",
//                 "Puppeteer Sign \"Maiden's Bunraku -Lunatic-\""
//             ]
//         },
//         boss: {
//             [
//                 "Blue Sign \"Fraternal French Dolls -Easy-\"",
//                 "Blue Sign \"Fraternal French Dolls\"",
//                 "Blue Sign \"Fraternal French Dolls -Hard-\"",
//                 "Blue Sign \"Fraternal Orléans Dolls\""
//             ],
//             [
//                 "Scarlet Sign \"Red-Haired Dutch Dolls -Easy-\"",
//                 "Scarlet Sign \"Red-Haired Dutch Dolls\"",
//                 "White Sign \"Chalk-White Russian Dolls\"",
//                 "White Sign \"Chalk-White Russian Dolls -Lunatic-\""
//             ],
//             [
//                 "Darkness Sign \"Foggy London Dolls -Easy-\"",
//                 "Darkness Sign \"Foggy London Dolls\"",
//                 "Cycle Sign \"Samsaric Tibetan Dolls\"",
//                 "Elegant Sign \"Spring Kyoto Dolls\""
//             ],
//             [
//                 "Malediction \"Magically Luminous Shanghai Dolls -Easy-\"",
//                 "Malediction \"Magically Luminous Shanghai Dolls\"",
//                 "Malediction \"Magically Luminous Shanghai Dolls -Hard-\"",
//                 "Malediction \"Hanged Hourai Dolls\""
//             ]
//         }
//     },
//     Four: {
//         boss: {
//             [
//                 "Noisy Sign \"Phantom Dinning -Easy-\"",
//                 "Noisy Sign \"Phantom Dinning\"",
//                 "Noisy Sign \"Live Poltergeist\"",
//                 "Noisy Sign \"Live Poltergeist -Lunatic-\""
//             ],
//             [
//                 "String Performance \"Guarneri del Gesù -Easy-\"",
//                 "String Performance \"Guarneri del Gesù\"",
//                 "Divine Strings \"Stradivarius\"",
//                 "Fake Strings \"Pseudo Stradivarius\""
//             ],
//             [
//                 "Trumpet Spirit \"Hino Phantasm -Easy-\"",
//                 "Trumpet Spirit \"Hino Phantasm\"",
//                 "Nether Trumpet \"Ghost Clifford\"",
//                 "Nether Trumpet \"Ghost Clifford -Lunatic-\""
//             ],
//             [
//                 "Nether Keys \"Fazioli Nether Performance -Easy-\"",
//                 "Nether Keys \"Fazioli Nether Performance\"",
//                 "Key Spirit \"Bösendorfer Divine Performance\"",
//                 "Key Spirit \"Bösendorfer Divine Performance -Lunatic-\""
//             ],
//             [
//                 "Funeral Concert \"Prism Concerto -Easy-\"",
//                 "Funeral Concert \"Prism Concerto\"",
//                 "Noisy Funeral \"Stygian Riverside\"",
//                 "Noisy Funeral \"Stygian Riverside -Lunatic-\""
//             ],
//             [
//                 "Great Funeral Concert \"Spirit Wheel Concerto Grosso -Easy-\"",
//                 "Great Funeral Concert \"Spirit Wheel Concerto Grosso\"",
//                 "Great Funeral Concert \"Spirit Wheel Concerto Grosso: Revised\"",
//                 "Great Funeral Concert \"Spirit Wheel Concerto Grosso: Wondrous\""
//             ]
//         }
//     },
//     Five: {
//         midboss: {
//             [
//                 (
//                     "Ghost Sword \"Fasting of the Young Preta -Easy-\"",
//                     "Ghost Sword \"Fasting of the Young Preta\""
//                 ),
//                 "Preta Sword \"Scroll of the Preta Realm\"",
//                 "Hungry King Sword \"Ten Kings' Retribution on the Preta\""
//             ]
//         },
//         boss: {
//             [
//                 "Hell Realm Sword \"Two Hundred Yojana in One Slash -Easy-\"",
//                 "Hell Realm Sword \"Two Hundred Yojana in One Slash\"",
//                 "Hell Fire Sword \"Sudden Phantom Formation Slash of Karmic Wind\"",
//                 "Hell God Sword \"Sudden Divine Severing of Karmic Wind\""
//             ],
//             [
//                 "Animal Realm Sword \"Karmic Punishment of the Idle and Unfocused -Easy-\"",
//                 "Animal Realm Sword \"Karmic Punishment of the Idle and Unfocused\"",
//                 "Asura Sword \"Obsession with the Present World\"",
//                 "Asura Sword \"Obsession with the Present World -Lunatic-\""
//             ],
//             [
//                 "Human Realm Sword \"Fantasy of Entering Enlightenment -Easy-\"",
//                 "Human Realm Sword \"Fantasy of Entering Enlightenment\"",
//                 "Human Era Sword \"Great Enlightenment Appearing and Disappearing\"",
//                 "Human God Sword \"Constancy of the Conventional Truth\""
//             ],
//             [
//                 "Heaven Sword \"Five Signs of the Dying Deva -Easy-\"",
//                 "Heaven Sword \"Five Signs of the Dying Deva\"",
//                 "Deva Realm Sword \"Displeasure of the Seven Hakus\"",
//                 "Heaven God Sword \"Three Kons, Seven Hakus\""
//             ]
//         }
//     },
//     Six: {
//         midboss: {
//             [
//                 (
//                     "Six Realms Sword \"A Single Thought and Infinite Kalpas -Easy-\"",
//                     "Six Realms Sword \"A Single Thought and Infinite Kalpas\""
//                 ),
//                 "Six Realms Sword \"A Single Thought and Infinite Kalpas -Hard-\"",
//                 "Six Realms Sword \"A Single Thought and Infinite Kalpas -Lunatic-\""
//             ]
//         },
//         boss: {
//             [
//                 "Losing Hometown \"Death of One's Home -Wandering Soul-\"",
//                 "Losing Hometown \"Death of One's Home -Past Sin-\"",
//                 "Losing Hometown \"Death of One's Home -Trackless Path-\"",
//                 "Losing Hometown \"Death of One's Home -Suicide-\""
//             ],
//             [
//                 "Deadly Dance \"Law of Mortality -Bewilderment-\"",
//                 "Deadly Dance \"Law of Mortality -Dead Butterfly-\"",
//                 "Deadly Dance \"Law of Mortality -Poisonous Moth-\"",
//                 "Deadly Dance \"Law of Mortality -Demon World-\""
//             ],
//             [
//                 "Flowery Soul \"Ghost Butterfly\"",
//                 "Flowery Soul \"Swallowtail Butterfly\"",
//                 "Flowery Soul \"Deep-Rooted Butterfly\"",
//                 "Flowery Soul \"Butterfly Delusion\""
//             ],
//             [
//                 "Subtle Melody \"Repository of Hirokawa -False Spirit-\"",
//                 "Subtle Melody \"Repository of Hirokawa -Dead Spirit-\"",
//                 "Subtle Melody \"Repository of Hirokawa -Phantom Spirit-\"",
//                 "Subtle Melody \"Repository of Hirokawa -Divine Spirit-\""
//             ],
//             [
//                 "Cherry Blossom Sign \"Perfect Ink-Black Cherry Blossom -Seal-\"",
//                 "Cherry Blossom Sign \"Perfect Ink-Black Cherry Blossom -Self-Loss-\"",
//                 "Cherry Blossom Sign \"Perfect Ink-Black Cherry Blossom -Spring Sleep-\"",
//                 "Cherry Blossom Sign \"Perfect Ink-Black Cherry Blossom -Bloom-\""
//             ],
//             [
//                 "Resurrection Butterfly -10% Reflowering-",
//                 "Resurrection Butterfly -30% Reflowering-",
//                 "Resurrection Butterfly -50% Reflowering-",
//                 "Resurrection Butterfly -80% Reflowering-"
//             ]
//         }
//     },
//     {
//         midboss: [
//             "Oni Sign \"Blue Oni, Red Oni\"",
//             "Kishin \"Soaring Bishamonten\""
//         ],
//         boss: [
//             "Shikigami \"Senko Thoughtful Meditation\"",
//             "Shikigami \"Banquet of the Twelve General Gods\"",
//             "Shiki Brilliance \"Kitsune-Tanuki Youkai Laser\"",
//             "Shiki Brilliance \"Charming Siege from All Sides\"",
//             "Shiki Brilliance \"Princess Tenko -Illusion-\"",
//             "Shiki Shot \"Ultimate Buddhist\"",
//             "Shiki Shot \"Unilateral Contact\"",
//             "Shikigami \"Chen\"",
//             "\"Kokkuri-san's Contract\"",
//             "Illusion God \"Descent of Izuna Gongen\""
//         ]
//     },
//     {
//         midboss: [
//             "Shikigami \"Protection of Zenki and Goki\"",
//             "Shikigami \"Channeling Dakiniten\""
//         ],
//         boss: [
//             "Barrier \"Curse of Dreams and Reality\"",
//             "Barrier \"Balance of Motion and Stillness\"",
//             "Barrier \"Mesh of Light and Darkness\"",
//             "Evil Spirits \"Dreamland of Straight and Curve\"",
//             "Evil Spirits \"Yukari Yakumo's Spiriting Away\"",
//             "Evil Spirits \"Bewitching Butterfly Living in the Zen Temple\"",
//             "Sinister Spirits \"Double Black Death Butterfly\"",
//             "Shikigami \"Ran Yakumo\"",
//             "\"Boundary of Humans and Youkai\"",
//             "Barrier \"Boundary of Life and Death\"",
//             "Yukari's Arcanum \"Danmaku Barrier\""
//         ]
//     }
//}
