use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use std::io;

#[get("/")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    HttpServer::new(|| App::new().service(echo)).bind(("127.0.0.1", 8080))?.run().await
}
