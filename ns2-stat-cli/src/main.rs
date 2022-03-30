mod table;

use std::{fs, io};

use clap::Parser;
use ns2_stat::types::GameStats;
use ns2_stat::{Games, Map, NS2Stats, User};
use rayon::prelude::*;

use table::Alignment;

#[derive(Parser)]
struct CliArgs {
    /// The path for the game data.
    #[clap(default_value = "test_data")]
    data: String,
}

struct UserRow {
    name: String,
    kills: u32,
    assists: u32,
    deaths: u32,
    kd: f32,
    kda: f32,
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
        .filter_map(|(name, User { kills, assists, deaths, kd, kda })| {
            if kills <= 50 || deaths <= 50 {
                None
            } else {
                Some(UserRow { name, kills, assists, deaths, kd, kda })
            }
        })
        .collect::<Vec<_>>();
    users.sort_by_key(|user| -(user.kd * 100f32) as i32);
    table::print_table(
        ["NAME", "KILLS", "ASSISTS", "DEATHS", "KD", "KDA"],
        [Alignment::Left, Alignment::Right, Alignment::Right, Alignment::Right, Alignment::Right, Alignment::Right],
        &users,
        |UserRow { name, kills, assists, deaths, kd, kda }| row!["{name}", "{kills}", "{assists}", "{deaths}", "{kd:.2}", "{kda:.2}"],
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
    kvp.sort_by_key(|map| -map.marine_wr as i32);
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

fn load_data<P: AsRef<std::path::Path>>(data: P) -> io::Result<Vec<GameStats>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(data)? {
        let path = entry?.path();
        if path.is_file() && path.extension().unwrap_or_default() == "json" {
            paths.push(path)
        }
    }

    paths
        .into_par_iter()
        .map(|path| {
            let data = fs::read_to_string(path)?;
            serde_json::from_str(&data).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        })
        .collect()
}

fn main() -> io::Result<()> {
    let args = CliArgs::parse();

    let game_stats = load_data(args.data)?;
    let stats = NS2Stats::compute(Games(game_stats.iter()).filter_genuine_games());
    print_stats(stats);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_parsable() {
        load_data("../test_data").unwrap();
    }

}