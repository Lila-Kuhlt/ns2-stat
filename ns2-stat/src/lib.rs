use std::collections::{HashMap, HashSet};

use nalgebra::{DMatrix, Dynamic, Norm};
use serde::Serialize;

use types::{GameStats, KillFeed, PlayerClass, PlayerStat, SteamId, WinningTeam};

pub mod types;

/// A wrapper around an `Iterator<Item = &GameStats>`.
#[derive(Clone)]
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
    pub fn filter_genuine_games(self) -> Games<'a, impl Iterator<Item = &'a GameStats>> {
        self.filter_on_length(|length| length >= 300.0).filter_bot_games()
    }

    /// Filter games with a predicate that takes the length of each game.
    pub fn filter_on_length(self, f: impl Fn(f32) -> bool) -> Games<'a, impl Iterator<Item = &'a GameStats>> {
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

        for game in games {
            for player_stat in game.player_stats.values() {
                let user = match users.get_mut(&player_stat.player_name) {
                    Some(user) => user,
                    None => users.entry(player_stat.player_name.clone()).or_insert_with(User::default),
                };

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

            total_games += 1;
        }

        for user in users.values_mut() {
            user.kd = user.kills as f32 / user.deaths as f32;
            user.kda = (user.kills + user.assists) as f32 / user.deaths as f32;
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

impl<'a, I: Iterator<Item = &'a GameStats> + Clone> Games<'a, I> {
    pub fn all_player_stats(self) -> impl Iterator<Item = &'a PlayerStat> {
        self.flat_map(|game| game.player_stats.values())
    }

    pub fn player_stats(self, id: SteamId) -> impl Iterator<Item = &'a PlayerStat> {
        self.flat_map(move |game| game.player_stats.get(&id))
    }

    pub fn player_ids(self) -> HashSet<SteamId> {
        self.flat_map(|game| game.player_stats.keys()).copied().collect::<HashSet<_>>()
    }

    pub fn player_ids_sorted(self) -> Vec<SteamId> {
        let mut ids = self.player_ids().into_iter().collect::<Vec<_>>();
        ids.sort_unstable();
        ids
    }

    pub fn player_name(self, id: SteamId) -> &'a str {
        self.player_stats(id)
            .next()
            .unwrap_or_else(|| panic!("Player with given steam id {id} was not found"))
            .player_name
            .as_str()
    }

    pub fn kill_feed(self) -> impl Iterator<Item = &'a KillFeed> {
        self
            .filter_genuine_games()
            .flat_map(|game| game.kill_feed.iter())
    }

    pub fn complex_kd(self) -> Vec<(String, f32)> {
        let mut kds = HashMap::<(SteamId, SteamId), u32>::new();
        for kill in self.clone().kill_feed() {
            match (kill.killer_steam_id, kill.victim_steam_id, kill.killer_class) {
                (Some(killer_id), victim_id, Some(class)) if class != PlayerClass::Commander => {
                    *kds.entry((killer_id, victim_id)).or_default() += 1;
                }
                _ => (),
            }
        }
        let mut to_remove: Vec<u32> = self
            .clone()
            .player_ids()
            .iter()
            .map(|id| (id, kds.iter().filter_map(|((_, id2), deaths)| (id == id2).then(|| deaths)).sum::<u32>()))
            .filter_map(|(&id, deaths)| (deaths < 100).then(|| id))
            .collect();
        to_remove.push(0);
        for id in dbg!(to_remove) {
            kds.retain(|&(id1, id2), _| id != id1 && id != id2);
        }

        let mut ids: Vec<_> = kds.keys().map(|(id, _)| *id).collect();
        ids.sort_unstable();
        ids.dedup();
        dbg!(&ids);

        //dbg!(&kds);
        let mut scores = HashMap::new();
        for (i, player1) in ids.iter().enumerate() {
            for player2 in &ids[i..] {
                let p1_k = *kds.get(&(*player1, *player2)).unwrap_or(&0);
                let p2_k = *kds.get(&(*player2, *player1)).unwrap_or(&0);
                let kd = p1_k as f32 / p2_k as f32;
                if player1 == player2 {
                    scores.insert((*player1, *player2), 0.);
                } else if kd.is_finite() && p1_k + p2_k > 20 && (1. / kd).is_finite() {
                    scores.insert((*player1, *player2), 1. / kd);
                    scores.insert((*player2, *player1), kd);
                }
            }
        }
        dbg!(scores.len());
        let mut encounter_count: Vec<_> = ids.iter().map(|&id| (id, scores.keys().filter(|(kid, _)| *kid == id).count())).collect();
        encounter_count.sort_by_key(|&(_, count)| count);
        let dimension = dbg!(&encounter_count).last().unwrap_or(&(0, 0)).1;
        let ids: Vec<_> = encounter_count
            .into_iter()
            .filter_map(|(id, count)| (count > dimension - 4).then(|| id))
            .collect();
        //dbg!(&scores);
        dbg!(&ids);
        let mut results = Vec::new();
        for player1 in &ids {
            for player2 in &ids {
                results.push(*scores.get(&(*player1, *player2)).unwrap());
            }
        }
        //dbg!(results.len());
        //dbg!(ids.len());
        let mat = DMatrix::from_iterator(ids.len(), ids.len(), results.clone());
        let eigenvector = Self::vector_iteration(mat, ids.len());
        let mut scores: Vec<_> = ids
            .into_iter()
            .map(|id| self.clone().player_name(id).to_string())
            .zip(eigenvector.iter().cloned())
            .collect();
        scores.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Less));

        println!("\n\n\n\n\n=======");
        for score in scores.iter() {
            if score.1 > 0. {
                println!("{:?}", score);
            }
        }
        println!("\n\n\n\n\n=======");
        scores
    }

    fn vector_iteration(matrix: DMatrix<f32>, dimensions: usize) -> nalgebra::DVector<f32> {
        let mut r = nalgebra::DVector::from_element(dimensions, 1.).normalize();
        for _ in 0..1000 {
            let new_r = (&matrix * &r).normalize();
            if (r - &new_r).norm() < 0.1 {
                return new_r;
            }
            r = new_r;
        }
        unreachable!("No eigenvector has been found");
    }
}
