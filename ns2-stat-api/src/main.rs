mod data;

use std::collections::BTreeMap;
use std::net::{IpAddr, SocketAddr};
use std::ops::Bound;
use std::path::PathBuf;
use std::io;

use actix_web::{
    body::EitherBody,
    error::JsonPayloadError,
    get,
    http::header::ContentType,
    web::{Data, Query},
    App, HttpResponse, HttpServer, Responder,
};
use clap::Parser;
use notify::Watcher;
use ns2_stat::{input_types::GameStats, Games, NS2Stats};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

fn json_response<T: Serialize>(data: &T) -> HttpResponse<EitherBody<String>> {
    match serde_json::to_string(data) {
        Ok(body) => match HttpResponse::Ok().content_type(ContentType::json()).message_body(body) {
            Ok(res) => res.map_into_left_body(),
            Err(err) => HttpResponse::from_error(err).map_into_right_body(),
        },
        Err(err) => HttpResponse::from_error(JsonPayloadError::Serialize(err)).map_into_right_body(),
    }
}

struct AppData {
    games: RwLock<BTreeMap<u32, GameStats>>,
    stats: RwLock<NS2Stats>,
    path: PathBuf,
}

#[derive(Debug, Serialize)]
struct DatedData<T> {
    date: u32,
    data: T,
}

trait Dated {
    fn date(&self) -> u32;
}

impl<T> Dated for DatedData<T> {
    fn date(&self) -> u32 {
        self.date
    }
}

impl Dated for GameStats {
    fn date(&self) -> u32 {
        self.round_info.round_date
    }
}

impl Dated for NS2Stats {
    fn date(&self) -> u32 {
        self.latest_game
    }
}

impl<T: Dated> Dated for &T {
    fn date(&self) -> u32 {
        (*self).date()
    }
}

impl<T: Dated> From<T> for DatedData<T> {
    fn from(data: T) -> Self {
        Self { date: data.date(), data }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct DateQuery {
    from: Option<u32>,
    to: Option<u32>,
}

impl DateQuery {
    fn to_range_bounds(self) -> (Bound<u32>, Bound<u32>) {
        (
            match self.from {
                Some(bound) => Bound::Included(bound),
                None => Bound::Unbounded,
            },
            match self.to {
                Some(bound) => Bound::Included(bound),
                None => Bound::Unbounded,
            },
        )
    }
}

#[get("/stats")]
async fn get_stats(data: Data<AppData>) -> impl Responder {
    json_response(&DatedData::from(&*data.stats.read()))
}

#[get("/stats/continuous")]
async fn get_continuous_stats(data: Data<AppData>, query: Query<DateQuery>) -> impl Responder {
    let games = data.games.read();
    let game_stats = Games(games.range(query.to_range_bounds()).map(|(_, game)| game))
        .genuine()
        .collect::<Vec<_>>();
    let continuous_stats = (0..game_stats.len())
        .map(|i| DatedData {
            date: game_stats[i].round_info.round_date,
            data: NS2Stats::compute(Games(game_stats[..=i].iter().copied())),
        })
        .collect::<Vec<_>>();
    json_response(&continuous_stats)
}

#[get("/games")]
async fn get_games(data: Data<AppData>, query: Query<DateQuery>) -> impl Responder {
    let games = data.games.read();
    json_response(&games.range(query.to_range_bounds()).map(|(_, game)| game).collect::<Vec<_>>())
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let args = CliArgs::parse();
    let games = data::load(&args.data_path)?;

    let data = Data::new(AppData {
        stats: RwLock::new(NS2Stats::compute(Games(games.values()).genuine())),
        games: RwLock::new(games),
        path: args.data_path,
    });

    let watcher_data = data.clone();
    let mut watcher = notify::recommended_watcher(move |res| match res {
        Ok(_) => {
            // reload all data
            println!("reloading data...");
            let games = match data::load(&watcher_data.path) {
                Ok(games) => games,
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    return;
                }
            };
            *watcher_data.stats.write() = NS2Stats::compute(Games(games.values()).genuine());
            *watcher_data.games.write() = games;
        }
        Err(e) => eprintln!("notify error: {:?}", e),
    })
    .unwrap();
    watcher.watch(&data.path, notify::RecursiveMode::NonRecursive).unwrap();

    let addr = SocketAddr::new(args.address, args.port);
    println!("starting server at {}...", addr);
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(get_stats)
            .service(get_continuous_stats)
            .service(get_games)
    })
    .bind(addr)?
    .run()
    .await
}

#[derive(Debug, Parser)]
struct CliArgs {
    /// The path for the game data.
    data_path: PathBuf,
    #[clap(long, default_value = "127.0.0.1")]
    address: IpAddr,
    #[clap(long, short, default_value = "8080")]
    port: u16,
}
