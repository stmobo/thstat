use super::location::Location;
use super::process::MemoryAccess;
use crate::memory::traits::*;
use crate::memory::{
    define_state_struct, try_into_or_mem_error, Location as LocationWrapper, MemoryReadError,
    SpellState,
};
use crate::th10::{ShotType as ShotID, SpellId, Stage as StageID, Touhou10};
use crate::types::{Difficulty, ShotPower, ShotType, SpellCard, Stage};

pub type ReadResult<T> = Result<T, MemoryReadError<Touhou10>>;

define_state_struct! {
    PlayerState {
        character: ShotType<Touhou10>,
        lives: u8,
        power: ShotPower<Touhou10>,
        continues: u8,
        score: u32,
        faith: u32,
        extends: u32,
    }
}

impl PlayerState {
    pub fn new(proc: &MemoryAccess) -> ReadResult<Self> {
        let character = match (proc.character()?, proc.character_subtype()?) {
            (0, 0) => ShotType::new(ShotID::ReimuA),
            (0, 1) => ShotType::new(ShotID::ReimuB),
            (0, 2) => ShotType::new(ShotID::ReimuC),
            (1, 0) => ShotType::new(ShotID::MarisaA),
            (1, 1) => ShotType::new(ShotID::MarisaB),
            (1, 2) => ShotType::new(ShotID::MarisaC),
            (0..=1, other) => {
                return Err(MemoryReadError::other_out_of_range(
                    "character subtype",
                    other,
                    0,
                    2,
                ));
            }
            (other, _) => {
                return Err(MemoryReadError::other_out_of_range(
                    "character",
                    other,
                    0,
                    1,
                ));
            }
        };

        Ok(Self {
            character,
            lives: proc.lives()? as u8,
            power: proc
                .power()
                .and_then(try_into_or_mem_error)
                .map(ShotPower::new)?,
            continues: proc.continues_used()? as u8,
            score: proc.score()?,
            faith: proc.faith()?,
            extends: proc.extends()?,
        })
    }
}

impl PlayerData<Touhou10> for PlayerState {
    fn shot(&self) -> ShotType<Touhou10> {
        self.character
    }

    fn power(&self) -> ShotPower<Touhou10> {
        self.power
    }
}

impl LifeStock<Touhou10> for PlayerState {
    fn lives(&self) -> u8 {
        self.lives
    }
}

impl ContinueCount<Touhou10> for PlayerState {
    fn continues_used(&self) -> u8 {
        self.continues
    }
}

impl PlayerScore<Touhou10> for PlayerState {
    fn score(&self) -> u64 {
        self.score as u64
    }
}

define_state_struct! {
    ActiveSpell {
        spell: SpellCard<Touhou10>,
        bonus: Option<u32>
    }
}

impl ActiveSpell {
    pub fn new(proc: &MemoryAccess) -> ReadResult<Option<Self>> {
        let status = proc.active_spell_status()?;
        if (status & 1) != 0 {
            let id = proc.active_spell()?;

            if id != 0 {
                let bonus = if (status & 2) != 0 {
                    let val = proc.active_spell_bonus()?;
                    if val > 0 { Some(val as u32) } else { None }
                } else {
                    None
                };

                let spell = SpellId::new((id + 1) as u16).map(SpellCard::new)?;

                return Ok(Some(ActiveSpell { spell, bonus }));
            }
        }

        Ok(None)
    }
}

define_state_struct! {
    BossState {
        active_spell: Option<ActiveSpell>,
        remaining_lifebars: u8
    }
}

impl BossState {
    pub fn new(proc: &MemoryAccess) -> ReadResult<Self> {
        Ok(Self {
            active_spell: ActiveSpell::new(proc)?,
            remaining_lifebars: proc.boss_lifebars()? as u8,
        })
    }

    pub fn spell(&self) -> Option<SpellCard<Touhou10>> {
        self.active_spell.as_ref().map(ActiveSpell::spell)
    }
}

impl BossData<Touhou10> for BossState {
    fn active_spell(&self) -> Option<SpellState<Touhou10>> {
        self.active_spell
            .map(|active| SpellState::new(active.spell().unwrap(), active.bonus().is_some()))
    }
}

impl BossLifebars<Touhou10> for BossState {
    fn remaining_lifebars(&self) -> u8 {
        self.remaining_lifebars
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
    pub fn new(stage: StageID, proc: &MemoryAccess) -> ReadResult<Self> {
        match proc.game_state()? {
            0 | 4..=5 => Ok(Activity::StageSection),
            2 => Ok(Activity::PostDialogue),
            1 | 3 => Ok(Activity::StageDialogue),
            6..=23 => {
                if !matches!(stage, StageID::Two | StageID::Four)
                    || (proc.game_state_frame()? < 900)
                {
                    BossState::new(proc).map(Self::Midboss)
                } else {
                    Ok(Self::StageSection)
                }
            }
            _ => BossState::new(proc).map(Self::Boss),
        }
    }
}

define_state_struct! {
    StageState {
        stage: Stage<Touhou10>,
        activity: Activity
    }
}

impl StageState {
    pub fn new(proc: &MemoryAccess) -> ReadResult<Self> {
        let stage: Stage<Touhou10> = proc
            .stage()
            .and_then(|id| {
                id.checked_sub(1)
                    .ok_or_else(|| MemoryReadError::other("invalid stage id 0"))
            })
            .and_then(try_into_or_mem_error)
            .map(Stage::new)?;

        Activity::new(stage.unwrap(), proc).map(|activity| Self { stage, activity })
    }
}

impl StageData<Touhou10> for StageState {
    type BossState = BossState;

    fn stage_id(&self) -> Stage<Touhou10> {
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
        difficulty: Difficulty<Touhou10>,
        practice: bool,
        player: PlayerState,
        stage: StageState
    }
}

impl RunState {
    pub fn new(proc: &MemoryAccess) -> ReadResult<Self> {
        let difficulty = proc
            .difficulty()
            .and_then(try_into_or_mem_error)
            .map(Difficulty::new)?;

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

    fn difficulty(&self) -> Difficulty<Touhou10> {
        self.difficulty
    }

    fn player(&self) -> &Self::PlayerState {
        &self.player
    }

    fn stage(&self) -> &Self::StageState {
        &self.stage
    }

    fn is_practice(&self) -> bool {
        self.practice
    }
}

impl ResolveLocation<Touhou10> for RunState {
    fn resolve_location(&self) -> Option<LocationWrapper<Touhou10>> {
        Location::resolve(self).map(LocationWrapper::new)
    }
}

impl HasLocations for Touhou10 {
    type Location = Location;

    fn stage_start_location(stage: Self::StageID) -> Self::Location {
        Location::stage_section(stage)
    }
}

fn read_bgm_id(proc: &MemoryAccess) -> ReadResult<Option<u32>> {
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
    pub fn game_is_active(proc: &MemoryAccess) -> ReadResult<bool> {
        if (0x1000..0x8000_0000).contains(&proc.menu_base_ptr()?) {
            Ok(false)
        } else {
            Ok(!matches!(read_bgm_id(proc)?, Some(2 | 13 | 14 | 17)) && (proc.replay_flag()? != 2))
        }
    }

    pub fn new(proc: &MemoryAccess) -> ReadResult<Self> {
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
