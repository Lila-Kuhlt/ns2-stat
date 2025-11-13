use std::collections::HashMap;
use std::ops::AddAssign;

use serde::Serialize;

use input_types::{Building, Event, GameStats, PlayerStat, SteamId, Team};

pub mod input_types;

/// An extension trait for `Iterator` that adds functions related to `GameStats`.
pub trait GameIterator<G: AsRef<GameStats>>: Iterator<Item = G> where Self: Sized {
    /// Filter the genuine games. This is done by ignoring games that took under 5 minutes
    /// and games that were likely bot games.
    fn genuine(self) -> impl Iterator<Item = G> {
        self.filter_by_length(|length| length >= 300.0).filter_bot_games()
    }

    /// Filter games with a predicate that takes the length of each game.
    fn filter_by_length(self, f: impl Fn(f32) -> bool) -> impl Iterator<Item = G> {
        self.filter(move |game| f(game.as_ref().round_info.round_length))
    }

    /// Ignore games that were likely bot games.
    fn filter_bot_games(self) -> impl Iterator<Item = G> {
        self.filter(move |game| {
            let mut max_marines = 0;
            let mut max_aliens = 0;
            for player in game.as_ref().player_stats.values() {
                if player.marines.time_played > 0.0 {
                    max_marines += 1;
                }
                if player.aliens.time_played > 0.0 {
                    max_aliens += 1;
                }
            }
            max_marines > 2 && max_aliens > 2
        })
    }
}

impl<G: AsRef<GameStats>, I: Iterator<Item = G>> GameIterator<G> for I {}

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
    pub score: Stat<f32>,
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
        Stat::map([Stat::map([self.games], |[games]| games as f32), self.score], |[games, score]| score / games)
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
    pub fn compute<'a, I: Iterator<Item = &'a GameStats>>(games: I) -> Self {
        use input_types::WinningTeam;

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

                let (team, stats) = if player_stat.marines.time_played > player_stat.aliens.time_played {
                    // player was in marine team
                    if game.round_info.winning_team == WinningTeam::Marines {
                        user.wins.add(Team::Marines, 1);
                    }
                    (Team::Marines, &player_stat.marines)
                } else {
                    // player was in alien team
                    if game.round_info.winning_team == WinningTeam::Aliens {
                        user.wins.add(Team::Aliens, 1);
                    }
                    (Team::Aliens, &player_stat.aliens)
                };
                user.games.add(team, 1);
                user.kills.add(team, stats.kills);
                user.assists.add(team, stats.assists);
                user.deaths.add(team, stats.deaths);
                user.score.add(team, stats.score as f32 / game.round_info.round_length);
                user.hits.add(team, stats.hits);
                user.misses.add(team, stats.misses);
            }
            let marine_commander = get_commander(Team::Marines, &game.player_stats).unwrap_or_default();
            if let Some(user) = users.get_mut(marine_commander) {
                user.commander.add(Team::Marines, 1);
            }
            let alien_commander = get_commander(Team::Aliens, &game.player_stats).unwrap_or_default();
            if let Some(user) = users.get_mut(alien_commander) {
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

#[derive(Debug, Serialize)]
pub struct PlayerSummary {
    pub kills: u32,
    pub assists: u32,
    pub deaths: u32,
    pub score: u32,
    pub hits: u32,
    pub misses: u32,
}

#[derive(Debug, Serialize)]
pub struct TeamSummary {
    pub players: HashMap<String, PlayerSummary>,
    /// The name of the commander, if present.
    pub commander: Option<String>,
    /// The times when the resource tower (RT) amount changed and the amounts it changed to.
    pub rt_graph: Vec<(f32, u32)>,
}

impl TeamSummary {
    pub fn is_commander(&self, player: &str) -> bool {
        self.commander.as_ref().is_some_and(|comm| comm == player)
    }
}

pub enum WinningTeam {
    None,
    Aliens,
    Marines,
}

impl From<input_types::WinningTeam> for WinningTeam {
    fn from(value: input_types::WinningTeam) -> Self {
        match value {
            input_types::WinningTeam::None => WinningTeam::None,
            input_types::WinningTeam::Aliens => WinningTeam::Aliens,
            input_types::WinningTeam::Marines => WinningTeam::Marines,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct GameSummary {
    /// The round date in Unix time.
    pub round_date: u32,
    pub winning_team: WinningTeam,
    /// The round length in seconds.
    pub round_length: f32,
    pub map_name: String,
    pub aliens: TeamSummary,
    pub marines: TeamSummary,
}

pub fn summarize_game(game: &GameStats) -> GameSummary {
    let round_info = &game.round_info;
    let mut aliens = HashMap::new();
    let mut marines = HashMap::new();
    for player_stat in game.player_stats.values() {
        let (team, stats) = if player_stat.marines.time_played > player_stat.aliens.time_played {
            (&mut marines, &player_stat.marines)
        } else {
            (&mut aliens, &player_stat.aliens)
        };
        team.insert(
            player_stat.player_name.clone(),
            PlayerSummary {
                kills: stats.kills,
                assists: stats.assists,
                deaths: stats.deaths,
                score: stats.score,
                hits: stats.hits,
                misses: stats.misses,
            },
        );
    }
    GameSummary {
        round_date: round_info.round_date,
        winning_team: round_info.winning_team.into(),
        round_length: round_info.round_length,
        map_name: round_info.map_name.clone(),
        aliens: TeamSummary {
            players: aliens,
            commander: get_commander(Team::Aliens, &game.player_stats).map(|name| name.to_owned()),
            rt_graph: compute_rt_graph(Team::Aliens, &game.buildings, round_info.round_length),
        },
        marines: TeamSummary {
            players: marines,
            commander: get_commander(Team::Marines, &game.player_stats).map(|name| name.to_owned()),
            rt_graph: compute_rt_graph(Team::Marines, &game.buildings, round_info.round_length),
        },
    }
}

fn compute_rt_graph(team: Team, buildings: &[Building], round_length: f32) -> Vec<(f32, u32)> {
    use Event::*;

    let rt_name = match team {
        Team::Aliens => "Harvester",
        Team::Marines => "Extractor",
    };
    let mut rt_graph = buildings
        .iter()
        .filter(|b| b.team == team && b.built && b.tech_id == rt_name)
        .filter_map(|b| match b.event {
            Some(Built) => Some((b.game_time, true)),
            Some(Destroyed | Recycled) => Some((b.game_time, false)),
            _ => None,
        })
        .scan(0, |rt, (time, add)| {
            if add {
                *rt += 1;
            } else {
                *rt -= 1;
            }
            Some((time, *rt))
        })
        .collect::<Vec<_>>();
    if let Some((_, last_rt)) = rt_graph.last().copied() {
        // add final RT amount
        rt_graph.push((round_length, last_rt));
    }
    rt_graph
}

fn get_commander(team: Team, player_stats: &HashMap<SteamId, PlayerStat>) -> Option<&str> {
    match team {
        Team::Marines => player_stats
            .values()
            .max_by_key(|player_stat| (player_stat.marines.commander_time * 1000.0) as u32),
        Team::Aliens => player_stats
            .values()
            .max_by_key(|player_stat| (player_stat.aliens.commander_time * 1000.0) as u32),
    }
    .map(|player_stat| &*player_stat.player_name)
}
