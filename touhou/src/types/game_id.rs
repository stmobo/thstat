use std::error::Error;
use std::fmt::Display;

#[derive(Debug, Copy, Clone)]
pub struct InvalidGameId(u8);

impl Display for InvalidGameId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid game ID {}", self.0)
    }
}

impl Error for InvalidGameId {}

macro_rules! define_game_info {
    {
        $(
            $id:ident : {
                id_number: $id_num:literal,
                title: $title:literal,
                subtitle: $subtitle:literal
            }
        ),*
    } => {
        /// An enumeration listing every game covered by this crate.
        ///
        /// This can be used to identify games at runtime, for example when tagging game-specific values with their corresponding games during serialization and deserialization.
        ///
        /// This type can be converted to and from corresponding `u8` values for each of the games; for example, `GameId::PCB` can be converted into `7u8` and back.
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
        )]
        #[serde(try_from = "u8", into = "u8")]
        #[non_exhaustive]
        pub enum GameId {
            $(
                #[doc = concat!("*Touhou ", $title, " ~ ", $subtitle, "*")]
                $id
            ),*
        }

        impl GameId {
            /// Converts a game's number into the corresponding `GameId` variant.
            ///
            /// This is equivalent to this type's implementation of [`TryFrom<u8>`].
            ///
            /// # Examples
            ///
            /// ```
            /// assert_eq!(GameId::new(7u8), GameId::PCB);
            /// assert_eq!(GameId::new(10u8), GameId::MoF);
            /// ```
            pub const fn new(number: u8) -> Result<Self, InvalidGameId> {
                match number {
                    $(
                        $id_num => Ok(Self::$id),
                    )*
                    v => Err(InvalidGameId(v)),
                }
            }

            /// Gets the number of this game in the series (e.g. `7` for Touhou 7).
            ///
            /// This is equivalent to this type's implementation of [`From<GameId>`].
            ///
            /// # Examples
            ///
            /// ```
            /// let pcb = GameID::PCB;
            /// assert_eq!(pcb.number(), 7u8);
            ///
            /// let mof = GameID::MoF;
            /// assert_eq!(mof.number(), 10u8);
            /// ```
            pub const fn number(self) -> u8 {
                match self {
                    $(
                        GameId::$id => $id_num
                    ),*
                }
            }

            /// Gets a name for this game in the format "Touhou N", where "N" is the number of this game.
            ///
            /// # Examples
            ///
            /// ```
            /// let pcb = GameId::PCB;
            /// assert_eq!(pcb.numbered_name(), "Touhou 7");
            ///
            /// let mof = GameId::MoF;
            /// assert_eq!(mof.numbered_name(), "Touhou 10");
            /// ```
            pub const fn numbered_name(&self) -> &'static str {
                match *self {
                    $(
                        Self::$id => concat!("Touhou ", stringify!($num_name))
                    ),*
                }
            }

            /// Gets the abbreviated form of this game's English subtitle.
            ///
            /// The return value from this function is just a stringified version of the variant name.
            ///
            /// # Examples
            ///
            /// ```
            /// let pcb = GameId::PCB;
            /// assert_eq!(pcb.numbered_name(), "PCB");
            ///
            /// let mof = GameId::MoF;
            /// assert_eq!(mof.numbered_name(), "MoF");
            /// ```
            pub const fn abbreviation(&self) -> &'static str {
                match *self {
                    $(
                        Self::$id => stringify!($id)
                    ),*
                }
            }

            /// Gets a romanized form of the Japanese title for this game.
            ///
            /// # Examples
            ///
            /// ```
            /// let pcb = GameId::PCB;
            /// assert_eq!(pcb.title(), "Youyoumu");
            ///
            /// let mof = GameId::MoF;
            /// assert_eq!(mof.title(), "Fuujinroku");
            /// ```
            pub const fn title(&self) -> &'static str {
                match *self {
                    $(
                        Self::$id => $title
                    ),*
                }
            }

            /// Gets the English subtitle for this game.
            ///
            /// # Examples
            ///
            /// ```
            /// let pcb = GameId::PCB;
            /// assert_eq!(pcb.title(), "Perfect Cherry Blossom");
            ///
            /// let mof = GameId::MoF;
            /// assert_eq!(mof.title(), "Mountain of Faith");
            /// ```
            pub const fn subtitle(&self) -> &'static str {
                match *self {
                    $(
                        Self::$id => $subtitle
                    ),*
                }
            }

            /// Gets the full title for this game.
            ///
            /// # Examples
            ///
            /// ```
            /// let pcb = GameId::PCB;
            /// assert_eq!(pcb.full_title(), "Touhou Youyoumu ~ Perfect Cherry Blossom");
            ///
            /// let mof = GameId::MoF;
            /// assert_eq!(mof.full_title(), "Touhou Fuujinroku ~ Mountain of Faith");
            /// ```
            pub const fn full_title(&self) -> &'static str {
                match *self {
                    $(
                        Self::$id => concat!("Touhou ", $title, " ~ ", $subtitle)
                    ),*
                }
            }
        }

        impl From<GameId> for u8 {
            fn from(value: GameId) -> Self {
                match value {
                    $(
                        GameId::$id => $id_num
                    ),*
                }
            }
        }

        impl TryFrom<u8> for GameId {
            type Error = InvalidGameId;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    $(
                        $id_num => Ok(Self::$id),
                    )*
                    v => Err(InvalidGameId(v)),
                }
            }
        }
    };
}

define_game_info! {
    PCB: {
        id_number: 7,
        title: "Youyoumu",
        subtitle: "Perfect Cherry Blossom"
    },
    IN: {
        id_number: 8,
        title: "Eiyashou",
        subtitle: "Imperishable Night"
    },
    MoF: {
        id_number: 10,
        title: "Fuujinroku",
        subtitle: "Mountain of Faith"
    },
    LoLK: {
        id_number: 15,
        title: "Kanjuden",
        subtitle: "Legacy of Lunatic Kingdom"
    }
}

impl serde::de::Expected for GameId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.pad(self.abbreviation())
    }
}

impl From<GameId> for u16 {
    fn from(value: GameId) -> Self {
        <GameId as Into<u8>>::into(value) as u16
    }
}

impl TryFrom<u16> for GameId {
    type Error = anyhow::Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        let v: u8 = value.try_into()?;
        GameId::try_from(v).map_err(|e| e.into())
    }
}

impl Display for GameId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(self.abbreviation())
    }
}

pub(crate) trait VisitGame: Sized {
    type Output;

    fn visit_th07(self) -> Self::Output {
        unimplemented!("Support for Touhou 7 was not compiled")
    }

    fn visit_th08(self) -> Self::Output {
        unimplemented!("Support for Touhou 8 was not compiled")
    }

    fn visit_th10(self) -> Self::Output {
        unimplemented!("Support for Touhou 10 was not compiled")
    }

    fn visit_th15(self) -> Self::Output {
        unimplemented!("Support for Touhou 15 was not compiled")
    }

    fn accept_id(self, game_id: GameId) -> Self::Output {
        match game_id {
            GameId::PCB => self.visit_th07(),
            GameId::IN => self.visit_th08(),
            GameId::MoF => self.visit_th10(),
            GameId::LoLK => self.visit_th15(),
        }
    }
}
