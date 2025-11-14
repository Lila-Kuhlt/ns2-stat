mod helpers;
mod skill;
mod table;
mod teams;

use std::fs;
use std::path::PathBuf;

use clap::Parser;
use ns2_stat::input_types::GameStats;
use ns2_stat::{GameIterator, Map, NS2Stats, summarize_game};
use rayon::prelude::*;

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
