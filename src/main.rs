use std::collections::HashMap;

use ns2_stat::{types::SteamId, NS2Stats};

#[derive(Default)]
struct User {
    id: SteamId,
    name: String,
    kills: u32,
    deaths: u32,
    commander_skill: u32,
}

impl User {
    fn new(id: SteamId, name: String) -> Self {
        User {
            id,
            name,
            ..Default::default()
        }
    }
}

fn main() -> std::io::Result<()> {
    let stats = NS2Stats::from_dir("test_data")?;

    let mut cache: HashMap<SteamId, User> = HashMap::new();
    for game in &stats.games {
        for (id, player_stats) in &game.player_stats {
            let user = cache.entry(*id).or_insert_with(|| User::new(*id, player_stats.player_name.to_owned()));

            if let Some(cs) = player_stats.commander_skill {
                if cs >= user.commander_skill as i64 {
                    user.commander_skill = cs as u32;
                }
            }

            for stats in vec![&player_stats.marines, &player_stats.marines] {
                user.kills += (*stats.get("kills").unwrap_or(&0f64)) as u32;
                user.deaths += (*stats.get("deaths").unwrap_or(&0f64)) as u32;
            }
        }
    }
    let mut users = cache.values().collect::<Vec<_>>();
    users.sort_by_key(|a| ((a.kills as f32 / a.deaths as f32) * 100f32) as u32);
    println!("NAME\tKILLS\tDEATHS\tKD\tCOM-SKILL");
    for User { name, kills, deaths, .. } in users.iter().rev() {
        if *kills <= 100 || *deaths <= 100 {
            continue;
        }
        let kd = *kills as f32 / *deaths as f32;
        println!("{name}\t{kills}\t{deaths}\t{kd:.2}");
    }

    println!("\n\n\n");

    let marine_wr = stats.games.iter().filter(|game| game.round_info.winning_team == 1).count() as f32 * 100f32 / stats.games.len() as f32;
    println!("MARINE WR: {marine_wr:.2}%");

    let mut cache: HashMap<String, (u32, u32)> = HashMap::new();

    for game in &stats.games {
        if game.round_info.round_length < 300f64 {
            continue;
        }
        let marines_won = game.round_info.winning_team == 1;
        let map = game.round_info.map_name.to_owned();
        let entry = cache.entry(map).or_insert((0, 0));

        entry.0 += 1;
        if marines_won {
            entry.1 += 1;
        }
    }

    println!("MAP\t\tMARINE WR\tTOTAL ROUNDS");
    let mut kvp = cache.iter().collect::<Vec<_>>();
    kvp.sort_by_key(|(_, (r, w))| ((*w as f32 / *r as f32) * 100f32) as u32);
    for (map, (rounds, marines_won)) in kvp.iter().rev() {
        let marine_wr = *marines_won as f32 * 100f32 / *rounds as f32;
        println!("{map}\t{marine_wr:.2}%\t\t{rounds} rounds");
    }

    let total_games = stats.games.len();
    println!("TOTAL GAMES: {total_games}");

    Ok(())
}
