use ns2_stat::NS2Stats;

fn main() -> std::io::Result<()> {
    let stats = NS2Stats::from_dir("test_data")?;
    println!("{:?}", stats.player_names());

    Ok(())
}
