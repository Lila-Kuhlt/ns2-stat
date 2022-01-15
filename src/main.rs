use ns2_stat::NS2Stats;

fn main() -> std::io::Result<()> {
    let stats = NS2Stats::from_dir("test_data")?;
    let toberius = 914508515u32;

    let kills = stats
        .kill_feed()
        .filter(|kf| kf.killer_steam_id == Some(toberius))
        .count();

    println!("{kills}");

    Ok(())
}
