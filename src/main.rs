use ns2_stat::types;

fn main() {
    let input = include_str!("test_data.json");
    let parse: types::NS2Stats = serde_json::from_str(input).unwrap();

    println!("{parse:?}");
}
