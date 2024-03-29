use std::collections::HashMap;

use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_repr::{Deserialize_repr, Serialize_repr};

pub type SteamId = i64;
pub type Location = usize;

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
    /// If the building was completely built when this happened.
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    #[serde(deserialize_with = "deserialize_weapons")]
    pub weapons: HashMap<String, Weapon>,
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

fn deserialize_weapons<'de, D>(deserializer: D) -> Result<HashMap<String, Weapon>, D::Error>
where
    D: Deserializer<'de>,
{
    struct WeaponsVisitor {}

    impl<'de> Visitor<'de> for WeaponsVisitor {
        type Value = HashMap<String, Weapon>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a map or an empty array")
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            <[(); 0]>::deserialize(serde::de::value::SeqAccessDeserializer::new(seq)).map(|_| HashMap::new())
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            HashMap::deserialize(serde::de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(WeaponsVisitor {})
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
    /// Number of attacks that hit (including Onos hits).
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

/// Research done during the game.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Research {
    /// Team that owns the research.
    #[serde(rename = "teamNumber")]
    pub team: Team,
    /// Time when this research completed (in seconds).
    pub game_time: f32,
    /// Name of the tech researched.
    pub research_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoundInfo {
    /// Unix time for the round.
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MinimapExtents {
    pub origin: String,
    pub scale: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StartingLocations {
    /// The marine's starting location.
    #[serde(rename = "1")]
    pub marines: Location,
    /// The alien's starting location.
    #[serde(rename = "2")]
    pub aliens: Location,
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
pub enum Event {
    Built,
    Destroyed,
    Placed,
    Recycled,
    Teleported,
}

#[derive(Debug, Deserialize_repr, Serialize_repr, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum Team {
    Marines = 1,
    Aliens = 2,
}

#[derive(Debug, Deserialize_repr, Serialize_repr, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum WinningTeam {
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

#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl<'de> Deserialize<'de> for Position {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
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
                let splits = s.split(' ').collect::<Vec<&str>>();
                if splits.len() != 3 {
                    return Err(serde::de::Error::invalid_length(splits.len(), &self));
                }
                let parse = |split: &str| split.parse().map_err(|_| serde::de::Error::invalid_type(serde::de::Unexpected::Str(s), &self));
                Ok(Position {
                    x: parse(splits[0])?,
                    y: parse(splits[1])?,
                    z: parse(splits[2])?,
                })
            }
        }

        deserializer.deserialize_str(PositonVisitor {})
    }
}

impl Serialize for Position {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{} {} {}", self.x, self.y, self.z))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_deserialize() {
        assert_eq!(
            serde_json::from_str::<Position>("\"1.0 -1.0 0.1\"").expect("Failed to parse position"),
            Position { x: 1.0, y: -1.0, z: 0.1 }
        );
        assert!(serde_json::from_str::<Position>("\"1.0 -1.0\"").is_err());
    }

    #[test]
    fn position_serialize() {
        assert_eq!(
            &*serde_json::to_string(&Position { x: 1.0, y: -1.0, z: 0.1 }).expect("Failed to deserialize position"),
            "\"1 -1 0.1\"" // serde removes trailing zeros
        )
    }
}
