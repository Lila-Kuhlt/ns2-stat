use std::collections::HashMap;

use ns2_stat::{types::SteamId, NS2Stats};

fn main() -> std::io::Result<()> {
    let stats = NS2Stats::from_dir("test_data")?;

    let mut cache: HashMap<SteamId, (u32, u32, String)> = HashMap::new();
    for (id, stat) in stats.games.iter().map(|game| game.player_stats.iter()).flatten() {
        let mut entry = cache.entry(*id).or_insert((0, 0, stat.player_name.clone()));
        let (k, d) = stats.kd(*id);
        entry.0 += k;
        entry.1 += d;
    }
    let mut vec: Vec<_> = cache.values().filter(|(k, _, _)| *k >= 15).collect();
    vec.sort_by_key(|(k, d, _)| ((*k as f32 / *d as f32) * 100.) as u32);

    for (k, d, name) in vec.iter().rev() {
        println!("{}: {:.2} {k} {d}", name, *k as f32 / *d as f32);
    }
    Ok(())
}
