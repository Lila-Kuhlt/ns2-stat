use std::collections::HashMap;
use std::ops::AddAssign;

use serde::Serialize;

use input_types::{GameStats, Team, WinningTeam};

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

// can be used for games, commander, wins, kills, deaths, assists
#[derive(Clone, Copy, Default, Serialize)]
pub struct Stat<T> {
    pub total: T,
    pub marines: T,
    pub aliens: T,
}

impl<T: AddAssign + Copy> Stat<T> {
    fn add(&mut self, team: Team, n: T) {
        self.total += n;
        match team {
            Team::Aliens => self.aliens += n,
            Team::Marines => self.marines += n,
        }
    }
}

impl<T: Copy> Stat<T> {
    fn map<const N: usize, U>(stats: [Stat<T>; N], f: impl Fn([T; N]) -> U) -> Stat<U> {
        Stat {
            total: f(stats.map(|stat| stat.total)),
            marines: f(stats.map(|stat| stat.marines)),
            aliens: f(stats.map(|stat| stat.aliens)),
        }
    }
}

#[derive(Default, Serialize)]
pub struct User {
    /// The number of games played.
    pub games: Stat<u32>,
    /// The number of games played as commander.
    pub commander: Stat<u32>,
    pub wins: Stat<u32>,
    pub kills: Stat<u32>,
    pub assists: Stat<u32>,
    pub deaths: Stat<u32>,
    pub score: Stat<u32>,
    pub hits: Stat<u32>,
    pub misses: Stat<u32>,
}

impl User {
    /// `kills / deaths`
    pub fn kd(&self) -> Stat<f32> {
        Stat::map([self.kills, self.deaths], |[kills, deaths]| kills as f32 / deaths as f32)
    }

    /// `(kills + assists) / deaths`
    pub fn kda(&self) -> Stat<f32> {
        Stat::map([self.kills, self.assists, self.deaths], |[kills, assists, deaths]| (kills + assists) as f32 / deaths as f32)
    }

    pub fn average_score(&self) -> Stat<f32> {
        Stat::map([self.games, self.score], |[games, score]| score as f32 / games as f32)
    }

    pub fn accuracy(&self) -> Stat<f32> {
        Stat::map([self.hits, self.misses], |[hits, misses]| hits as f32 / (hits + misses) as f32)
    }
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
            let mut marine_comm = "";
            let mut marine_comm_time = 0.0;
            let mut alien_comm = "";
            let mut alien_comm_time = 0.0;

            for player_stat in game.player_stats.values() {
                let user = match users.get_mut(&player_stat.player_name) {
                    Some(user) => user,
                    None => users.entry(player_stat.player_name.clone()).or_insert_with(User::default),
                };

                let (team, stats) = if player_stat.marines.time_played > player_stat.aliens.time_played {
                    // player was in marine team
                    if game.round_info.winning_team == WinningTeam::Marines {
                        user.wins.add(Team::Marines, 1);
                    }
                    if player_stat.marines.commander_time > marine_comm_time {
                        marine_comm = &player_stat.player_name;
                        marine_comm_time = player_stat.marines.commander_time;
                    }
                    (Team::Marines, &player_stat.marines)
                } else {
                    // player was in alien team
                    if game.round_info.winning_team == WinningTeam::Aliens {
                        user.wins.add(Team::Aliens, 1);
                    }
                    if player_stat.aliens.commander_time > alien_comm_time {
                        alien_comm = &player_stat.player_name;
                        alien_comm_time = player_stat.aliens.commander_time;
                    }
                    (Team::Aliens, &player_stat.aliens)
                };
                user.games.add(team, 1);
                user.kills.add(team, stats.kills);
                user.assists.add(team, stats.assists);
                user.deaths.add(team, stats.deaths);
                user.score.add(team, stats.score);
                user.hits.add(team, stats.hits);
                user.misses.add(team, stats.misses);
            }
            if let Some(user) = users.get_mut(marine_comm) {
                user.commander.add(Team::Marines, 1);
            }
            if let Some(user) = users.get_mut(alien_comm) {
                user.commander.add(Team::Aliens, 1);
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
