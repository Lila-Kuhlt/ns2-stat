mod table;

use std::{fs, io};

use clap::Parser;
use ns2_stat_lib::types::GameStats;
use ns2_stat_lib::{Games, Map, NS2Stats, User};
use rayon::prelude::*;
use serde::Serialize;

use table::Alignment;

#[derive(Parser)]
struct CliArgs {
    /// The path for the game data.
    #[clap(default_value = "test_data")]
    data: String,
    /// Write the output to <OUTPUT>.
    #[clap(long)]
    output: Option<String>,
    /// Output the statistics as JSON.
    #[clap(long)]
    json: bool,
    /// Write the continuous stats as JSON to a file.
    #[clap(short, long)]
    continuous: Option<String>,
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

fn run(mut f: impl io::Write, stats: NS2Stats, json: bool) -> io::Result<()> {
    if json {
        let json_data = serde_json::to_string_pretty(&stats)?;
        writeln!(f, "{}", json_data)?;
        return Ok(());
    }

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
        &mut f,
        ["NAME", "KILLS", "ASSISTS", "DEATHS", "KD", "KDA"],
        [Alignment::Left, Alignment::Right, Alignment::Right, Alignment::Right, Alignment::Right, Alignment::Right],
        &users,
        |UserRow { name, kills, assists, deaths, kd, kda }| row!["{name}", "{kills}", "{assists}", "{deaths}", "{kd:.2}", "{kda:.2}"],
    )?;

    writeln!(f, "\n\n")?;

    let marine_wr = stats.marine_wins as f32 * 100f32 / stats.total_games as f32;
    writeln!(f, "MARINE WR: {marine_wr:.2}%")?;

    writeln!(f)?;

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
        &mut f,
        ["MAP", "MARINE WR", "TOTAL ROUNDS"],
        [Alignment::Left, Alignment::Right, Alignment::Right],
        &kvp,
        |MapRow { map, marine_wr, total_games }| row!["{map}", "{marine_wr:.2}%", "{total_games} rounds"],
    )?;

    writeln!(f)?;

    let total_games = stats.total_games;
    writeln!(f, "TOTAL GAMES: {total_games}")?;

    Ok(())
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
    // HACK
    if let Some(path) = args.continuous {
        use io::Write;

        #[derive(Serialize)]
        struct Entry {
            date: u32,
            stats: NS2Stats,
        }

        let mut f = std::fs::File::create(path)?;
        let mut game_stats: Vec<&GameStats> = Games(game_stats.iter()).filter_genuine_games().collect();
        game_stats.sort_by_key(|game| game.round_info.round_date);

        let continuous_stats = (0..game_stats.len())
            .map(|i| {
                let stats = NS2Stats::compute(Games(game_stats[..=i].iter().copied()));
                Entry { date: game_stats[i].round_info.round_date, stats }
            })
            .collect::<Vec<_>>();

        let json_data = serde_json::to_string_pretty(&continuous_stats)?;
        writeln!(f, "{}", json_data)?;

        return Ok(())
    }

    let stats = NS2Stats::compute(Games(game_stats.iter()).filter_genuine_games());

    match args.output {
        Some(path) => {
            let f = std::fs::File::create(path)?;
            run(f, stats, args.json)
        }
        None => run(io::stdout(), stats, args.json),
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
