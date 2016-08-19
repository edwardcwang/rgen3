//! Library for manipulating Pokémon Gen3 (Fire Red/Leaf Green, Ruby/Emerald/Sapphire) save data.

#![feature(associated_consts, question_mark)]

#[macro_use]
extern crate log;
extern crate byteorder;
extern crate rgen3_string;

mod util {
    mod lower_upper;
    pub use self::lower_upper::LowerUpper;
}
mod rw;

use std::{io, fmt};
use std::fs::File;
use std::error::Error;
use std::path::Path;
use util::LowerUpper;

const UNKNOWN_SAVE_FOOTER_SIZE: usize = 16384;

/// Pokémon Gen3 save data.
pub struct Save {
    blocks: [SaveBlock; 2],
    unknown: [u8; UNKNOWN_SAVE_FOOTER_SIZE],
    most_recent_index: usize,
}

impl fmt::Debug for Save {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.blocks.fmt(f)
    }
}

impl Save {
    /// Load the save data from a file at the provided path.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<Error>> {
        let mut file = File::open(path)?;
        Save::read(&mut file)
    }
    /// Save the save data to a file at the provided path.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), io::Error> {
        let mut file = File::create(path)?;
        self.write(&mut file)
    }
    pub fn sections(&mut self) -> SaveSections {
        let block = &mut self.blocks[self.most_recent_index];
        let team_and_items = if let SectionData::TeamAndItems(ref mut data) =
                                    block.sections[block.team_and_items_index].data {
            data
        } else {
            panic!("Unexpected section data. Expected TeamAndItems");
        };
        let trainer_info = if let SectionData::TrainerInfo(ref mut data) =
                                  block.sections[block.trainer_info_index].data {
            data
        } else {
            panic!("Unexpected section data. Expected TrainerInfo");
        };
        SaveSections {
            team: &mut team_and_items.team,
            trainer: trainer_info,
        }
    }
}

pub struct SaveSections<'a> {
    pub trainer: &'a mut TrainerInfo,
    pub team: &'a mut Vec<Pokemon>,
}

#[derive(Debug)]
struct SaveBlock {
    sections: [Section; 14],
    trainer_info_index: usize,
    team_and_items_index: usize,
    // Does not exist yet, meaning the game has only been saved once, and this
    // block hasn't been written over yet.
    nonexistent: bool,
}

enum SectionData {
    Unimplemented {
        raw: [u8; DATA_SIZE as usize],
        id: u16,
        cksum: u16,
    },
    TrainerInfo(TrainerInfo),
    TeamAndItems(TeamAndItems),
}

impl fmt::Debug for SectionData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SectionData::Unimplemented { .. } => f.write_str("<Unimplemented section>"),
            SectionData::TrainerInfo(ref data) => data.fmt(f),
            SectionData::TeamAndItems(ref data) => data.fmt(f),
        }
    }
}

#[derive(Debug)]
struct Section {
    data: SectionData,
    unknown_1: u32,
    save_idx: u32,
}

const DATA_SIZE: i64 = 0xFF4;

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
enum Gender {
    Male = 0,
    Female = 1,
}

impl fmt::Display for Gender {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match *self {
            Gender::Male => "Male",
            Gender::Female => "Female",
        };
        f.write_str(str)
    }
}

#[derive(Debug)]
struct Time {
    hours: u16,
    minutes: u8,
    seconds: u8,
    frames: u8,
}

const TRAINER_INFO_UNKNOWN_3_SIZE: usize = 0x00AC - (0x0013 + 3);
const RS_EM_PLAYERINFO_TRAILING_DATA_SIZE: usize = DATA_SIZE as usize - (0x0AC + 4);
const FRLG_PLAYERINFO_UNKNOWN_CHUNK_SIZE: usize = 0x0AF8 - (0x00AC + 4);
const FRLG_PLAYERINFO_TRAILING_DATA_SIZE: usize = DATA_SIZE as usize - (0x0AF8 + 4);

enum Game {
    RubyOrSapphire { trailing_data: [u8; RS_EM_PLAYERINFO_TRAILING_DATA_SIZE], },
    FireredOrLeafgreen {
        unknown: [u8; FRLG_PLAYERINFO_UNKNOWN_CHUNK_SIZE],
        security_key: u32,
        trailing_data: [u8; FRLG_PLAYERINFO_TRAILING_DATA_SIZE],
    },
    Emerald {
        security_key: u32,
        trailing_data: [u8; RS_EM_PLAYERINFO_TRAILING_DATA_SIZE],
    },
}

#[derive(Clone, Copy)]
enum GameType {
    RubyOrSapphire,
    FireredOrLeafgreen,
    Emerald,
}

impl<'a> From<&'a Game> for GameType {
    fn from(src: &Game) -> Self {
        match *src {
            Game::RubyOrSapphire { .. } => GameType::RubyOrSapphire,
            Game::FireredOrLeafgreen { .. } => GameType::FireredOrLeafgreen,
            Game::Emerald { .. } => GameType::Emerald,
        }
    }
}

impl fmt::Debug for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Game::RubyOrSapphire { .. } => write!(f, "Ruby/Sapphire"),
            Game::FireredOrLeafgreen { security_key, .. } => {
                write!(f, "Fire Red/Leaf Green (security key: {})", security_key)
            }
            Game::Emerald { security_key, .. } => {
                write!(f, "Emerald (security key: {})", security_key)
            }
        }
    }
}

const TRAINER_NAME_LEN: usize = 7;

pub struct TrainerInfo {
    pub name: TrainerName,
    unknown_1: u8,
    gender: Gender,
    unknown_2: u8,
    public_id: u16,
    secret_id: u16,
    time_played: Time,
    options_data: [u8; 3],
    unknown_3: [u8; TRAINER_INFO_UNKNOWN_3_SIZE],
    game: Game,
}

impl TrainerInfo {
    pub fn full_id(&self) -> u32 {
        u32::merge(self.public_id, self.secret_id)
    }
}

impl fmt::Debug for TrainerInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("== Trainer Info ==\n")?;
        writeln!(f, "name: {:?}", self.name)?;
        writeln!(f, "gender: {:?}", self.gender)?;
        writeln!(f,
                 "id: public({:?}) secret({:?}) combined({:?})",
                 self.public_id,
                 self.secret_id,
                 u32::merge(self.public_id, self.secret_id))?;
        writeln!(f, "Time played: {:?}", self.time_played)?;
        writeln!(f, "GAME ID: {:?}", self.game)
    }
}

const EM_RU_SA_TEAMANDITEMS_UNK_LEN: usize = 0x0234;
const FR_LG_TEAMANDITEMS_UNK_LEN: usize = 0x0034;
const TEAMANDITEMS_POKE_LEN: usize = 600;
const EM_RU_SA_TEAMANDITEMS_REM_LEN: usize =
    DATA_SIZE as usize - (EM_RU_SA_TEAMANDITEMS_UNK_LEN + TEAMANDITEMS_POKE_LEN + 4);
const FR_LG_TEAMANDITEMS_REM_LEN: usize = DATA_SIZE as usize -
                                          (FR_LG_TEAMANDITEMS_UNK_LEN + TEAMANDITEMS_POKE_LEN + 4);

enum TeamAndItemsUnknown {
    EmeraldOrRubyOrSapphire([u8; EM_RU_SA_TEAMANDITEMS_UNK_LEN]),
    FireRedOrLeafGreen([u8; FR_LG_TEAMANDITEMS_UNK_LEN]),
}

enum TeamAndItemsRemaining {
    EmeraldOrRubyOrSapphire([u8; EM_RU_SA_TEAMANDITEMS_REM_LEN]),
    FireredOrLeafgreen([u8; FR_LG_TEAMANDITEMS_REM_LEN]),
}

struct TeamAndItems {
    unknown: TeamAndItemsUnknown,
    team: Vec<Pokemon>,
    orig_pokemon_data: [u8; TEAMANDITEMS_POKE_LEN],
    remaining_data: TeamAndItemsRemaining,
}

impl fmt::Debug for TeamAndItems {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("=== Team And Items ===\nTeam listing:\n")?;
        for pokemon in &self.team {
            pokemon.fmt(f)?
        }
        Ok(())
    }
}

const POKEMON_NICK_LEN: usize = 10;

#[derive(Default, Clone, Copy)]
pub struct PokemonNick(pub [u8; POKEMON_NICK_LEN]);
#[derive(Default, Clone, Copy)]
pub struct TrainerName(pub [u8; TRAINER_NAME_LEN]);

macro_rules! debug_impl {
    ($target:ident) => {
        impl fmt::Debug for $target {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", rgen3_string::decode_string(&self.0[..]))
            }
        }
    }
}

debug_impl!(PokemonNick);
debug_impl!(TrainerName);

/// A Pokemon.
#[derive(Debug, Default)]
#[allow(missing_docs)]
pub struct Pokemon {
    pub personality: u32,
    pub ot_id: u32,
    pub nickname: PokemonNick,
    language: u16,
    pub ot_name: TrainerName,
    markings: u8,
    checksum: u16,
    unknown_1: u16,
    pub data: PokemonData,
    status_condition: u32,
    pub level: u8,
    pokerus_remaining: u8,
    pub current_hp: u16,
    pub total_hp: u16,
    pub attack: u16,
    pub defense: u16,
    pub speed: u16,
    pub sp_attack: u16,
    pub sp_defense: u16,
}

#[derive(Debug,Default)]
pub struct PokemonData {
    pub growth: PokemonGrowth,
    pub attacks: PokemonAttacks,
    evs_and_condition: PokemonEvsAndCondition,
    misc: PokemonMisc,
}

#[derive(Debug,Default)]
pub struct PokemonGrowth {
    pub species: u16,
    item_held: u16,
    pub experience: u32,
    pp_bonuses: u8,
    friendship: u8,
    unknown: u16,
}

#[derive(Debug, Default)]
pub struct PokemonAttacks {
    pub move1: u16,
    pub move2: u16,
    pub move3: u16,
    pub move4: u16,
    pub pp1: u8,
    pub pp2: u8,
    pub pp3: u8,
    pub pp4: u8,
}

#[derive(Debug, Default)]
struct PokemonEvsAndCondition {
    hp_ev: u8,
    attack_ev: u8,
    defense_ev: u8,
    speed_ev: u8,
    special_attack_ev: u8,
    special_defense_ev: u8,
    coolness: u8,
    beauty: u8,
    cuteness: u8,
    smartness: u8,
    toughness: u8,
    feel: u8,
}

#[derive(Debug, Default)]
struct PokemonMisc {
    pokerus_status: u8,
    met_location: u8,
    origins_info: u16,
    ivs_eggs_and_ability: u32,
    ribbons_and_obedience: u32,
}