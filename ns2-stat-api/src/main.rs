use std::net::{IpAddr, SocketAddr};
use std::sync::RwLock;
use std::{fs, io, path::PathBuf};

use actix_web::post;
use actix_web::web::Json;
use actix_web::{
    body::EitherBody,
    error::JsonPayloadError,
    get,
    http::header::ContentType,
    web::{Data, Query},
    App, HttpResponse, HttpServer, Responder,
};

use clap::Parser;
use ns2_stat::{input_types::GameStats, Games, Merge, NS2Stats};
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
    cli_args: CliArgs,
}

#[derive(Debug, Serialize)]
struct DatedData<T> {
    date: u32,
    data: T,
}

trait Dated {
    fn date(&self) -> u32;
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

#[derive(Debug, Deserialize)]
struct DateQuery {
    from: Option<u32>,
    to: Option<u32>,
}

impl DateQuery {
    fn slice<'a, T: Dated>(&self, data: &'a [T]) -> &'a [T] {
        let start = self.from.map(|date| data.partition_point(|x| x.date() < date)).unwrap_or(0);
        let end = self.to.map(|date| data.partition_point(|x| x.date() <= date)).unwrap_or(data.len());
        &data[start..end]
    }
}

#[get("/stats")]
async fn get_stats(data: Data<RwLock<AppData>>) -> impl Responder {
    let data = data.read().unwrap();
    json_response(&DatedData::from(&data.stats))
}

#[get("/stats/continuous")]
async fn get_continuous_stats(data: Data<RwLock<AppData>>, query: Query<DateQuery>) -> impl Responder {
    let data = data.read().unwrap();
    let game_stats = Games(query.slice(&data.games).iter()).genuine().collect::<Vec<_>>();
    let continuous_stats = (0..game_stats.len())
        .map(|i| DatedData {
            date: game_stats[i].round_info.round_date,
            data: NS2Stats::compute(Games(game_stats[..=i].iter().copied())),
        })
        .collect::<Vec<_>>();
    json_response(&continuous_stats)
}

#[get("/games")]
async fn get_games(data: Data<RwLock<AppData>>, query: Query<DateQuery>) -> impl Responder {
    let data = data.read().unwrap();
    json_response(&query.slice(&data.games))
}

#[post("/post/game")]
async fn post_game(data: Data<RwLock<AppData>>, game: Json<GameStats>) -> impl Responder {
    let res = {
        let mut data = data.write().unwrap(); // Needs better error handling -- esp. with if the RwLock is poisoned
        let game = game.into_inner();
        let stats = NS2Stats::from(&game);
        let res = json_response(&stats);

        data.stats.merge(stats);
        data.games.push(game);
        res
    };

    let data = data.read().unwrap();
    if !data.cli_args.no_copy {
        let game = data.games.last().unwrap(); // we just pushed a game
        let path = data.cli_args.data_path.join(&format!("{}.json", game.date()));
        if path.exists() {
            log::warn!("Tried to write {path:?}, but file already exists -- skipping.");
            return res;
        }
        fs::write(&path, serde_json::to_string_pretty(&game).unwrap()).unwrap();
        log::trace!("Writing {path:?}");
    }

    res
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let cli_args = CliArgs::parse();
    init_logger(&cli_args);

    let mut games = fs::read_dir(&cli_args.data_path)?
        .map(|e| e.map(|e| e.path()))
        .map(|p| p.and_then(fs::read_to_string))
        .map(|s| s.and_then(|o| serde_json::from_str::<GameStats>(&o).map_err(|e| io::Error::new(io::ErrorKind::Other, e))))
        .collect::<io::Result<Vec<_>>>()?;

    games.sort_by_key(|game| game.round_info.round_date);

    let addr = SocketAddr::new(cli_args.address, cli_args.port);
    let data = Data::new(RwLock::new(AppData {
        cli_args,
        stats: NS2Stats::compute(Games(games.iter()).genuine()).expect("No stats found"),
        games,
    }));

    log::info!("starting server at http://{}", addr);
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(post_game)
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

    /// Wether the Webserver should copy new games (e.g. via /post/game) to `data_path`
    #[clap(long, short)]
    no_copy: bool,

    #[clap(long)]
    no_color: bool,
}

fn init_logger(args: &CliArgs) {
    use fern::colors::{Color, ColoredLevelConfig};
    let colors = match args.no_color {
        false => ColoredLevelConfig::new().debug(Color::Magenta).info(Color::Green).error(Color::Red),
        true => ColoredLevelConfig::default(),
    };

    fern::Dispatch::new()
        .chain(std::io::stdout())
        .level_for("ns2_stat_api", log::LevelFilter::Trace)
        .level(log::LevelFilter::Warn)
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}] {}",
                // This will color the log level only, not the whole line. Just a touch.
                colors.color(record.level()),
                message
            ))
        })
        .apply()
        .unwrap();
}
