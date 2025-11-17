mod helpers;
mod ml;
mod skill;
mod table;
mod teams;

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use burn::backend::{Autodiff, Wgpu};
use burn::optim::AdamConfig;
use clap::Parser;
use itertools::Itertools;
use ns2_stat::input_types::GameStats;
use ns2_stat::{GameIterator, Map, NS2Stats, Stat, summarize_game};
use rand::prelude::*;
use rayon::prelude::*;

use crate::ml::data::{GameData, GameDataset, GameInput, GameOutput, PlayerData};
use crate::ml::model::ModelConfig;
use crate::ml::train::{infer, train, TrainingConfig};
use crate::table::Alignment;

#[derive(Parser)]
struct CliArgs {
    /// The path for the game data
    #[clap(default_value = "test_data")]
    data_path: PathBuf,
    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Show team suggestions
    Teams {
        players: Vec<String>,
        #[arg(long)]
        marine_com: Option<String>,
        #[arg(long)]
        alien_com: Option<String>,
    },
    /// Compute the skill of each player
    Skills,
    #[command(name = "ml")]
    MachineLearning,
}

struct UserRow {
    name: String,
    kd: f32,
    kda: f32,
    games: u32,
    commander: u32,
    avg_score: f32,
    accuracy: f32,
}

struct MapRow {
    map: String,
    marine_wr: f32,
    total_games: u32,
}

fn print_stats(stats: NS2Stats) {
    let mut users = stats
        .users
        .into_iter()
        .filter_map(|(name, user)| {
            if user.games.total > 2 {
                Some(UserRow {
                    name,
                    kd: user.kd().total,
                    kda: user.kda().total,
                    games: user.games.total,
                    commander: user.commander.total,
                    avg_score: user.average_score().total,
                    accuracy: user.accuracy().total,
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    users.sort_by(|user1, user2| f32::total_cmp(&user1.avg_score, &user2.avg_score).reverse());
    table::print_table(
        ["NAME", "KD", "KDA", "GAMES", "COMMANDER", "AVG SCORE", "ACCURACY"],
        [
            Alignment::Left,
            Alignment::Right,
            Alignment::Right,
            Alignment::Right,
            Alignment::Right,
            Alignment::Right,
            Alignment::Right,
        ],
        &users,
        |UserRow {
             name,
             kd,
             kda,
             games,
             commander,
             avg_score,
             accuracy,
         }| row!["{name}", "{kd:.2}", "{kda:.2}", "{games}", "{commander}", "{avg_score:.2}", "{accuracy:.2}"],
    );

    println!("\n\n");

    let marine_wr = stats.marine_wins as f32 * 100f32 / stats.total_games as f32;
    println!("MARINE WR: {marine_wr:.2}%");

    println!();

    let mut kvp = stats
        .maps
        .into_iter()
        .map(|(map, Map { total_games, marine_wins, .. })| {
            let marine_wr = marine_wins as f32 * 100f32 / total_games as f32;
            MapRow { map, marine_wr, total_games }
        })
        .collect::<Vec<_>>();
    kvp.sort_by(|map1, map2| f32::total_cmp(&map1.marine_wr, &map2.marine_wr).reverse());
    table::print_table(
        ["MAP", "MARINE WR", "TOTAL ROUNDS"],
        [Alignment::Left, Alignment::Right, Alignment::Right],
        &kvp,
        |MapRow { map, marine_wr, total_games }| row!["{map}", "{marine_wr:.2}%", "{total_games} rounds"],
    );

    println!();

    let total_games = stats.total_games;
    println!("TOTAL GAMES: {total_games}");
}

fn load_data<P: AsRef<std::path::Path>>(data: P) -> Result<Vec<GameStats>, String> {
    let data = data.as_ref();
    let mut paths = Vec::new();
    for entry in fs::read_dir(data).map_err(|e| format!("failed to read directory `{}`\n{}", data.display(), e))? {
        let path = entry.map_err(|e| format!("{}", e))?.path();
        if path.is_file() && path.extension().unwrap_or_default() == "json" {
            paths.push(path)
        }
    }

    paths
        .into_par_iter()
        .map(|path| {
            let data = fs::read_to_string(&path).map_err(|e| format!("failed to read `{}`\n{}", path.display(), e))?;
            serde_json::from_str(&data).map_err(|e| format!("failed to parse `{}`\n{}", path.display(), e))
        })
        .collect()
}

fn square(x: f32) -> f32 {
    x * x
}

fn main() {
    let args = CliArgs::parse();

    let game_stats = load_data(args.data_path).unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    });
    let games = game_stats.iter().genuine();
    match args.command {
        Some(Command::Teams {
            players,
            marine_com,
            alien_com,
        }) => teams::suggest_teams(games.map(summarize_game).collect(), players, marine_com, alien_com),
        Some(Command::Skills) => {
            let game_summaries = games.map(summarize_game).collect::<Vec<_>>();
            let mut skills = skill::compute_skills(&game_summaries, 50).into_iter().collect::<Vec<_>>();
            skills.sort_by(|(_, skill1), (_, skill2)| f32::total_cmp(&skill1.common, &skill2.common));
            table::print_table(
                ["NAME", "MARINE SKILL", "ALIEN SKILL", "COMMANDER SKILL"],
                [Alignment::Left, Alignment::Right, Alignment::Right, Alignment::Right],
                &skills,
                |(name, skill)| {
                    let marine_skill = skill.common + skill.offset;
                    let alien_skill = skill.common - skill.offset;
                    let commander_skill = skill.commander;
                    row!["{name}", "{marine_skill:.2}", "{alien_skill:.2}", "{commander_skill:.2}"]
                },
            )
        }
        Some(Command::MachineLearning) => {
            // compute player data
            let stats = NS2Stats::compute(games);
            let mut player_data = HashMap::new();
            for (name, user) in stats.users {
                let id = player_data.len() + 1;
                player_data.insert(name, Stat {
                    total: PlayerData::new(id, user.average_score().total, user.kd().total),
                    marines: PlayerData::new(id, user.average_score().marines, user.kd().marines),
                    aliens: PlayerData::new(id, user.average_score().aliens, user.kd().aliens),
                });
            }

            // normalize player_data (x = (x - mean) / std_dev)
            let mut mean = Stat {
                total: PlayerData::default(),
                marines: PlayerData::default(),
                aliens: PlayerData::default(),
            };
            for player_stats in player_data.values() {
                mean.total.avg_score += player_stats.total.avg_score;
                mean.total.kd += player_stats.total.kd;
                mean.marines.avg_score += player_stats.marines.avg_score;
                mean.marines.kd += player_stats.marines.kd;
                mean.aliens.avg_score += player_stats.aliens.avg_score;
                mean.aliens.kd += player_stats.aliens.kd;
            }
            let len = player_data.len() as f32;
            mean.total.avg_score /= len;
            mean.total.kd /= len;
            mean.marines.avg_score /= len;
            mean.marines.kd /= len;
            mean.aliens.avg_score /= len;
            mean.aliens.kd /= len;
            let mut std_dev = Stat {
                total: PlayerData::default(),
                marines: PlayerData::default(),
                aliens: PlayerData::default(),
            };
            for player_stats in player_data.values() {
                std_dev.total.avg_score += square(player_stats.total.avg_score - mean.total.avg_score);
                std_dev.total.kd += square(player_stats.total.kd - mean.total.kd);
                std_dev.marines.avg_score += square(player_stats.marines.avg_score - mean.marines.avg_score);
                std_dev.marines.kd += square(player_stats.marines.kd - mean.marines.kd);
                std_dev.aliens.avg_score += square(player_stats.aliens.avg_score - mean.aliens.avg_score);
                std_dev.aliens.kd += square(player_stats.aliens.kd - mean.aliens.kd);
            }
            std_dev.total.avg_score = std_dev.total.avg_score.sqrt();
            std_dev.total.kd = std_dev.total.kd.sqrt();
            std_dev.marines.avg_score = std_dev.marines.avg_score.sqrt();
            std_dev.marines.kd = std_dev.marines.kd.sqrt();
            std_dev.aliens.avg_score = std_dev.aliens.avg_score.sqrt();
            std_dev.aliens.kd = std_dev.aliens.kd.sqrt();
            for player_stats in player_data.values_mut() {
                player_stats.total.avg_score = (player_stats.total.avg_score - mean.total.avg_score) / std_dev.total.avg_score;
                player_stats.total.kd = (player_stats.total.kd - mean.total.kd) / std_dev.total.kd;
                player_stats.marines.avg_score = (player_stats.marines.avg_score - mean.marines.avg_score) / std_dev.marines.avg_score;
                player_stats.marines.kd = (player_stats.marines.kd - mean.marines.kd) / std_dev.marines.kd;
                player_stats.aliens.avg_score = (player_stats.aliens.avg_score - mean.aliens.avg_score) / std_dev.aliens.avg_score;
                player_stats.aliens.kd = (player_stats.aliens.kd - mean.aliens.kd) / std_dev.aliens.kd;
            }

            // compute game data
            let mut game_data = Vec::new();
            let mut alien_commander_data = None;
            let mut marine_commander_data = None;
            for game in game_stats.iter().genuine().map(summarize_game) {
                let mut aliens = Vec::new();
                for name in game.aliens.players.keys() {
                    let player = player_data.get(name).unwrap().aliens.clone();
                    if game.aliens.is_commander(name) {
                        alien_commander_data = Some(player);
                    } else {
                        aliens.push(player);
                    }
                }

                let mut marines = Vec::new();
                for name in game.marines.players.keys() {
                    let player = player_data.get(name).unwrap().marines.clone();
                    if game.marines.is_commander(name) {
                        marine_commander_data = Some(player);
                    } else {
                        marines.push(player);
                    }
                }

                let num_aliens = aliens.len();
                let num_marines = marines.len();
                for mut aliens in aliens.into_iter().permutations(num_aliens).take(100) {
                    if let Some(alien_commander) = alien_commander_data.clone() {
                        aliens.insert(0, alien_commander);
                    }
                    for mut marines in marines.clone().into_iter().permutations(num_marines).take(100) {
                        if let Some(marine_commander) = marine_commander_data.clone() {
                            marines.insert(0, marine_commander);
                        }
                        game_data.push(GameData {
                            input: GameInput { aliens: aliens.clone(), marines },
                            output: GameOutput { round_length: game.round_length },
                        });
                    }
                }
            }

            let mut rng = rand::rng();
            game_data.shuffle(&mut rng);
            let test_data = game_data.split_off(2 * game_data.len() / 3);
            let game_dataset = GameDataset::new(game_data.clone());
            let test_dataset = GameDataset::new(test_data.clone());

            type MyBackend = Wgpu;
            type MyAutodiffBackend = Autodiff<MyBackend>;

            let start = std::time::Instant::now();

            let device = Default::default();
            let artifact_dir = "/tmp/ns2";
            train::<MyAutodiffBackend>(
                artifact_dir,
                TrainingConfig::new(ModelConfig::new(8, 8, 16), AdamConfig::new()),
                &device,
                game_dataset,
                test_dataset,
            );

            let end = std::time::Instant::now();
            println!("training took {:?}", end - start);

            for (typ, data) in [("training", game_data), ("validation", test_data)] {
                let start = std::time::Instant::now();

                let mut sum_length_diff = 0.0;
                let mut max_length_diff = 0.0;
                let num_games = data.len();
                for game in data {
                    let output = infer::<MyBackend>(artifact_dir, &device, game.input);
                    let length_diff = f32::abs(output.round_length - game.output.round_length);
                    sum_length_diff += length_diff;
                    if length_diff > max_length_diff {
                        max_length_diff = length_diff;
                    }
                }
                let avg_length_diff = sum_length_diff / num_games as f32;
                println!("{} max length diff: {}", typ, max_length_diff);
                println!("{} avg length diff: {}", typ, avg_length_diff);

                let end = std::time::Instant::now();
                println!("testing {} accuracy took {:?}", typ, end - start);
            }
        }
        None => print_stats(NS2Stats::compute(games)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_parsable() {
        load_data("../test_data").unwrap();
    }
}
