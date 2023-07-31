use std::fmt::Display;
use std::io::Read;
use std::num::TryFromIntError;
use std::path::Path;
use std::time::Duration;

use sysinfo::{ProcessExt, ProcessRefreshKind, System, SystemExt};

use super::{
    Difficulty, Game as GameTrait, InvalidCardId, InvalidShotType,
    PracticeRecord as PracticeRecordTrait, ScoreFile as ScoreFileTrait, ShotType,
    ShotTypeId as ShotIdTrait, SpellCard, SpellCardId as CardIdTrait, SpellCardInfo,
    SpellCardRecord as CardRecordTrait, Stage,
};
use crate::th07;

macro_rules! define_wrappers {
    {
        $(
            $id:ident : {
                id_number: $id_num:literal,
                numbered_name: $num_name:literal,
                abbreviation: $abbr:literal,
                full_name: $full_name:literal,
                game: $game:ty,
                spell_id: $spell_id:ty,
                shot_id: $shot_id:ty,
                shot_types: [ $($shot_val:ident),* ],
                score_file: $score_file:ty,
                spell_record: $spell_record:ty,
                practice_record: $practice_record:ty
            }
        ),*
    } => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize, sqlx::Type,
        )]
        #[serde(try_from = "u8", into = "u8")]
        #[repr(u8)]
        pub enum GameId {
            $(
                $id = $id_num
            ),*
        }

        impl GameId {
            pub const fn numbered_name(&self) -> &'static str {
                match *self {
                    $(
                        Self::$id => $num_name
                    ),*
                }
            }

            pub const fn abbreviation(&self) -> &'static str {
                match *self {
                    $(
                        Self::$id => $abbr
                    ),*
                }
            }

            pub const fn full_name(&self) -> &'static str {
                match *self {
                    $(
                        Self::$id => $full_name
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
            type Error = anyhow::Error;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    $(
                        $id_num => Ok(Self::$id),
                    )*
                    v => Err(anyhow::anyhow!("invalid game ID {}", v)),
                }
            }
        }

        #[derive(Debug, Clone)]
        pub enum Touhou {
            $(
                $id($game)
            ),*
        }

        impl GameTrait for Touhou {
            type SpellID = SpellId;
            type ShotTypeID = ShotId;
            type ScoreFile = ScoreFile;
            type SpellCardRecord = SpellCardRecord;
            type PracticeRecord = PracticeRecord;

            fn game_id(&self) -> GameId {
                match self {
                    $(
                        &Self::$id(_) => GameId::$id
                    ),*
                }
            }

            fn score_path(&self) -> &Path {
                match &self {
                    $(
                        &Self::$id(game) => game.score_path()
                    ),*
                }
            }

            fn load_score_file<R: Read>(&self, src: R) -> Result<ScoreFile, anyhow::Error> {
                match &self {
                    $(
                        &Self::$id(game) => game.load_score_file(src).map(ScoreFile::from)
                    ),*
                }
            }
        }

        $(
            impl From<$game> for Touhou {
                fn from(value: $game) -> Touhou {
                    Touhou::$id(value)
                }
            }

            impl TryFrom<Touhou> for $game {
                type Error = Touhou;

                fn try_from(value: Touhou) -> Result<$game, Touhou> {
                    if let Touhou::$id(inner) = value {
                        Ok(inner)
                    } else {
                        Err(value)
                    }
                }
            }
        )*

        #[derive(Debug, Copy, Clone)]
        pub enum SpellId {
            $(
                $id($spell_id)
            ),*
        }

        impl CardIdTrait for SpellId {
            fn card_info(&self) -> &'static SpellCardInfo {
                match self {
                    $(
                        &Self::$id(inner) => inner.card_info()
                    ),*
                }
            }

            fn game_id(&self) -> GameId {
                match self {
                    $(
                        &Self::$id(_) => GameId::$id
                    ),*
                }
            }

            fn raw_id(&self) -> u32 {
                match self {
                    $(
                        &Self::$id(inner) => inner.raw_id()
                    ),*
                }
            }

            fn from_raw(id: u32, game: GameId) -> Result<Self, InvalidCardId> {
                match game {
                    $(
                        GameId::$id => <$spell_id>::try_from(id).map(Self::$id)
                    ),*
                }
            }
        }

        impl From<SpellId> for u32 {
            fn from(value: SpellId) -> Self {
                match value {
                    $(
                        SpellId::$id(inner) => ($id_num << 24) | <$spell_id as Into<u32>>::into(inner)
                    ),*
                }
            }
        }

        impl TryFrom<u32> for SpellId {
            type Error = InvalidCardId;

            fn try_from(value: u32) -> Result<Self, Self::Error> {
                let game_id = (value >> 24) & 0xFF;
                let spell_id = value & 0x00FF_FFFF;
                match game_id {
                    $(
                        $id_num => spell_id.try_into().map(Self::$id),
                    )*
                    _v => Err(InvalidCardId::InvalidGameId(game_id as u8)),
                }
            }
        }

        $(
            impl From<$spell_id> for SpellId {
                fn from(value: $spell_id) -> SpellId {
                    SpellId::$id(value)
                }
            }

            impl TryFrom<SpellId> for $spell_id {
                type Error = SpellId;

                fn try_from(value: SpellId) -> Result<$spell_id, SpellId> {
                    if let SpellId::$id(inner) = value {
                        Ok(inner)
                    } else {
                        Err(value)
                    }
                }
            }

            impl From<SpellCard<$game>> for SpellCard<Touhou> {
                fn from(value: SpellCard<$game>) -> SpellCard<Touhou> {
                    let id: SpellId = value.unwrap().into();
                    SpellCard::new(id)
                }
            }

            impl TryFrom<SpellCard<Touhou>> for SpellCard<$game> {
                type Error = SpellCard<Touhou>;

                fn try_from(value: SpellCard<Touhou>) -> Result<SpellCard<$game>, SpellCard<Touhou>> {
                    if let SpellId::$id(inner) = value.unwrap() {
                        Ok(SpellCard::new(inner))
                    } else {
                        Err(value)
                    }
                }
            }
        )*

        #[derive(Debug, Copy, Clone)]
        pub enum ShotId {
            $(
                $id($shot_id)
            ),*
        }

        impl ShotIdTrait for ShotId {
            fn fmt_name(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        &Self::$id(inner) => inner.fmt_name(f)
                    ),*
                }
            }

            fn game_id(&self) -> GameId {
                match self {
                    $(
                        &Self::$id(_) => GameId::$id
                    ),*
                }
            }

            fn raw_id(&self) -> u16 {
                match self {
                    $(
                        &Self::$id(inner) => inner.raw_id()
                    ),*
                }
            }

            fn from_raw(id: u16, game: GameId) -> Result<Self, InvalidShotType> {
                match game {
                    $(
                        GameId::$id => <$shot_id>::try_from(id).map(Self::$id)
                    ),*
                }
            }
        }

        impl From<ShotId> for u16 {
            fn from(value: ShotId) -> Self {
                match value {
                    $(
                        ShotId::$id(inner) => ($id_num << 8) | <$shot_id as Into<u16>>::into(inner)
                    ),*
                }
            }
        }

        impl TryFrom<u16> for ShotId {
            type Error = InvalidShotType;

            fn try_from(value: u16) -> Result<Self, Self::Error> {
                let game_id = (value >> 8) & 0xFF;
                let shot_id = value & 0x00FF;
                match game_id {
                    $(
                        $id_num => shot_id.try_into().map(Self::$id),
                    )*
                    v => Err(InvalidShotType::InvalidGameId(v as u8)),
                }
            }
        }

        $(
            impl From<$shot_id> for ShotId {
                fn from(value: $shot_id) -> ShotId {
                    ShotId::$id(value)
                }
            }

            impl TryFrom<ShotId> for $shot_id {
                type Error = ShotId;

                fn try_from(value: ShotId) -> Result<$shot_id, ShotId> {
                    if let ShotId::$id(inner) = value {
                        Ok(inner)
                    } else {
                        Err(value)
                    }
                }
            }

            impl From<ShotType<$game>> for ShotType<Touhou> {
                fn from(value: ShotType<$game>) -> ShotType<Touhou> {
                    let id: ShotId = value.unwrap().into();
                    ShotType::new(id)
                }
            }

            impl TryFrom<ShotType<Touhou>> for ShotType<$game> {
                type Error = ShotType<Touhou>;

                fn try_from(value: ShotType<Touhou>) -> Result<ShotType<$game>, ShotType<Touhou>> {
                    if let ShotId::$id(inner) = value.unwrap() {
                        Ok(ShotType::new(inner))
                    } else {
                        Err(value)
                    }
                }
            }
        )*

        #[derive(Debug, Clone)]
        pub enum SpellCardRecord {
            $(
                $id($spell_record)
            ),*
        }

        impl SpellCardRecord {
            pub fn game_id(&self) -> GameId {
                match self {
                    $(
                        &Self::$id(_) => GameId::$id
                    ),*
                }
            }
        }

        $(
            impl From<$spell_record> for SpellCardRecord {
                fn from(value: $spell_record) -> SpellCardRecord {
                    SpellCardRecord::$id(value)
                }
            }

            impl TryFrom<SpellCardRecord> for $spell_record {
                type Error = SpellCardRecord;

                fn try_from(value: SpellCardRecord) -> Result<$spell_record, SpellCardRecord> {
                    if let SpellCardRecord::$id(inner) = value {
                        Ok(inner)
                    } else {
                        Err(value)
                    }
                }
            }
        )*

        impl CardRecordTrait<Touhou> for SpellCardRecord {
            fn shot_types(&self) -> &[ShotType<Touhou>] {
                $(
                    const $id: &[ShotType<Touhou>] = &[
                        $(ShotType::new(ShotId::$id(<$shot_id>::$shot_val))),*
                    ];
                )*

                match self {
                    $(
                        &Self::$id(_) => $id
                    ),*
                }
            }

            fn card(&self) -> SpellCard<Touhou> {
                match &self {
                    $(
                        &Self::$id(inner) => inner.card().into()
                    ),*
                }
            }

            fn attempts(&self, shot: &ShotType<Touhou>) -> u32 {
                match &self {
                    $(
                        &Self::$id(inner) => {
                            let shot: ShotType<$game> = ShotType::try_from(*shot).unwrap();
                            inner.attempts(&shot).into()
                        }
                    ),*
                }
            }

            fn captures(&self, shot: &ShotType<Touhou>) -> u32 {
                match &self {
                    $(
                        &Self::$id(inner) => {
                            let shot: ShotType<$game> = ShotType::try_from(*shot).unwrap();
                            inner.captures(&shot).into()
                        }
                    ),*
                }
            }

            fn max_bonus(&self, shot: &ShotType<Touhou>) -> u32 {
                match &self {
                    $(
                        &Self::$id(inner) => {
                            let shot: ShotType<$game> = ShotType::try_from(*shot).unwrap();
                            inner.max_bonuses(&shot).into()
                        }
                    ),*
                }
            }

            fn total_attempts(&self,) -> u32 {
                match &self {
                    $(
                        &Self::$id(inner) => inner.total_attempts().into()
                    ),*
                }
            }

            fn total_captures(&self) -> u32 {
                match &self {
                    $(
                        &Self::$id(inner) => inner.total_captures().into()
                    ),*
                }
            }

            fn total_max_bonus(&self) -> u32 {
                match &self {
                    $(
                        &Self::$id(inner) => inner.total_max_bonus().into()
                    ),*
                }
            }
        }

        #[derive(Debug, Clone)]
        pub enum PracticeRecord {
            $(
                $id($practice_record)
            ),*
        }

        impl PracticeRecord {
            pub fn game_id(&self) -> GameId {
                match self {
                    $(
                        &Self::$id(_) => GameId::$id
                    ),*
                }
            }
        }

        $(
            impl From<$practice_record> for PracticeRecord {
                fn from(value: $practice_record) -> PracticeRecord {
                    PracticeRecord::$id(value)
                }
            }

            impl TryFrom<PracticeRecord> for $practice_record {
                type Error = PracticeRecord;

                fn try_from(value: PracticeRecord) -> Result<$practice_record, PracticeRecord> {
                    if let PracticeRecord::$id(inner) = value {
                        Ok(inner)
                    } else {
                        Err(value)
                    }
                }
            }
        )*

        impl PracticeRecordTrait<Touhou> for PracticeRecord {
            fn stage(&self) -> Stage {
                match self {
                    $(
                        &Self::$id(inner) => inner.stage()
                    ),*
                }
            }

            fn shot_type(&self) -> ShotType<Touhou> {
                match self {
                    $(
                        &Self::$id(inner) => ShotType::new(inner.shot_type().into())
                    ),*
                }
            }

            fn difficulty(&self) -> Difficulty {
                match self {
                    $(
                        &Self::$id(inner) => inner.difficulty()
                    ),*
                }
            }

            fn high_score(&self) -> u32 {
                match self {
                    $(
                        &Self::$id(inner) => inner.high_score()
                    ),*
                }
            }

            fn attempts(&self) -> u32 {
                match self {
                    $(
                        &Self::$id(inner) => inner.attempts()
                    ),*
                }
            }
        }
    };
}

define_wrappers! {
    PCB: {
        id_number: 7,
        numbered_name: "Touhou 7",
        abbreviation: "PCB",
        full_name: "Perfect Cherry Blossom",
        game: th07::Touhou7,
        spell_id: th07::SpellId,
        shot_id: th07::ShotType,
        shot_types: [ReimuA, ReimuB, MarisaA, MarisaB, SakuyaA, SakuyaB],
        score_file: th07::ScoreFile,
        spell_record: th07::SpellCardData,
        practice_record: th07::PracticeData
    }
}

impl serde::de::Expected for GameId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(self.abbreviation())
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
        let value: Result<u8, TryFromIntError> = value.try_into();
        value.map_err(|e| e.into()).and_then(GameId::try_from)
    }
}

impl Display for GameId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.abbreviation())
    }
}

#[derive(Debug, Clone)]
pub struct ScoreFile {
    spells: Vec<SpellCardRecord>,
    practices: Vec<PracticeRecord>,
}

impl ScoreFile {
    pub(crate) fn new(spells: Vec<SpellCardRecord>, practices: Vec<PracticeRecord>) -> Self {
        Self { spells, practices }
    }
}

impl ScoreFileTrait<Touhou> for ScoreFile {
    fn spell_cards(&self) -> &[SpellCardRecord] {
        &self.spells[..]
    }

    fn practice_records(&self) -> &[PracticeRecord] {
        &self.practices[..]
    }
}

impl Touhou {
    pub fn find_running(system: &System) -> Option<Touhou> {
        system
            .processes()
            .iter()
            .map(|(_, process)| process)
            .find_map(|proc| {
                proc.exe()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .and_then(|exe| {
                        if exe.starts_with("th07") {
                            Some(th07::Touhou7::new_from_process(proc).into())
                        } else {
                            None
                        }
                    })
            })
    }

    pub async fn wait_for_game() -> Self {
        let mut system = System::new();
        loop {
            system.refresh_processes_specifics(ProcessRefreshKind::new());

            if let Some(instance) = Self::find_running(&system) {
                return instance;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}
