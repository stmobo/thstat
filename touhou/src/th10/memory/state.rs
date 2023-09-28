use std::io::{Error as IOError, ErrorKind, Result as IOResult};

use super::location::Location;
use super::process::MemoryAccess;
use crate::memory::traits::*;
use crate::memory::{define_state_struct, try_into_or_io_error, wrap_io_error, SpellState};
use crate::th10::{Difficulty, ShotType, SpellId, Stage, Touhou10};
use crate::types::Gen2Power;
use crate::SpellCard;

define_state_struct! {
    PlayerState {
        character: ShotType,
        lives: u8,
        power: Gen2Power<100>,
        continues: u8,
        score: u32,
        faith: u32,
        extends: u32,
    }
}

impl PlayerState {
    pub fn new(proc: &MemoryAccess) -> IOResult<Self> {
        let character = match (proc.character()?, proc.character_subtype()?) {
            (0, 0) => ShotType::ReimuA,
            (0, 1) => ShotType::ReimuB,
            (0, 2) => ShotType::ReimuC,
            (1, 0) => ShotType::MarisaA,
            (1, 1) => ShotType::MarisaB,
            (1, 2) => ShotType::MarisaC,
            (0..=1, other) => {
                return Err(IOError::new(
                    ErrorKind::InvalidData,
                    format!("invalid character subtype {}", other),
                ));
            }
            (other, _) => {
                return Err(IOError::new(
                    ErrorKind::InvalidData,
                    format!("invalid character {}", other),
                ));
            }
        };

        Ok(Self {
            character,
            lives: proc.lives()? as u8,
            power: proc
                .power()
                .and_then(try_into_or_io_error(ErrorKind::InvalidData))?,
            continues: proc.continues_used()? as u8,
            score: proc.score()?,
            faith: proc.faith()?,
            extends: proc.extends()?,
        })
    }
}

impl PlayerData<Touhou10> for PlayerState {
    fn shot(&self) -> ShotType {
        self.character
    }

    fn lives(&self) -> u8 {
        self.lives
    }

    fn continues_used(&self) -> u8 {
        self.continues
    }

    fn score(&self) -> u64 {
        self.score as u64
    }
}

define_state_struct! {
    ActiveSpell {
        spell: SpellId,
        bonus: Option<u32>
    }
}

impl ActiveSpell {
    pub fn new(proc: &MemoryAccess) -> IOResult<Option<Self>> {
        let id = proc.active_spell()?;
        let cur_bonus = proc.active_spell_bonus()?;

        if (id != 0) || (cur_bonus != 0) {
            let spell =
                SpellId::new((id + 1) as u16).map_err(wrap_io_error(ErrorKind::InvalidData))?;

            Ok(Some(ActiveSpell {
                spell,
                bonus: if cur_bonus > 0 {
                    Some(cur_bonus as u32)
                } else {
                    None
                },
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct BossState(Option<ActiveSpell>);

impl BossState {
    pub fn spell(&self) -> Option<SpellCard<Touhou10>> {
        self.0.as_ref().map(ActiveSpell::spell).map(SpellCard::from)
    }
}

impl BossData<Touhou10> for BossState {
    fn active_spell(&self) -> Option<SpellState<Touhou10>> {
        self.0
            .map(|active| SpellState::new(active.spell(), active.bonus().is_some()))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Activity {
    StageSection,
    StageDialogue,
    PostDialogue,
    Midboss(BossState),
    Boss(BossState),
}

impl Activity {
    pub fn new(stage: Stage, proc: &MemoryAccess) -> IOResult<Self> {
        match proc.game_state()? {
            0 | 4..=5 => Ok(Activity::StageSection),
            2 => Ok(Activity::PostDialogue),
            1 | 3 => Ok(Activity::StageDialogue),
            6..=23 => {
                if !matches!(stage, Stage::Two | Stage::Four) || (proc.game_state_frame()? < 900) {
                    ActiveSpell::new(proc).map(BossState).map(Self::Midboss)
                } else {
                    Ok(Self::StageSection)
                }
            }
            _ => ActiveSpell::new(proc).map(BossState).map(Self::Boss),
        }
    }
}

define_state_struct! {
    StageState {
        stage: Stage,
        activity: Activity
    }
}

impl StageState {
    pub fn new(proc: &MemoryAccess) -> IOResult<Self> {
        let stage = proc
            .stage()
            .and_then(|id| {
                id.checked_sub(1)
                    .ok_or_else(|| IOError::new(ErrorKind::InvalidData, "invalid stage id 0"))
            })
            .and_then(try_into_or_io_error(ErrorKind::InvalidData))?;
        Activity::new(stage, proc).map(|activity| Self { stage, activity })
    }
}

impl StageData<Touhou10> for StageState {
    type BossState = BossState;

    fn stage_id(&self) -> Stage {
        self.stage
    }

    fn active_boss(&self) -> Option<&Self::BossState> {
        match &self.activity {
            Activity::Midboss(state) | Activity::Boss(state) => Some(state),
            _ => None,
        }
    }
}

define_state_struct! {
    RunState {
        difficulty: Difficulty,
        practice: bool,
        player: PlayerState,
        stage: StageState
    }
}

impl RunState {
    pub fn new(proc: &MemoryAccess) -> IOResult<Self> {
        let difficulty = proc
            .difficulty()
            .and_then(try_into_or_io_error(ErrorKind::InvalidData))?;
        Ok(Self {
            difficulty,
            practice: proc.practice_flag()? == 16,
            player: PlayerState::new(proc)?,
            stage: StageState::new(proc)?,
        })
    }
}

impl RunData<Touhou10> for RunState {
    type PlayerState = PlayerState;
    type StageState = StageState;

    fn difficulty(&self) -> <Touhou10 as crate::Game>::DifficultyID {
        self.difficulty
    }

    fn player(&self) -> &Self::PlayerState {
        &self.player
    }

    fn stage(&self) -> &Self::StageState {
        &self.stage
    }
}

impl ResolveLocation<Touhou10> for RunState {
    fn resolve_location(&self) -> Option<Location> {
        Location::resolve(&self.stage)
    }
}

impl HasLocations for Touhou10 {
    type Location = Location;
}

fn parse_bytes_to_u32(src: &[u8]) -> IOResult<u32> {
    std::str::from_utf8(src)
        .map_err(wrap_io_error(ErrorKind::InvalidData))
        .and_then(|s| s.parse().map_err(wrap_io_error(ErrorKind::InvalidData)))
}

fn read_bgm_id(proc: &MemoryAccess) -> IOResult<Option<u32>> {
    // read segment between _ and . apparently...?
    let bgm_filename = proc.bgm_filename()?;

    let idx1 = bgm_filename
        .iter()
        .rposition(|&c| c == b'_')
        .and_then(|idx| idx.checked_add(1));

    let idx2 = bgm_filename
        .iter()
        .rposition(|&c| c == b'.')
        .and_then(|idx| idx.checked_sub(1));

    if let Some((idx1, idx2)) = idx1.zip(idx2) {
        if idx1 < idx2 {
            return Ok(bgm_filename
                .get(idx1..=idx2)
                .and_then(|src| std::str::from_utf8(src).ok())
                .and_then(|str| str.parse().ok()));
        }
    }

    Ok(None)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GameMenu {
    MainMenu,
    GameStart,
    ExtraStart,
    PracticeStart,
    Replays,
    PlayerData,
    MusicRoom,
    Options,
    Unknown(u32),
}

#[derive(Debug, Copy, Clone)]
pub enum GameState {
    TitleScreen,
    InMenu(GameMenu),
    InGame(RunState),
    InReplay(RunState),
    Ending(RunState),
    StaffRoll,
    GameOver(RunState),
}

impl GameState {
    pub fn is_in_game(proc: &MemoryAccess) -> IOResult<bool> {
        if (0x1000..0x8000_0000).contains(&proc.menu_base_ptr()?) {
            Ok(false)
        } else {
            Ok(!matches!(read_bgm_id(proc)?, Some(2 | 13 | 14 | 17)) && (proc.replay_flag()? != 2))
        }
    }

    pub fn new(proc: &MemoryAccess) -> IOResult<Self> {
        let bgm_id = read_bgm_id(proc)?;

        if bgm_id == Some(17) {
            return RunState::new(proc).map(Self::GameOver);
        }

        if (0x1000..0x8000_0000).contains(&proc.menu_base_ptr()?) {
            if proc.submenu_flag()? != 0 {
                Ok(Self::InMenu(match proc.submenu_selection()? {
                    0 => GameMenu::GameStart,
                    1 => GameMenu::ExtraStart,
                    2 => GameMenu::PracticeStart,
                    3 => GameMenu::Replays,
                    4 => GameMenu::PlayerData,
                    5 => GameMenu::MusicRoom,
                    6 => GameMenu::Options,
                    other => GameMenu::Unknown(other),
                }))
            } else {
                Ok(Self::TitleScreen)
            }
        } else {
            match bgm_id {
                Some(2) => Ok(Self::TitleScreen),
                Some(13) => RunState::new(proc).map(Self::Ending),
                Some(14) => Ok(Self::StaffRoll),
                _ => {
                    if proc.replay_flag()? == 2 {
                        RunState::new(proc).map(Self::InReplay)
                    } else {
                        RunState::new(proc).map(Self::InGame)
                    }
                }
            }
        }
    }
}
