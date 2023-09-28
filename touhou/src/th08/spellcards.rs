use touhou_macros::spellcards;

use super::{Difficulty, Stage, Touhou8};

spellcards! {
    /// Identifies a spell card from Touhou 8.
    Game: Touhou8,
    Expected: 222,
    S1: {
        Midboss: [
            {
                Hard: #1 "Firefly Sign \"Earthly Meteor\"",
                Lunatic: #2 "Firefly Sign \"Earthly Comet\""
            }
        ],
        Boss: [
            Easy | Normal | Hard | Lunatic: #3 "Lamp Sign \"Firefly Phenomenon\"",
            {
                Easy: #7 "Wriggle Sign \"Little Bug\"",
                Normal: #8 "Wriggle Sign \"Little Bug Storm\"",
                Hard: #9 "Wriggle Sign \"Nightbug Storm\"",
                Lunatic: #10 "Wriggle Sign \"Nightbug Tornado\"",
            }
        ],
        LastSpell: [
            Normal | Hard | Lunatic: #11 "Hidden Bug \"Endless Night Seclusion\""
        ]
    },
    S2: {
        Midboss: [
            {
                Easy | Normal: #14 "Vocal Sign \"Hooting in the Night\"",
                Hard | Lunatic: #16 "Vocal Sign \"Howl of the Horned Owl\"",
            }
        ],
        Boss: [
            {
                Easy | Normal: #18 "Moth Sign \"Hawk Moth's Insect Curse\"",
                Hard: #20 "Toxin Sign \"Poisonous Moth's Scales\"",
                Lunatic: #21 "Deadly Toxin \"Poisonous Moth's Dance in the Dark\"",
            },
            Easy | Normal | Hard | Lunatic: #22 "Hawk Sign \"Ill-Starred Dive\"",
            Easy | Normal | Hard | Lunatic: #26 "Night-Blindness \"Song of the Night Sparrow\"",
        ],
        LastSpell: [
            Normal | Hard | Lunatic: #30 "Night-Sparrow \"Midnight Chorus-Master\""
        ]
    },
    S3: {
        Midboss: [
            Easy | Normal | Hard | Lunatic: #33 "Spiritual Birth \"First Pyramid\""
        ],
        Boss: [
            Normal | Hard | Lunatic: #37 "Origin Sign \"Ephemerality 137\"",
            {

                Easy: #40 "Ambition Sign \"Buretsu Crisis\"",
                Normal: #41 "Ambition Sign \"Masakado Crisis\"",
                Hard: #42 "Ambition Sign \"Yoshimitsu Crisis\"",
                Lunatic: #43 "Ambition Sign \"General Headquarters Crisis\"",
            },
            {
                Easy: #44 "Land Sign \"Three Sacred Treasures - Sword\"",
                Normal: #45 "Land Sign \"Three Sacred Treasures - Orb\"",
                Hard: #46 "Land Sign \"Three Sacred Treasures - Mirror\"",
                Lunatic: #47 "Land Scheme \"Three Sacred Treasures - Country\"",
            },
            {
                Easy | Normal: #48 "Ending Sign \"Phantasmal Emperor\"",
                Hard | Lunatic: #50 "Pseudo-History \"The Legend of Gensokyo\"",
            }
        ],
        LastSpell: [
            Normal | Hard | Lunatic: #52 "Future \"Gods' Realm\""
        ]
    },
    S4A: {
        Midboss: [
            {
                Easy | Normal: #55 "Dream Sign \"Duplex Barrier\"",
                Hard | Lunatic: #57 "Dream Land \"Great Duplex Barrier\""
            },
            {
                Easy | Normal: #59 "Spirit Sign \"Fantasy Seal -Spread-\"",
                Hard | Lunatic: #61 "Scattered Spirit \"Fantasy Seal -Worn-\"",
            }
        ],
        Boss: [
            {
                Easy | Normal: #63 "Dream Sign \"Evil-Sealing Circle\"",
                Hard: #65 "Divine Arts \"Omnidirectional Oni-Binding Circle\"",
                Lunatic: #66 "Divine Arts \"Omnidirectional Dragon-Slaying Circle\"",
            },
            {
                Easy | Normal: #67 "Spirit Sign \"Fantasy Seal -Concentrate-\"",
                Hard | Lunatic: #69 "Migrating Spirit \"Fantasy Seal -Marred-\"",
            },
            {
                Easy | Normal: #71 "Boundary \"Duplex Danmaku Barrier\"",
                Hard | Lunatic: #73 "Great Barrier \"Hakurei Danmaku Barrier\"",
            }
        ],
        LastSpell: [
            Normal | Hard | Lunatic: #75 "Divine Spirit \"Fantasy Seal -Blink-\""
        ]
    },
    S4B: {
        Midboss: [
            {
                Easy | Normal: #78 "Magic Sign \"Milky Way\"",
                Hard | Lunatic: #80 "Magic Space \"Asteroid Belt\"",
            },
            {
                Easy | Normal: #82 "Magic Sign \"Stardust Reverie\"",
                Hard | Lunatic: #84 "Black Magic \"Event Horizon\"",
            }
        ],
        Boss: [
            {
                Easy | Normal: #86 "Love Sign \"Non-Directional Laser\"",
                Hard | Lunatic: #88 "Love Storm \"Starlight Typhoon\""
            },
            {
                Easy | Normal: #90 "Love Sign \"Master Spark\"",
                Hard | Lunatic: #92 "Loving Heart \"Double Spark\"",
            },
            {
                Easy | Normal: #94 "Light Sign \"Earthlight Ray\"",
                Hard | Lunatic: #96 "Light Blast \"Shoot the Moon\"",
            }
        ],
        LastSpell: [
            {
                Normal | Hard: #98 "Magicannon \"Final Spark\"",
                Lunatic: #100 "Magicannon \"Final Master Spark\"",
            }
        ]
    },
    S5: {
        Boss: [
            {
                Easy | Normal: #101 "Wave Sign \"Red-Eyed Hypnosis (Mind Shaker)\"",
                Hard | Lunatic: #103 "Illusion Wave \"Red-Eyed Hypnosis (Mind Blowing)\"",
            },
            {
                Easy | Normal: #105 "Lunatic Sign \"Hallucinogenic Tuning (Visionary Tuning)\"",
                Hard | Lunatic: #107 "Lunatic Gaze \"Lunatic Stare Tuning (Illusion Seeker)\"",
            },
            {
                Easy | Normal: #109 "Loafing Sign \"Life & Spirit Stopping (Idling Wave)\"",
                Hard | Lunatic: #111 "Indolence \"Life & Spirit Stopping (Mind Stopper)\"",
            },
            Easy | Normal | Hard | Lunatic: #113 "Spread Sign \"Moon of Truth (Invisible Full Moon)\"",
        ],
        LastSpell: [
            Normal | Hard | Lunatic: #117 "Lunar Eyes \"Lunar Rabbit's Remote Mesmerism (Tele-Mesmerism)\""
        ]
    },
    S6A: {
        Midboss: [
            Easy | Normal | Hard | Lunatic: #120 "Spacesphere \"Earth in a Pot\"",
        ],
        Boss: [
            {
                Easy | Normal: #124 "Awakened God \"Memories of the Age of the Gods\"",
                Hard | Lunatic: #126 "God Sign \"Genealogy of the Celestials\"",
            },
            {
                Easy | Normal: #128 "Revival \"Seimei Yūgi -Life Game-\"",
                Hard | Lunatic: #130 "Resurrection \"Rising Game\"",
            },
            {
                Easy | Normal: #132 "Leading God \"Omoikane's Device\"",
                Hard | Lunatic: #134 "Mind of God \"Omoikane's Brain\""
            },
            Easy | Normal | Hard | Lunatic: #136 "Curse of the Heavens \"Apollo 13\"",
            Easy | Normal | Hard | Lunatic: #140 "Esoterica \"Astronomical Entombing\""
        ],
        LastSpell: [
            Easy | Normal | Hard | Lunatic: #144 "Forbidden Elixir \"Hourai Elixir\""
        ]
    },
    S6B: {
        Midboss: [
            Easy | Normal | Hard | Lunatic: #148 "Medicine Sign \"Galaxy in a Pot\"",
        ],
        Boss: [
            {
                Easy | Normal: #152 "Impossible Request \"Dragon's Neck's Jewel -Five-Colored Shots-\"",
                Hard | Lunatic: #154 "Divine Treasure \"Brilliant Dragon Bullet\"",
            },
            {
                Easy | Normal: #156 "Impossible Request \"Buddha's Stone Bowl -Indomitable Will-\"",
                Hard | Lunatic: #158 "Divine Treasure \"Buddhist Diamond\"",
            },
            {
                Easy | Normal: #160 "Impossible Request \"Robe of Fire Rat -Patient Mind-\"",
                Hard | Lunatic: #162 "Divine Treasure \"Salamander Shield\""
            },
            {
                Easy | Normal: #164 "Impossible Request \"Swallow's Cowrie Shell -Everlasting Life-\"",
                Hard | Lunatic: #166 "Divine Treasure \"Life Spring Infinity\""
            },
            {
                Easy | Normal: #168 "Impossible Request \"Bullet Branch of Hourai -Rainbow Danmaku-\"",
                Hard | Lunatic: #170 "Divine Treasure \"Jeweled Branch of Hourai -Dreamlike Paradise-\""
            }
        ],
        LastSpell: [
            {
                Easy: #172 "\"End of Imperishable Night -New Moon-\"",
                Normal: #173 "\"End of Imperishable Night -Crescent Moon-\"",
                Hard: #174 "\"End of Imperishable Night -1st Quarter's Moon-\"",
                Lunatic: #175 "\"End of Imperishable Night -Matsuyoi-\""
            },
            {
                Easy: #176 "\"End of Imperishable Night -11 o'Clock-\"",
                Normal: #177 "\"End of Imperishable Night -Half to Midnight-\"",
                Hard: #178 "\"End of Imperishable Night -Midnight-\"",
                Lunatic: #179 "\"End of Imperishable Night -Half Past Midnight-\""
            },
            {
                Easy: #180 "\"End of Imperishable Night -1 o'Clock-\"",
                Normal: #181 "\"End of Imperishable Night -Half Past 1-\"",
                Hard: #182 "\"End of Imperishable Night -Dead of Night-\"",
                Lunatic: #183 "\"End of Imperishable Night -Half Past 2-\""
            },
            {
                Easy: #184 "\"End of Imperishable Night -3 o'Clock-\"",
                Normal: #185 "\"End of Imperishable Night -Half Past 3-\"",
                Hard: #186 "\"End of Imperishable Night -4 o'Clock-\"",
                Lunatic: #187 "\"End of Imperishable Night -Half Past 4-\""
            },
            {
                Easy: #188 "\"End of Imperishable Night -Morning Mist-\"",
                Normal: #189 "\"End of Imperishable Night -Dawn-\"",
                Hard: #190 "\"End of Imperishable Night -Morning Star-\"",
                Lunatic: #191 "\"End of Imperishable Night -Rising World-\""
            }
        ]
    },
    Extra: {
        Midboss: [
            #192 "Past \"Old History of an Untrodden Land -Old History-\"",
            #193 "Reincarnation \"Ichijou Returning Bridge\"",
            #194 "Future \"New History of Fantasy -Next History-\""
        ],
        Boss: [
            #195 "Limiting Edict \"Curse of Tsuki-no-Iwakasa\"",
            #196 "Undying \"Fire Bird -Feng Wing Ascension-\"",
            #197 "Fujiwara \"Wounds of Metsuzai Temple\"",
            #198 "Undying \"Xu Fu's Dimension\"",
            #199 "Expiation \"Honest Man's Death\"",
            #200 "Hollow Being \"Wu\"",
            #201 "Inextinguishable \"Phoenix's Tail\"",
            #202 "Hourai \"South Wind, Clear Sky -Fujiyama Volcano-\"",
            #203 "\"Possessed by Phoenix\"",
            #204 "\"Hourai Doll\"",
        ],
        LastSpell: [
            #205 "\"Imperishable Shooting\""
        ]
    },
    LastWord: [
        #206 "\"Unseasonal Butterfly Storm\"",
        #207 "\"Blind Nightbird\"",
        #208 "\"Emperor of the Land of the Rising Sun\"",
        #209 "\"Stare of the Hazy Phantom Moon (Lunatic Red Eyes)\"",
        #210 "\"Heaven Spider's Butterfly-Capturing Web\"",
        #211 "\"Tree-Ocean of Hourai\"",
        #212 "\"Phoenix Rebirth\"",
        #213 "\"Ancient Duper\"",
        #214 "\"Total Purification\"",
        #215 "\"Fantasy Nature\"",
        #216 "\"Blazing Star\"",
        #217 "\"Deflation World\"",
        #218 "\"Matsuyoi-Reflecting Satellite Slash\"",
        #219 "\"The Phantom of the Grand Guignol\"",
        #220 "\"Scarlet Destiny\"",
        #221 "\"Saigyouji Parinirvana\"",
        #222 "\"Profound Danmaku Barrier -Phantasm, Foam, and Shadow-\""
    ]
}
