use std::collections::HashMap;
use std::{fs, io};

pub mod types;

#[derive(Default)]
pub struct User {
    pub kills: u64,
    pub deaths: u64,
    pub commander_skill: u64,
}

pub struct Map {
    pub total_games: u64,
    pub marine_wins: u64,
}

pub struct NS2Stats {
    pub users: HashMap<String, User>,
    pub maps: HashMap<String, Map>,
    pub total_games: u64,
    pub marine_wins: u64,
}

impl NS2Stats {
    pub fn from_dir(path: &str) -> io::Result<Self> {
        let games = Self::load_data(path)?;
        let mut users = HashMap::new();
        let mut maps = HashMap::new();
        let mut marine_wins = 0;
        let total_games = games.len() as u64;

        for game in games {
            for player_stat in game.player_stats.into_values() {
                let user = users.entry(player_stat.player_name).or_insert_with(|| User::default());

                if let Some(cs) = player_stat.commander_skill {
                    if cs >= user.commander_skill {
                        user.commander_skill = cs;
                    }
                }

                for stats in [player_stat.marines, player_stat.aliens] {
                    user.kills += stats.kills;
                    user.deaths += stats.deaths;
                }
            }

            if game.round_info.round_length < 300.0 {
                continue;
            }
            let map_entry = maps.entry(game.round_info.map_name).or_insert(Map { total_games: 0, marine_wins: 0 });
            map_entry.total_games += 1;
            if game.round_info.winning_team == 1 {
                map_entry.marine_wins += 1;
            }

            if game.round_info.winning_team == 1 {
                marine_wins += 1;
            }
        }

        Ok(NS2Stats {
            users,
            maps,
            total_games,
            marine_wins,
        })
    }

    fn load_data(path: &str) -> io::Result<Vec<types::GameStats>> {
        let mut stats = Vec::new();
        for entry in fs::read_dir(path)? {
            let path = entry?.path();
            if path.is_file() && path.extension().unwrap_or_default() == "json" {
                let data = fs::read_to_string(path)?;
                let stat = serde_json::from_str(&data).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                stats.push(stat);
            }
        }
        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_parsable() -> io::Result<()> {
        NS2Stats::from_dir("test_data").map(|_| ())
    }
}
