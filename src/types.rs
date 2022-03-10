use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

pub type SteamId = u32;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct GameStats {
    pub kill_feed: Vec<KillFeed>,
    /// A vector with the location names, the locations in other tables are indices into this vector.
    pub locations: Vec<String>,
    pub research: Vec<Research>,
    pub buildings: Vec<Building>,
    pub player_stats: HashMap<SteamId, PlayerStat>,
    pub round_info: RoundInfo,
    pub server_info: ServerInfo,
    pub marine_comm_stats: HashMap<String, MarineCommStat>,
}

/// Building completions, deaths and recycles during the game.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Building {
    /// Team that owns the building.
    #[serde(rename = "teamNumber")]
    pub team: Team,
    /// Time when this building action completed (in seconds).
    pub game_time: f32,
    // If the building was completely built when this happened.
    pub built: bool,
    pub location: String,
    /// The building was recycled.
    pub recycled: bool,
    /// The building was destroyed.
    pub destroyed: bool,
    /// Name of the building.
    pub tech_id: String,
    /// How much biomass was lost (only when a hive dies).
    pub biomass: Option<u8>,
    pub entity_id: Option<u32>,
    pub event: Option<Event>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KillFeed {
    /// Weapon used for the kill.
    pub killer_weapon: String,
    #[serde(rename = "killerSteamID")]
    pub killer_steam_id: Option<SteamId>,
    /// Location of the killer.
    pub killer_location: Option<usize>,
    /// Map coordinates of the killer.
    pub killer_position: Option<String>,
    /// The killer's class.
    pub killer_class: Option<PlayerClass>,
    /// Location of the killer entity position (grenades/turrets/hydras, etc.).
    pub doer_location: Option<usize>,
    /// Map coordinates for the killer entity position (grenades/turrets/hydras, etc.).
    pub doer_position: Option<String>,
    /// Team that got awarded this kill.
    #[serde(rename = "killerTeamNumber")]
    pub killer_team: Team,
    /// Location of the victim.
    pub victim_location: Option<usize>,
    #[serde(rename = "victimSteamID")]
    pub victim_steam_id: SteamId,
    /// The victim's class.
    pub victim_class: PlayerClass,
    /// Map coordinates for the victim.
    pub victim_position: String,
    /// Game time when the kill happened (in seconds).
    pub game_time: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarineCommStat {
    pub medpack: Medpack,
    pub ammopack: Ammopack,
    pub catpack: Catpack,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Medpack {
    /// Number of medpacks picked up by players.
    pub picks: u32,
    /// Number of medpacks that are never picked up.
    pub misses: u32,
    /// Amount of health given to players through medpacks.
    pub refilled: f32,
    /// Number of medpacks dropped directly on players.
    pub hits_acc: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ammopack {
    /// Number of ammopacks picked up by players.
    pub picks: u32,
    /// Number of ammopacks that are never picked up.
    pub misses: u32,
    /// Amount of bullets given to players through ammopacks.
    pub refilled: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Catpack {
    /// Number of catpacks picked up by players.
    pub picks: u32,
    /// Number of catpacks that are never picked up.
    pub misses: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStat {
    #[serde(rename = "1")]
    pub marines: PlayerTeamStats,
    #[serde(rename = "2")]
    pub aliens: PlayerTeamStats,
    /// If the player is a rookie.
    pub is_rookie: bool,
    pub weapons: Weapons,
    /// Breakdown of classes for the player during the round.
    pub status: Vec<Status>,
    /// Last team the player belonged to.
    pub last_team: Team,
    /// Hive skill for the player.
    pub hive_skill: u32,
    /// The player name.
    pub player_name: String,
    pub commander_skill_offset: Option<i32>,
    pub commander_skill: Option<u32>,
    pub player_skill_offset: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerTeamStats {
    /// Number of kills.
    pub kills: u32,
    /// Number of deaths.
    pub deaths: u32,
    /// Number of assists.
    pub assists: u32,
    /// Player score for the round.
    pub score: u32,
    /// Time that the player has spent building during the round (in seconds).
    pub time_building: f32,
    /// Number if attacks that hit (including Onos hits).
    pub hits: u32,
    /// Number of attacks that hit an Onos.
    pub onos_hits: u32,
    /// Number of attacks that missed.
    pub misses: u32,
    /// Best killstreak during the round.
    pub killstreak: u32,
    /// Time that the player was on this team for the round (in seconds).
    pub time_played: f32,
    /// Time that the player spent as commander for this team (in seconds).
    pub commander_time: f32,
    /// Player damage.
    pub player_damage: f32,
    /// Structure damage.
    pub structure_damage: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// The class.
    pub status_id: PlayerClass,
    /// Time as this class (in seconds).
    pub class_time: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Weapon {
    #[serde(rename = "teamNumber")]
    pub team: Team,
    pub kills: u32,
    pub onos_hits: u32,
    pub player_damage: f32,
    pub hits: u32,
    pub structure_damage: f32,
    pub misses: u32,
}

// Research done during the game.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Research {
    /// Team that owns the research.
    #[serde(rename = "teamNumber")]
    pub team: Team,
    // Time when this research completed (in seconds).
    pub game_time: f32,
    /// Name of the tech researched.
    pub research_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoundInfo {
    /// Epoch time for the round.
    pub round_date: u32,
    /// The maximum amount of marine players during the round.
    #[serde(rename = "maxPlayers1")]
    pub max_players_marines: u32,
    /// The maximum amount of alien players during the round.
    #[serde(rename = "maxPlayers2")]
    pub max_players_aliens: u32,
    pub minimap_extents: MinimapExtents,
    /// Starting locations for each team.
    pub starting_locations: StartingLocations,
    /// Team that won the game.
    pub winning_team: Team,
    /// If the game had tournament mode enabled.
    pub tournament_mode: bool,
    /// Round length (in seconds).
    pub round_length: f32,
    /// Name of the map played.
    pub map_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MinimapExtents {
    pub origin: String,
    pub scale: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StartingLocations {
    /// The marine's starting location.
    #[serde(rename = "1")]
    marines: usize,
    /// The alien's starting location.
    #[serde(rename = "2")]
    aliens: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    /// The mods active on this server.
    pub mods: Vec<Mod>,
    /// Number of slots for this server.
    pub slots: u32,
    /// If the server is rookie only or not.
    pub rookie_only: bool,
    /// NS2 build number.
    pub build_number: u32,
    /// Server IP.
    pub ip: String,
    /// Server name.
    pub name: String,
    /// Server port.
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Mod {
    pub mod_id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Weapons {
    AnythingArray(Vec<Option<serde_json::Value>>),
    WeaponMap(HashMap<String, Weapon>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Event {
    Built,
    Destroyed,
    Placed,
    Recycled,
    Teleported,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum Team {
    None = 0,
    Marines = 1,
    Aliens = 2,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerClass {
    CommandStation,
    Commander,
    Dead,
    DeathTrigger,
    Embryo,
    Exo,
    Fade,
    FadeEgg,
    Flamethrower,
    Gorge,
    GorgeEgg,
    GrenadeLauncher,
    HeavyMachineGun,
    Lerk,
    LerkEgg,
    Mine,
    Onos,
    OnosEgg,
    Rifle,
    Sentry,
    Shotgun,
    Skulk,
    Void,
}
