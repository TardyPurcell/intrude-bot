mod models;
mod plugins;
use actix_web::{post, web, App, HttpServer, Responder};
use models::Bot;
use plugins::*;

use crate::models::CQEvent;

#[post("/")]
async fn handle_event(event: web::Json<CQEvent>, bot: web::Data<Bot>) -> impl Responder {
    let event = event.into_inner();
    match event.post_type.as_str() {
        "message" => bot.handle_message(event).await,
        "request" => bot.handle_request(event).await,
        "notice" => bot.handle_notice(event).await,
        "meta_event" => bot.handle_meta_event(event).await,
        _ => (),
    }
    "ok"
}

#[actix_web::main]
async fn main() {
    let mut bot = Bot::new();
    bot.register_plugin(EchoPlugin);
    bot.register_plugin(QuestionPlugin);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(bot.clone()))
            .service(handle_event)
    })
    .bind("127.0.0.1:5701")
    .unwrap()
    .run()
    .await
    .unwrap();
}
