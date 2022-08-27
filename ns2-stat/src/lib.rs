use std::collections::HashMap;

use serde::Serialize;

use input_types::{GameStats, WinningTeam};

pub mod input_types;

pub trait Merge {
    fn merge(&mut self, other: Self);
}

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

#[derive(Serialize, Default)]
pub struct NS2Stats {
    pub latest_game: u32,
    pub users: HashMap<String, User>,
    pub maps: HashMap<String, Map>,
    pub total_games: u32,
    pub marine_wins: u32,
    pub alien_wins: u32,
}

impl Merge for User {
    fn merge(&mut self, other: Self) {
        self.total_games += other.total_games;
        self.kills += other.kills;
        self.assists += other.assists;
        self.deaths += other.assists;
        self.kd = self.kills as f32 / self.deaths as f32;
        self.kda = (self.kills + self.assists) as f32 / self.deaths as f32;
    }
}

impl Merge for Map {
    fn merge(&mut self, other: Self) {
        self.total_games += other.total_games;
        self.marine_wins += other.marine_wins;
        self.alien_wins += other.alien_wins;
    }
}

impl Merge for NS2Stats {
    fn merge(&mut self, other: Self) {
        self.total_games += other.total_games;
        self.marine_wins += other.marine_wins;
        self.alien_wins += other.marine_wins;

        if self.latest_game < other.latest_game {
            self.latest_game = other.latest_game
        }

        // There should be a better way for this
        for (key, value) in other.users {
            match self.users.get_mut(&key) {
                Some(t) => {
                    t.merge(value);
                }
                None => {
                    self.users.insert(key, value);
                }
            }
        }

        for (key, value) in other.maps {
            match self.maps.get_mut(&key) {
                Some(t) => {
                    t.merge(value);
                }
                None => {
                    self.maps.insert(key, value);
                }
            }
        }
    }
}

// While this method is slower than thre previous, it is more convinient.
// Also this enables iterative building the stats eg. while the server is already
// started, without parsing game again.
impl FromIterator<NS2Stats> for Option<NS2Stats> {
    fn from_iter<T: IntoIterator<Item = NS2Stats>>(iter: T) -> Self {
        iter.into_iter().reduce(|mut acc, item| {
            acc.merge(item);
            acc
        })
    }
}

impl From<GameStats> for NS2Stats {
    fn from(gs: GameStats) -> Self {
        (&gs).into()
    }
}

impl From<&GameStats> for NS2Stats {
    fn from(game: &GameStats) -> Self {
        use std::collections::hash_map::Entry;
        let mut stats = Self::default();

        let Self {
            users,
            maps,
            alien_wins,
            latest_game,
            marine_wins,
            total_games,
        } = &mut stats;

        for player_stat in game.player_stats.values() {
            let user = match users.entry(player_stat.player_name.clone()) {
                Entry::Occupied(o) => o.into_mut(),
                Entry::Vacant(v) => v.insert(User::default()),
            };

            user.total_games += 1;

            for stats in [&player_stat.marines, &player_stat.aliens] {
                user.kills += stats.kills;
                user.assists += stats.assists;
                user.deaths += stats.deaths;
            }
        }

        let map_entry = match maps.entry(game.round_info.map_name.clone()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(Map::default()),
        };

        map_entry.total_games += 1;
        match game.round_info.winning_team {
            WinningTeam::Marines => {
                map_entry.marine_wins += 1;
                *marine_wins += 1;
            }
            WinningTeam::Aliens => {
                map_entry.alien_wins += 1;
                *alien_wins += 1;
            }
            WinningTeam::None => {}
        }

        *latest_game = game.round_info.round_date;
        *total_games += 1;
        stats
    }
}

impl NS2Stats {
    pub fn compute<'a, I: Iterator<Item = &'a GameStats>>(games: Games<'a, I>) -> Option<Self> {
        games.map(NS2Stats::from).collect()
    }
}
