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

pub struct Map {
    pub total_games: u32,
    pub marine_wins: u32,
}

pub struct NS2Stats<'a> {
    pub users: HashMap<&'a str, User>,
    pub maps: HashMap<&'a str, Map>,
    pub total_games: u32,
    pub marine_wins: u32,
}

impl<'a> NS2Stats<'a> {
    pub fn compute(games: &'a [GameStats]) -> Self {
        let mut users = HashMap::new();
        let mut maps = HashMap::new();
        let mut marine_wins = 0;
        let total_games = games.len() as u32;

        for game in games {
            for player_stat in game.player_stats.values() {
                let user = users.entry(&*player_stat.player_name).or_insert_with(|| User::default());

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

            if game.round_info.round_length < 300.0 {
                continue;
            }
            let map_entry = maps.entry(&*game.round_info.map_name).or_insert(Map {
                total_games: 0,
                marine_wins: 0,
            });
            map_entry.total_games += 1;
            if game.round_info.winning_team == WinningTeam::Marines {
                map_entry.marine_wins += 1;
            }

            if game.round_info.winning_team == WinningTeam::Marines {
                marine_wins += 1;
            }
        }

        Self {
            users,
            maps,
            total_games,
            marine_wins,
        }
    }
}
