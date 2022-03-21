use std::collections::HashMap;
use std::path::Path;
use std::{fs, io};

use serde::de::Visitor;
use serde::Deserialize;
use serde_repr::Deserialize_repr;

pub type SteamId = u32;
pub type Location = usize;

#[derive(Debug, Deserialize, Clone)]
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

impl GameStats {
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let data = fs::read_to_string(path)?;
        let stat = serde_json::from_str(&data).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(stat)
    }

    pub fn from_dir<P: AsRef<Path>>(path: P) -> io::Result<Vec<Self>> {
        let mut stats = Vec::new();
        for entry in fs::read_dir(path)? {
            let path = entry?.path();
            if path.is_file() && path.extension().unwrap_or_default() == "json" {
                stats.push(Self::from_file(path)?)
            }
        }
        Ok(stats)
    }
}

/// Building completions, deaths and recycles during the game.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Building {
    /// Team that owns the building.
    #[serde(rename = "teamNumber")]
    pub team: Team,
    /// Time when this building action completed (in seconds).
    pub game_time: f32,
    // If the building was completely built when this happened.
    pub built: bool,
    pub location: Position,
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

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KillFeed {
    /// Weapon used for the kill.
    pub killer_weapon: String,
    #[serde(rename = "killerSteamID")]
    pub killer_steam_id: Option<SteamId>,
    /// Location of the killer.
    pub killer_location: Option<Location>,
    /// Map coordinates of the killer.
    pub killer_position: Option<Position>,
    /// The killer's class.
    pub killer_class: Option<PlayerClass>,
    /// Location of the killer entity position (grenades/turrets/hydras, etc.).
    pub doer_location: Option<Location>,
    /// Map coordinates for the killer entity position (grenades/turrets/hydras, etc.).
    pub doer_position: Option<Position>,
    /// Team that got awarded this kill.
    #[serde(rename = "killerTeamNumber")]
    pub killer_team: Team,
    /// Location of the victim.
    pub victim_location: Option<Location>,
    #[serde(rename = "victimSteamID")]
    pub victim_steam_id: SteamId,
    /// The victim's class.
    pub victim_class: PlayerClass,
    /// Map coordinates for the victim.
    pub victim_position: Position,
    /// Game time when the kill happened (in seconds).
    pub game_time: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MarineCommStat {
    pub medpack: Medpack,
    pub ammopack: Ammopack,
    pub catpack: Catpack,
}

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
pub struct Ammopack {
    /// Number of ammopacks picked up by players.
    pub picks: u32,
    /// Number of ammopacks that are never picked up.
    pub misses: u32,
    /// Amount of bullets given to players through ammopacks.
    pub refilled: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Catpack {
    /// Number of catpacks picked up by players.
    pub picks: u32,
    /// Number of catpacks that are never picked up.
    pub misses: u32,
}

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// The class.
    pub status_id: PlayerClass,
    /// Time as this class (in seconds).
    pub class_time: f32,
}

#[derive(Debug, Deserialize, Clone)]
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
#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
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
    pub winning_team: WinningTeam,
    /// If the game had tournament mode enabled.
    pub tournament_mode: bool,
    /// Round length (in seconds).
    pub round_length: f32,
    /// Name of the map played.
    pub map_name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MinimapExtents {
    pub origin: String,
    pub scale: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StartingLocations {
    /// The marine's starting location.
    #[serde(rename = "1")]
    pub marines: Location,
    /// The alien's starting location.
    #[serde(rename = "2")]
    pub aliens: Location,
}

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Mod {
    pub mod_id: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum Weapons {
    AnythingArray(Vec<Option<serde_json::Value>>),
    WeaponMap(HashMap<String, Weapon>),
}

#[derive(Debug, Deserialize, Clone)]
pub enum Event {
    Built,
    Destroyed,
    Placed,
    Recycled,
    Teleported,
}

#[derive(Debug, Deserialize_repr, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum Team {
    Marines = 1,
    Aliens = 2,
}

#[derive(Debug, Deserialize_repr, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum WinningTeam {
    None = 0,
    Marines = 1,
    Aliens = 2,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone)]
pub struct Position {
    x: f32,
    y: f32,
    z: f32,
}

impl<'de> Deserialize<'de> for Position {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct PositonVisitor {}

        impl<'de> Visitor<'de> for PositonVisitor {
            type Value = Position;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a 3 element list of f32 seperated by space")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let mut coords = [0f32; 3];
                let splits = s.split(" ").collect::<Vec<&str>>();
                if splits.len() != 3 {
                    return Err(serde::de::Error::invalid_length(splits.len(), &self));
                }

                for (i, s) in splits.iter().enumerate() {
                    coords[i] = s.parse().map_err(|_| serde::de::Error::invalid_type(serde::de::Unexpected::Str(s), &self))?;
                }
                let [x, y, z] = coords;
                Ok(Position { x, y, z })
            }

            fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(&s)
            }
        }

        deserializer.deserialize_str(PositonVisitor {})
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_data_parsable() -> io::Result<()> {
        GameStats::from_dir("test_data").map(|_| ())
    }

    #[test]
    fn position_deserialize() -> serde_json::Result<()> {
        serde_json::from_str::<Position>("\"1.0 -1.0 0.1\"").map(|_| ())
    }
}
