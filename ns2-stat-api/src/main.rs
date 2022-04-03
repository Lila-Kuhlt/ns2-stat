use std::{fs, io, net::IpAddr, path::PathBuf};

use actix_web::{
    body::EitherBody,
    error::JsonPayloadError,
    get,
    http::header::ContentType,
    web::{Data, Query},
    App, HttpResponse, HttpServer, Responder,
};
use clap::Parser;
use ns2_stat::{types::GameStats, Games, NS2Stats};
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
    games: Vec<GameStats>,
    stats: NS2Stats,
}

#[derive(Debug, Serialize)]
struct DatedData<T> {
    date: u32,
    data: T,
}

trait Dated<T: Ord> {
    fn date(&self) -> T;
}

impl Dated<u32> for GameStats {
    fn date(&self) -> u32 {
        self.round_info.round_date
    }
}

impl Dated<u32> for &NS2Stats {
    fn date(&self) -> u32 {
        self.latest_game
    }
}

impl<T: Dated<u32>> From<T> for DatedData<T> {
    fn from(data: T) -> Self {
        Self { date: data.date(), data }
    }
}

#[derive(Debug, Deserialize)]
struct DateQuery {
    from: Option<u32>,
    to: Option<u32>,
}

impl DateQuery {
    fn slice<'a, T: Dated<u32>>(&self, data: &'a [T]) -> &'a [T] {
        let start = self.from.map(|date| data.partition_point(|x| x.date() < date)).unwrap_or(0);
        let end = self.to.map(|date| data.partition_point(|x| x.date() <= date)).unwrap_or(data.len());
        &data[start..end]
    }
}

#[get("/stats")]
async fn get_stats(data: Data<AppData>) -> impl Responder {
    json_response(&DatedData::from(&data.stats))
}

#[get("/stats/continuous")]
async fn get_continuous_stats(data: Data<AppData>, query: Query<DateQuery>) -> impl Responder {
    let game_stats = Games(query.slice(&data.games).iter()).filter_genuine_games().collect::<Vec<_>>();
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
    json_response(&query.slice(&data.games))
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
