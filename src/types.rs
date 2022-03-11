use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// XXX: use https://github.com/dtolnay/serde-repr

pub type SteamId = u32;
/// A Team Number.
/// 0: No Team
/// 1: Marines
/// 2: Aliens
pub type Team = u8;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameStats {
    #[serde(rename = "KillFeed")]
    pub kill_feed: Vec<KillFeed>,
    /// A vector with the location names, the locations in other tables are indices into this vector.
    #[serde(rename = "Locations")]
    pub locations: Vec<String>,
    #[serde(rename = "Research")]
    pub research: Vec<Research>,
    #[serde(rename = "Buildings")]
    pub buildings: Vec<Building>,
    #[serde(rename = "PlayerStats")]
    pub player_stats: HashMap<SteamId, PlayerStat>,
    #[serde(rename = "RoundInfo")]
    pub round_info: RoundInfo,
    #[serde(rename = "ServerInfo")]
    pub server_info: ServerInfo,
    #[serde(rename = "MarineCommStats")]
    pub marine_comm_stats: HashMap<String, MarineCommStat>,
}

/// Building completions, deaths and recycles during the game.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Building {
    /// Team that owns the building.
    #[serde(rename = "teamNumber")]
    pub team: Team,
    /// Time when this building action completed (in seconds).
    #[serde(rename = "gameTime")]
    pub game_time: f64,
    // If the building was completely built when this happened.
    pub built: bool,
    pub location: String,
    /// The building was recycled.
    pub recycled: bool,
    /// The building was destroyed.
    pub destroyed: bool,
    /// Name of the building.
    #[serde(rename = "techId")]
    pub tech_id: String,
    /// How much biomass was lost (only when a hive dies).
    pub biomass: Option<u8>,
    #[serde(rename = "entityId")]
    pub entity_id: Option<i64>,
    pub event: Option<Event>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KillFeed {
    /// Weapon used for the kill.
    #[serde(rename = "killerWeapon")]
    pub killer_weapon: String,
    #[serde(rename = "killerSteamID")]
    pub killer_steam_id: Option<SteamId>,
    /// Location of the killer.
    #[serde(rename = "killerLocation")]
    pub killer_location: Option<usize>,
    /// Map coordinates of the killer.
    #[serde(rename = "killerPosition")]
    pub killer_position: Option<String>,
    #[serde(rename = "killerClass")]
    /// The killer's class.
    pub killer_class: Option<String>,
    /// Location of the killer entity position (grenades/turrets/hydras, etc.).
    #[serde(rename = "doerLocation")]
    pub doer_location: Option<usize>,
    /// Map coordinates for the killer entity position (grenades/turrets/hydras, etc.).
    #[serde(rename = "doerPosition")]
    pub doer_position: Option<String>,
    /// Team that got awarded this kill.
    #[serde(rename = "killerTeamNumber")]
    pub killer_team: Team,
    /// Location of the victim.
    #[serde(rename = "victimLocation")]
    pub victim_location: Option<usize>,
    #[serde(rename = "victimSteamID")]
    pub victim_steam_id: SteamId,
    /// The victim's class.
    #[serde(rename = "victimClass")]
    pub victim_class: String,
    /// Map coordinates for the victim.
    #[serde(rename = "victimPosition")]
    pub victim_position: String,
    /// Game time when the kill happened (in seconds).
    #[serde(rename = "gameTime")]
    pub game_time: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarineCommStat {
    pub medpack: Medpack,
    pub ammopack: Ammopack,
    pub catpack: Catpack,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Medpack {
    /// Number of medpacks picked up by players.
    pub picks: u64,
    /// Number of medpacks that are never picked up.
    pub misses: u64,
    /// Amount of health given to players through medpacks.
    pub refilled: f64,
    /// Number of medpacks dropped directly on players.
    #[serde(rename = "hitsAcc")]
    pub hits_acc: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ammopack {
    /// Number of ammopacks picked up by players.
    pub picks: u64,
    /// Number of ammopacks that are never picked up.
    pub misses: u64,
    /// Amount of bullets given to players through ammopacks.
    pub refilled: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Catpack {
    /// Number of catpacks picked up by players.
    pub picks: u64,
    /// Number of catpacks that are never picked up.
    pub misses: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerStat {
    #[serde(rename = "1")]
    pub marines: PlayerTeamStats,
    #[serde(rename = "2")]
    pub aliens: PlayerTeamStats,
    /// If the player is a rookie.
    #[serde(rename = "isRookie")]
    pub is_rookie: bool,
    pub weapons: Weapons,
    /// Breakdown of classes for the player during the round.
    pub status: Vec<Status>,
    /// Last team the player belonged to.
    #[serde(rename = "lastTeam")]
    pub last_team: Team,
    /// Hive skill for the player.
    #[serde(rename = "hiveSkill")]
    pub hive_skill: u64,
    /// The player name.
    #[serde(rename = "playerName")]
    pub player_name: String,
    #[serde(rename = "commanderSkillOffset")]
    pub commander_skill_offset: Option<i64>,
    #[serde(rename = "commanderSkill")]
    pub commander_skill: Option<u64>,
    #[serde(rename = "playerSkillOffset")]
    pub player_skill_offset: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerTeamStats {
    /// Number of kills.
    pub kills: u64,
    /// Number of deaths.
    pub deaths: u64,
    /// Number of assists.
    pub assists: u64,
    /// Player score for the round.
    pub score: u64,
    /// Time that the player has spent building during the round (in seconds).
    #[serde(rename = "timeBuilding")]
    pub time_building: f64,
    /// Number if attacks that hit (including Onos hits).
    pub hits: u64,
    /// Number of attacks that hit an Onos.
    #[serde(rename = "onosHits")]
    pub onos_hits: u64,
    /// Number of attacks that missed.
    pub misses: u64,
    /// Best killstreak during the round.
    pub killstreak: u64,
    /// Time that the player was on this team for the round (in seconds).
    #[serde(rename = "timePlayed")]
    pub time_played: f64,
    /// Time that the player spent as commander for this team (in seconds).
    #[serde(rename = "commanderTime")]
    pub commander_time: f64,
    /// Player damage.
    #[serde(rename = "playerDamage")]
    pub player_damage: f64,
    /// Structure damage.
    #[serde(rename = "structureDamage")]
    pub structure_damage: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Status {
    /// Name of the cass.
    #[serde(rename = "statusId")]
    pub status_id: String,
    /// Time as this class (in seconds).
    #[serde(rename = "classTime")]
    pub class_time: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Weapon {
    #[serde(rename = "teamNumber")]
    pub team_number: Team,
    pub kills: u64,
    #[serde(rename = "onosHits")]
    pub onos_hits: u64,
    #[serde(rename = "playerDamage")]
    pub player_damage: f64,
    pub hits: u64,
    #[serde(rename = "structureDamage")]
    pub structure_damage: f64,
    pub misses: u64,
}

// Research done during the game.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Research {
    /// Team that owns the research.
    #[serde(rename = "teamNumber")]
    pub team_number: Team,
    // Time when this research completed (in seconds).
    #[serde(rename = "gameTime")]
    pub game_time: f64,
    /// Name of the tech researched.
    #[serde(rename = "researchId")]
    pub research_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoundInfo {
    /// Epoch time for the round.
    #[serde(rename = "roundDate")]
    pub round_date: u64,
    /// The maximum amount of marine players during the round.
    #[serde(rename = "maxPlayers1")]
    pub max_players_marines: u64,
    /// The maximum amount of alien players during the round.
    #[serde(rename = "maxPlayers2")]
    pub max_players_aliens: u64,
    #[serde(rename = "minimapExtents")]
    pub minimap_extents: MinimapExtents,
    /// Starting locations for each team.
    #[serde(rename = "startingLocations")]
    pub starting_locations: StartingLocations,
    /// Team that won the game.
    #[serde(rename = "winningTeam")]
    pub winning_team: Team,
    /// If the game had tournament mode enabled.
    #[serde(rename = "tournamentMode")]
    pub tournament_mode: bool,
    /// Round length (in seconds).
    #[serde(rename = "roundLength")]
    pub round_length: f64,
    /// Name of the map played.
    #[serde(rename = "mapName")]
    pub map_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MinimapExtents {
    pub origin: String,
    pub scale: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StartingLocations {
    #[serde(rename = "1")]
    marines: usize,
    #[serde(rename = "2")]
    aliens: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerInfo {
    /// The mods active on this server.
    pub mods: Vec<Mod>,
    /// Number of slots for this server.
    pub slots: u64,
    /// If the server is rookie only or not.
    #[serde(rename = "rookieOnly")]
    pub rookie_only: bool,
    /// NS2 build number.
    #[serde(rename = "buildNumber")]
    pub build_number: u64,
    /// Server IP.
    pub ip: String,
    /// Server name.
    pub name: String,
    /// Server port.
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mod {
    #[serde(rename = "modId")]
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
