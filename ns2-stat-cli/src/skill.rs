//! Based on https://moultano.wordpress.com/2014/08/04/a-skill-ranking-system-for-natural-selection-2/.

use std::collections::HashMap;

use ns2_stat::{GameSummary, WinningTeam};

#[derive(Default)]
pub struct Skill {
    pub common: f32,
    pub offset: f32,
    pub commander: f32,
}

impl Skill {
    pub fn marine(&self) -> f32 {
        self.common + self.offset
    }

    pub fn alien(&self) -> f32 {
        self.common - self.offset
    }
}

pub fn compute_skills(games: &[GameSummary], epochs: usize) -> HashMap<String, Skill> {
    let marine_win_rate = games.iter().filter(|game| game.winning_team == WinningTeam::Marines).count() as f32 / games.len() as f32;

    let mut skills: HashMap<String, Skill> = HashMap::new();

    for _ in 0..epochs {
        for game in games {
            // model: log(p / (1 - p)) = (sum_i T_i s_i) / n + (sum_i T_i x_i) / 2 + log(marine_win_rate / (1 - marine_win_rate))
            // we assume that all players played for the whole round

            // the outcome of the game
            let g = match game.winning_team {
                WinningTeam::Marines => 1.0,
                WinningTeam::Aliens => 0.0,
                WinningTeam::None => continue, // skip games without winner
            };

            // the number of players
            let n = (game.marines.players.len() + game.aliens.players.len()) as f32;

            // predict probability of marines winning based on current skills
            let mut skill_sum = 0.0;
            let mut comm_skill_sum = 0.0;
            for player in game.marines.players.keys() {
                let s = skills.entry(player.clone()).or_default();
                if game.marines.is_commander(player) {
                    comm_skill_sum += s.commander;
                } else {
                    skill_sum += s.marine();
                }
            }
            for player in game.aliens.players.keys() {
                let s = skills.entry(player.clone()).or_default();
                if game.aliens.is_commander(player) {
                    comm_skill_sum -= s.commander;
                } else {
                    skill_sum -= s.alien();
                }
            }
            let p = 1.0 / (1.0 + f32::exp(-(skill_sum / n + comm_skill_sum / 2.0)) * (1.0 - marine_win_rate) / marine_win_rate);

            // update skill values
            for player in game.marines.players.keys() {
                let s = skills.get_mut(player).unwrap();
                if game.marines.is_commander(player) {
                    s.commander += (g - p) / 2.0;
                } else {
                    s.common += (g - p) / n;
                    s.offset += (g - p) / n;
                }
            }
            for player in game.aliens.players.keys() {
                let s = skills.get_mut(player).unwrap();
                if game.aliens.is_commander(player) {
                    s.commander -= (g - p) / 2.0;
                } else {
                    s.common -= (g - p) / n;
                    s.offset += (g - p) / n;
                }
            }
        }
    }

    skills
}
