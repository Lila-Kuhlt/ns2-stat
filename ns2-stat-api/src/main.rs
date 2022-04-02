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
struct DateQuery {
    from: Option<u32>,
    to: Option<u32>,
}

impl DateQuery {
    fn slice<'a, T>(&self, data: &'a [T], mut get_date: impl FnMut(&T) -> u32) -> &'a [T] {
        let start = self.from.map(|date| data.partition_point(|x| get_date(x) < date)).unwrap_or(0);
        let end = self.to.map(|date| data.partition_point(|x| get_date(x) <= date)).unwrap_or(data.len());
        &data[start..end]
    }
}

#[get("/stats")]
async fn get_stats(data: Data<AppData>) -> impl Responder {
    serde_json::to_string(&DatedData {
        date: data.newest,
        data: &data.stats,
    })
}

#[get("/stats/continuous")]
async fn get_continuous_stats(data: Data<AppData>, query: Query<DateQuery>) -> impl Responder {
    let game_stats = Games(query.slice(&data.games, |game| game.round_info.round_date).iter())
        .filter_genuine_games()
        .collect::<Vec<_>>();
    let continuous_stats = (0..game_stats.len())
        .map(|i| {
            let stats = NS2Stats::compute(Games(game_stats[..=i].iter().copied()));
            DatedData {
                date: game_stats[i].round_info.round_date,
                data: stats,
            }
        })
        .collect::<Vec<_>>();
    serde_json::to_string(&continuous_stats)
}

#[get("/games")]
async fn get_games(data: Data<AppData>, query: Query<DateQuery>) -> impl Responder {
    let games = query.slice(&data.games, |game| game.round_info.round_date);
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

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(get_stats)
            .service(get_continuous_stats)
            .service(get_games)
    })
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
