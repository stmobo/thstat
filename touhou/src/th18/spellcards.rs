use touhou_macros::spellcards;

use crate::types::{Difficulty, SpellCardInfo, Stage};

pub(crate) const SPELL_CARDS: &[SpellCardInfo] = spellcards! {
    0 (S1 Easy): "Beckon Sign \"Danmaku Business Boom\"",
    1 (S1 Normal): "Beckon Sign \"Danmaku Business Boom\"",
    2 (S1 Hard): "Beckon Sign \"Danmaku Business Boom\"",
    3 (S1 Lunatic): "Beckon Sign \"Danmaku Business Boom\"",

    4 (S1 Easy): "Beckon Sign \"Shoot Away Disaster, Beckon in Fortune\"",
    5 (S1 Normal): "Beckon Sign \"Shoot Away Disaster, Beckon in Fortune\"",
    6 (S1 Hard): "Beckon Sign \"Shoot Away Disaster, Beckon in Fortune\"",
    7 (S1 Lunatic): "Beckon Sign \"Shoot Away Disaster, Beckon in Fortune\"",

    8 (S2 Easy): "Forest Sign \"Tree-Veiling Technology\"",
    9 (S2 Normal): "Forest Sign \"Tree-Veiling Technology\"",
    10 (S2 Hard): "Forest Sign \"Extreme Tree-Veiling Technology\"",
    11 (S2 Lunatic): "Forest Sign \"True Tree-Veiling Technology\"",

    12 (S2 Easy): "Forest Sign \"Innermost Forest Region\"",
    13 (S2 Normal): "Forest Sign \"Innermost Forest Region\"",
    14 (S2 Hard): "Forest Sign \"Extreme Innermost Forest Region\"",
    15 (S2 Lunatic): "Forest Sign \"True Innermost Forest Region\"",

    16 (S2 Easy): "Leaf Skill \"Green Spiral\"",
    17 (S2 Normal): "Leaf Skill \"Green Spiral\"",
    18 (S2 Hard): "Leaf Skill \"Green Cyclone\"",
    19 (S2 Lunatic): "Leaf Skill \"Green Tornado\"",

    20 (S3 Easy): "Mountain Sign \"Heaven-Shaking Kumomagusa\"",
    21 (S3 Normal): "Mountain Sign \"Heaven-Shaking Kumomagusa\"",
    22 (S3 Hard): "Mountain Apparition \"Astonishing Kumomagusa\"",
    23 (S3 Lunatic): "Mountain Apparition \"Astonishing Kumomagusa\"",

    24 (S3 Easy): "Mountain Sign \"Usuyukisou Shining with Bewitching Light\"",
    25 (S3 Normal): "Mountain Sign \"Usuyukisou Shining with Bewitching Light\"",
    26 (S3 Hard): "Mountain Apparition \"Usuyukisou of Thronging Crowds of Youma\"",
    27 (S3 Lunatic): "Mountain Apparition \"Usuyukisou of Thronging Crowds of Youma\"",

    28 (S3 Easy): "Mountain Flower \"Komakusa of Massacre\"",
    29 (S3 Normal): "Mountain Flower \"Komakusa of Massacre\"",
    30 (S3 Hard): "Mountain Flower \"Mountain Queen of Massacre\"",
    31 (S3 Lunatic): "Mountain Flower \"Mountain Queen of Massacre\"",

    32 (S4 Easy): "Orb Sign \"Rainbow Dragon Yin-Yang Orbs\"",
    33 (S4 Normal): "Orb Sign \"Rainbow Dragon Yin-Yang Orbs\"",
    34 (S4 Hard): "Orb Sign \"Rainbow Dragon Yin-Yang Orbs\"",
    35 (S4 Lunatic): "Orb Sign \"Yin-Yang Divine Orbs\"",

    36 (S4 Easy): "Jeweled General \"Queen of Yin-Yang Sphere\"",
    37 (S4 Normal): "Jeweled General \"Queen of Yin-Yang Sphere\"",
    38 (S4 Hard): "Queenly Gem \"Beyond the Rainbow Door\"",
    39 (S4 Lunatic): "Queenly Gem \"Beyond the Rainbow Door\"",

    40 (S4 Easy): "\"Yin-Yang Suffocation\"",
    41 (S4 Normal): "\"Yin-Yang Suffocation\"",
    42 (S4 Hard): "\"Yin-Yang Suffocation\"",
    43 (S4 Lunatic): "\"Yin-Yang Suffocation\"",

    44 (S5 Easy): "Calamitous Star \"Dance of Star-Sparked Wildfire\"",
    45 (S5 Normal): "Calamitous Star \"Dance of Star-Sparked Wildfire\"",
    46 (S5 Hard): "Calamitous Star \"Wild Dance of Star-Sparked Wildfire\"",
    47 (S5 Lunatic): "Calamitous Star \"Wild Dance of Star-Sparked Wildfire\"",

    48 (S5 Easy): "Stellar Wind \"Dance of Dazzling Iridescence\"",
    49 (S5 Normal): "Stellar Wind \"Dance of Dazzling Iridescence\"",
    50 (S5 Hard): "Stellar Wind \"Wild Dance of Dazzling Iridescence\"",
    51 (S5 Lunatic): "Stellar Wind \"Wild Dance of Dazzling Iridescence\"",

    52 (S5 Easy): "Luminous Horse \"Dance of Sky-Racing Heavenly Steeds\"",
    53 (S5 Normal): "Luminous Horse \"Dance of Sky-Racing Heavenly Steeds\"",
    54 (S5 Hard): "Luminous Horse \"Wild Dance of Sky-Racing Heavenly Steeds\"",
    55 (S5 Lunatic): "Luminous Horse \"Wild Dance of Sky-Racing Heavenly Steeds\"",

    56 (S5 Easy): "Rainbow Illumination \"Clear and Tranquil Wind and Moon\"",
    57 (S5 Normal): "Rainbow Illumination \"Clear and Tranquil Wind and Moon\"",
    58 (S5 Hard): "Rainbow Illumination \"Clear and Tranquil Wind and Moon\"",
    59 (S5 Lunatic): "Rainbow Illumination \"Clear and Tranquil Wind and Moon\"",

    60 (S6 Easy): "\"An Offering to the Ownerless\"",
    61 (S6 Normal): "\"An Offering to the Ownerless\"",
    62 (S6 Hard): "\"An Offering to the Ownerless\"",
    63 (S6 Lunatic): "\"An Offering to the Ownerless\"",

    64 (S6 Easy): "\"Danmaku Hoarder's Obsession\"",
    65 (S6 Normal): "\"Danmaku Hoarder's Obsession\"",
    66 (S6 Hard): "\"Danmaku Hoarder's Obsession\"",
    67 (S6 Lunatic): "\"Danmaku Hoarder's Obsession\"",

    68 (S6 Easy): "\"Bullet Market\"",
    69 (S6 Normal): "\"Bullet Market\"",
    70 (S6 Hard): "\"High Density Bullet Market\"",
    71 (S6 Lunatic): "\"Danmaku Free Market\"",

    72 (S6 Easy): "\"Rainbow Ring of People\"",
    73 (S6 Normal): "\"Rainbow Ring of People\"",
    74 (S6 Hard): "\"Rainbow Ring of People\"",
    75 (S6 Lunatic): "\"Rainbow Ring of People\"",

    76 (S6 Easy): "\"Bullet Dominion\"",
    77 (S6 Normal): "\"Bullet Dominion\"",
    78 (S6 Hard): "\"Tyrannical Bullet Dominion\"",
    79 (S6 Lunatic): "\"Inhumane Bullet Dominion\"",

    80 (S6 Easy): "\"Asylum of Danmaku\"",
    81 (S6 Normal): "\"Asylum of Danmaku\"",
    82 (S6 Hard): "\"Asylum of Danmaku\"",
    83 (S6 Lunatic): "\"Asylum of Danmaku\"",

    84 (Extra Midboss): "Fox Sign \"Fox Winder\"",
    85 (Extra Midboss): "Kuda-gitsune \"Cylinder Fox\"",
    86 (Extra Midboss): "Stellar Fox \"Dance of Heavenly Foxes and Dragon Stars\"",

    87 (Extra): "Kodoku \"Cannibalistic Insect\"",
    88 (Extra): "Kodoku \"Cave Swarmer\"",
    89 (Extra): "Kodoku \"Sky Pendra\"",
    90 (Extra): "Mining \"Ever-Accumulating Mine Dump\"",
    91 (Extra): "Mining \"Mine Blast\"",
    92 (Extra): "Mining \"Shield Method of the Youkai\"",
    93 (Extra): "Oomukade \"Snake Eater\"",
    94 (Extra): "Oomukade \"Dragon Eater\"",
    95 (Extra): "\"Kodoku Gourmet\"",
    96 (Extra): "\"Mushihime-sama's Resplendent and Restless Daily Life\""
};
