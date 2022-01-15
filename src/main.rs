mod types;

fn main() {
    let input = include_str!("input.json");
    let parse: types::NS2Stats = serde_json::from_str(input).unwrap();

    println!("{parse:?}");
}
