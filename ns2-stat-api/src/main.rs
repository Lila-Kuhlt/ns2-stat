use actix_web::{get, web::Data, App, HttpServer, Responder};
use clap::Parser;
use ns2_stat::{types::GameStats, Games, NS2Stats};
use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};

struct AppData {
    games: Vec<GameStats>,
    newest: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DatedResponse<T> {
    date: u32,
    data: T,
}

#[derive(Debug, Parser)]
struct CliArgs {
    data: PathBuf,
    #[clap(long, default_value = "127.0.0.1")]
    address: String,
    #[clap(long, short, default_value = "8080")]
    port: u16,
}

#[get("/player")]
async fn get_player(data: Data<AppData>) -> impl Responder {
    serde_json::to_string(&DatedResponse {
        date: data.newest,
        data: NS2Stats::compute(Games(data.games.iter())),
    })
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let args = CliArgs::parse();
    let games = fs::read_dir(args.data)?
        .map(|e| e.unwrap().path())
        .map(|p| fs::read_to_string(p).unwrap())
        .map(|s| serde_json::from_str::<GameStats>(&s).unwrap())
        .collect::<Vec<_>>();

    let data = Data::new(AppData {
        newest: games.iter().map(|game| game.round_info.round_date).max().unwrap_or(0),
        games,
    });

    HttpServer::new(move || App::new().app_data(data.clone()).service(get_player))
        .bind((args.address, args.port))?
        .run()
        .await
}
