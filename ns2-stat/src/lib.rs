use std::collections::HashMap;

use serde::Serialize;

use input_types::{GameStats, WinningTeam};

pub mod input_types;

/// A wrapper around an `Iterator<Item = &GameStats>`.
pub struct Games<'a, I: Iterator<Item = &'a GameStats>>(pub I);

impl<'a, I: Iterator<Item = &'a GameStats>> Iterator for Games<'a, I> {
    type Item = &'a GameStats;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a, I: Iterator<Item = &'a GameStats>> Games<'a, I> {
    /// Filter the genuine games. This is done by ignoring games that took under 5 minutes
    /// and games that were likely bot games.
    pub fn genuine(self) -> Games<'a, impl Iterator<Item = &'a GameStats>> {
        self.filter_by_length(|length| length >= 300.0).filter_bot_games()
    }

    /// Filter games with a predicate that takes the length of each game.
    pub fn filter_by_length(self, f: impl Fn(f32) -> bool) -> Games<'a, impl Iterator<Item = &'a GameStats>> {
        Games(self.filter(move |game| f(game.round_info.round_length)))
    }

    /// Ignore games that were likely bot games.
    pub fn filter_bot_games(self) -> Games<'a, impl Iterator<Item = &'a GameStats>> {
        Games(self.filter(move |game| {
            let mut max_marines = 0;
            let mut max_aliens = 0;
            for player in game.player_stats.values() {
                if player.marines.time_played > 0.0 {
                    max_marines += 1;
                }
                if player.aliens.time_played > 0.0 {
                    max_aliens += 1;
                }
            }
            max_marines > 2 && max_aliens > 2
        }))
    }
}

#[derive(Default, Serialize)]
pub struct User {
    pub total_games: u32,
    pub kills: u32,
    pub assists: u32,
    pub deaths: u32,
    pub kd: f32,
    pub kda: f32,
}

#[derive(Default, Serialize)]
pub struct Map {
    pub total_games: u32,
    pub marine_wins: u32,
    pub alien_wins: u32,
}

#[derive(Serialize)]
pub struct NS2Stats {
    pub latest_game: u32,
    pub users: HashMap<String, User>,
    pub maps: HashMap<String, Map>,
    pub total_games: u32,
    pub marine_wins: u32,
    pub alien_wins: u32,
}

impl NS2Stats {
    pub fn compute<'a, I: Iterator<Item = &'a GameStats>>(games: Games<'a, I>) -> Self {
        let mut users = HashMap::new();
        let mut maps = HashMap::new();
        let mut marine_wins = 0;
        let mut alien_wins = 0;
        let mut total_games = 0;
        let mut latest_game = 0;

        for game in games {
            for player_stat in game.player_stats.values() {
                let user = match users.get_mut(&player_stat.player_name) {
                    Some(user) => user,
                    None => users.entry(player_stat.player_name.clone()).or_insert_with(User::default),
                };
                user.total_games += 1;

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

            if game.round_info.round_date > latest_game {
                latest_game = game.round_info.round_date;
            }
            total_games += 1;
        }

        for user in users.values_mut() {
            user.kd = user.kills as f32 / user.deaths as f32;
            user.kda = (user.kills + user.assists) as f32 / user.deaths as f32;
        }

        Self {
            latest_game,
            users,
            maps,
            total_games,
            marine_wins,
            alien_wins,
        }
    }
}
