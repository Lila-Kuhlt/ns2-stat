mod table;

use ns2_stat::types::GameStats;
use ns2_stat::{Games, Map, NS2Stats, User};
use table::Alignment;

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

fn main() -> std::io::Result<()> {
    let game_stats = GameStats::from_dir("test_data")?;
    let stats = NS2Stats::compute(Games(game_stats.iter()).filter_genuine_games());

    let mut users = stats
        .users
        .into_iter()
        .filter_map(|(name, User { kills, assists, deaths, .. })| {
            if kills <= 50 || deaths <= 50 {
                None
            } else {
                let kd = kills as f32 / deaths as f32;
                let kda = (kills + assists) as f32 / deaths as f32;
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

    Ok(())
}
