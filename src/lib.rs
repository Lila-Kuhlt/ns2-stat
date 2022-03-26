use std::collections::HashMap;

use types::{GameStats, WinningTeam};

pub mod types;

#[derive(Default)]
pub struct User {
    pub kills: u32,
    pub assists: u32,
    pub deaths: u32,
    pub commander_skill: u32,
}

#[derive(Default)]
pub struct Map {
    pub total_games: u32,
    pub marine_wins: u32,
    pub alien_wins: u32,
}

pub struct NS2Stats {
    pub users: HashMap<String, User>,
    pub maps: HashMap<String, Map>,
    pub total_games: u32,
    pub marine_wins: u32,
    pub alien_wins: u32,
}

impl NS2Stats {
    pub fn compute(games: &[GameStats]) -> Self {
        let mut users = HashMap::new();
        let mut maps = HashMap::new();
        let mut marine_wins = 0;
        let mut alien_wins = 0;
        let total_games = games.len() as u32;

        for game in games {
            for player_stat in game.player_stats.values() {
                let user = match users.get_mut(&player_stat.player_name) {
                    Some(user) => user,
                    None => users.entry(player_stat.player_name.clone()).or_insert_with(User::default),
                };

                if let Some(cs) = player_stat.commander_skill {
                    if cs >= user.commander_skill {
                        user.commander_skill = cs;
                    }
                }

                for stats in [&player_stat.marines, &player_stat.aliens] {
                    user.kills += stats.kills;
                    user.assists += stats.assists;
                    user.deaths += stats.deaths;
                }
            }

            let map_entry = match maps.get_mut(&game.round_info.map_name) {
                Some(map) => map,
                None => maps.entry(game.round_info.map_name.clone()).or_insert_with(Map::default),
            };
            map_entry.total_games += 1;
            match game.round_info.winning_team {
                WinningTeam::Marines => {
                    map_entry.marine_wins += 1;
                    marine_wins += 1;
                }
                WinningTeam::Aliens => {
                    map_entry.alien_wins += 1;
                    alien_wins += 1;
                }
                WinningTeam::None => {}
            }
        }

        Self {
            users,
            maps,
            total_games,
            marine_wins,
            alien_wins,
        }
    }
}
