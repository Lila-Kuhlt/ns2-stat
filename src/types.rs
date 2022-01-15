use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct NS2Stats {
    #[serde(rename = "KillFeed")]
    pub kill_feed: Vec<KillFeed>,
    #[serde(rename = "Locations")]
    pub locations: Vec<String>,
    #[serde(rename = "Research")]
    pub research: Vec<Research>,
    #[serde(rename = "Buildings")]
    pub buildings: Vec<Building>,
    #[serde(rename = "PlayerStats")]
    pub player_stats: HashMap<String, PlayerStat>,
    #[serde(rename = "RoundInfo")]
    pub round_info: RoundInfo,
    #[serde(rename = "ServerInfo")]
    pub server_info: ServerInfo,
    #[serde(rename = "MarineCommStats")]
    pub marine_comm_stats: HashMap<String, MarineCommStat>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Building {
    #[serde(rename = "teamNumber")]
    pub team_number: i64,
    #[serde(rename = "techId")]
    pub tech_id: TechId,
    pub built: bool,
    #[serde(rename = "entityId")]
    pub entity_id: i64,
    pub event: Event,
    pub location: String,
    #[serde(rename = "gameTime")]
    pub game_time: f64,
    pub destroyed: bool,
    pub recycled: bool,
    pub biomass: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KillFeed {
    #[serde(rename = "killerWeapon")]
    pub killer_weapon: KillerWeapon,
    #[serde(rename = "killerSteamID")]
    pub killer_steam_id: i64,
    #[serde(rename = "killerLocation")]
    pub killer_location: i64,
    #[serde(rename = "killerPosition")]
    pub killer_position: String,
    #[serde(rename = "killerClass")]
    pub killer_class: KillerClass,
    #[serde(rename = "killerTeamNumber")]
    pub killer_team_number: i64,
    #[serde(rename = "victimLocation")]
    pub victim_location: i64,
    #[serde(rename = "victimSteamID")]
    pub victim_steam_id: i64,
    #[serde(rename = "victimClass")]
    pub victim_class: KillerClass,
    #[serde(rename = "victimPosition")]
    pub victim_position: String,
    #[serde(rename = "gameTime")]
    pub game_time: f64,
    #[serde(rename = "doerPosition")]
    pub doer_position: Option<String>,
    #[serde(rename = "doerLocation")]
    pub doer_location: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarineCommStat {
    pub catpack: Catpack,
    pub medpack: Pack,
    pub ammopack: Pack,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pack {
    pub picks: i64,
    pub misses: i64,
    pub refilled: f64,
    #[serde(rename = "hitsAcc")]
    pub hits_acc: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Catpack {
    pub misses: i64,
    pub picks: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerStat {
    #[serde(rename = "1")]
    pub the_1: HashMap<String, f64>,
    #[serde(rename = "2")]
    pub the_2: HashMap<String, f64>,
    #[serde(rename = "commanderSkillOffset")]
    pub commander_skill_offset: i64,
    #[serde(rename = "commanderSkill")]
    pub commander_skill: i64,
    #[serde(rename = "playerName")]
    pub player_name: String,
    pub status: Vec<Status>,
    #[serde(rename = "isRookie")]
    pub is_rookie: bool,
    #[serde(rename = "playerSkillOffset")]
    pub player_skill_offset: i64,
    #[serde(rename = "lastTeam")]
    pub last_team: i64,
    #[serde(rename = "hiveSkill")]
    pub hive_skill: i64,
    pub weapons: HashMap<String, Weapon>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Status {
    #[serde(rename = "statusId")]
    pub status_id: KillerClass,
    #[serde(rename = "classTime")]
    pub class_time: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Weapon {
    #[serde(rename = "teamNumber")]
    pub team_number: i64,
    pub kills: i64,
    #[serde(rename = "onosHits")]
    pub onos_hits: i64,
    #[serde(rename = "playerDamage")]
    pub player_damage: f64,
    pub hits: i64,
    #[serde(rename = "structureDamage")]
    pub structure_damage: f64,
    pub misses: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Research {
    #[serde(rename = "teamNumber")]
    pub team_number: i64,
    #[serde(rename = "gameTime")]
    pub game_time: f64,
    #[serde(rename = "researchId")]
    pub research_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoundInfo {
    #[serde(rename = "roundDate")]
    pub round_date: i64,
    #[serde(rename = "maxPlayers1")]
    pub max_players1: i64,
    #[serde(rename = "minimapExtents")]
    pub minimap_extents: MinimapExtents,
    #[serde(rename = "startingLocations")]
    pub starting_locations: HashMap<String, i64>,
    #[serde(rename = "winningTeam")]
    pub winning_team: i64,
    #[serde(rename = "tournamentMode")]
    pub tournament_mode: bool,
    #[serde(rename = "maxPlayers2")]
    pub max_players2: i64,
    #[serde(rename = "roundLength")]
    pub round_length: f64,
    #[serde(rename = "mapName")]
    pub map_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinimapExtents {
    pub origin: String,
    pub scale: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerInfo {
    pub mods: Vec<Mod>,
    pub slots: i64,
    #[serde(rename = "rookieOnly")]
    pub rookie_only: bool,
    #[serde(rename = "buildNumber")]
    pub build_number: i64,
    pub ip: String,
    pub name: String,
    pub port: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mod {
    #[serde(rename = "modId")]
    pub mod_id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
    Built,
    Destroyed,
    Placed,
    Recycled,
    Teleported,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TechId {
    #[serde(rename = "ARC")]
    Arc,
    #[serde(rename = "ARCRoboticsFactory")]
    ArcRoboticsFactory,
    Armory,
    ArmsLab,
    CommandStation,
    Crag,
    CragHive,
    Cyst,
    Drifter,
    Extractor,
    Harvester,
    Hive,
    Hydra,
    InfantryPortal,
    InfestedTunnel,
    #[serde(rename = "MAC")]
    Mac,
    Observatory,
    PhaseGate,
    PowerPoint,
    PrototypeLab,
    RoboticsFactory,
    Sentry,
    SentryBattery,
    Shade,
    ShadeHive,
    Shell,
    Shift,
    Spur,
    Tunnel,
    Veil,
    Whip,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum KillerClass {
    Commander,
    Dead,
    Embryo,
    Flamethrower,
    Gorge,
    HeavyMachineGun,
    Onos,
    Rifle,
    Skulk,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum KillerWeapon {
    Bite,
    Flamethrower,
    Gore,
    HeavyMachineGun,
    Rifle,
    Sentry,
    Welder,
    Whip,
}
