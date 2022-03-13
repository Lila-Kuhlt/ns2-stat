use ns2_stat::{NS2Stats, User, Map};

fn main() -> std::io::Result<()> {
    let stats = NS2Stats::from_dir("test_data")?;

    let mut users = stats.users.into_iter().collect::<Vec<_>>();
    users.sort_by_key(|(_, user)| ((user.kills as f32 / user.deaths as f32) * 100f32) as u32);
    println!("NAME\t\tKILLS\tDEATHS\tKD");
    for (name, User { kills, deaths, .. }) in users.into_iter().rev() {
        if kills <= 50 || deaths <= 50 {
            continue;
        }
        let kd = kills as f32 / deaths as f32;
        println!("{name}\t{kills}\t{deaths}\t{kd:.2}");
    }

    println!("\n\n\n");

    let marine_wr = stats.marine_wins as f32 * 100f32 / stats.total_games as f32;
    println!("MARINE WR: {marine_wr:.2}%");

    println!("MAP\t\tMARINE WR\tTOTAL ROUNDS");
    let mut kvp = stats.maps.into_iter().collect::<Vec<_>>();
    kvp.sort_by_key(|(_, Map { total_games: r, marine_wins: w })| ((*w as f32 / *r as f32) * 100f32) as u32);
    for (map, Map { total_games, marine_wins }) in kvp.into_iter().rev() {
        let marine_wr = marine_wins as f32 * 100f32 / total_games as f32;
        println!("{map}\t{marine_wr:.2}%\t\t{total_games} rounds");
    }

    let total_games = stats.total_games;
    println!("TOTAL GAMES: {total_games}");

    Ok(())
}
