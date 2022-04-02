use std::{fs, io, net::IpAddr, path::PathBuf};

use actix_web::{
    get,
    web::{Data, Query},
    App, HttpServer, Responder,
};
use clap::Parser;
use ns2_stat::{types::GameStats, Games, NS2Stats};
use serde::{Deserialize, Serialize};

struct AppData {
    games: Vec<GameStats>,
    stats: NS2Stats,
    newest: u32,
}

#[derive(Debug, Serialize)]
struct DatedData<T> {
    date: u32,
    data: T,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct GameQuery {
    limit: usize,
    skip: usize,
}

impl Default for GameQuery {
    fn default() -> Self {
        Self { limit: 10, skip: 0 }
    }
}

#[get("/stats")]
async fn get_stats(data: Data<AppData>) -> impl Responder {
    serde_json::to_string(&DatedData {
        date: data.newest,
        data: &data.stats,
    })
}

#[get("/games")]
async fn get_games(data: Data<AppData>, query: Query<GameQuery>) -> impl Responder {
    let games = data.games.iter().rev().skip(query.skip).take(query.limit).collect::<Vec<_>>();
    serde_json::to_string(&games)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let args = CliArgs::parse();
    let mut games = fs::read_dir(args.data)?
        .map(|e| e.map(|e| e.path()))
        .map(|p| p.and_then(fs::read_to_string))
        .map(|s| s.and_then(|o| serde_json::from_str::<GameStats>(&o).map_err(|e| io::Error::new(io::ErrorKind::Other, e))))
        .collect::<io::Result<Vec<_>>>()?;

    games.sort_by_key(|game| game.round_info.round_date);

    let data = Data::new(AppData {
        newest: games.iter().map(|game| game.round_info.round_date).max().unwrap_or_default(),
        stats: NS2Stats::compute(Games(games.iter()).filter_genuine_games()),
        games,
    });

    HttpServer::new(move || App::new().app_data(data.clone()).service(get_stats).service(get_games))
        .bind((args.address, args.port))?
        .run()
        .await
}

#[derive(Debug, Parser)]
struct CliArgs {
    data: PathBuf,
    #[clap(long, default_value = "127.0.0.1")]
    address: IpAddr,
    #[clap(long, short, default_value = "8080")]
    port: u16,
}
