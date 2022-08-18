mod models;
mod plugins;
use actix_web::{post, web, App, HttpServer, Responder};
use models::{AppConfig, Bot};
use plugins::*;

use std::sync::mpsc::Sender;

use crate::models::CQEvent;

#[post("/")]
async fn handle_event(event: web::Json<CQEvent>, tx: web::Data<Sender<CQEvent>>) -> impl Responder {
    let event = event.into_inner();
    // match event.post_type.as_str() {
    //     "message" => bot.handle_message(event).await,
    //     "request" => bot.handle_request(event).await,
    //     "notice" => bot.handle_notice(event).await,
    //     "meta_event" => bot.handle_meta_event(event).await,
    //     _ => (),
    // }
    tx.send(event).unwrap();
    "ok"
}

#[actix_web::main]
async fn main() {
    let cfg: AppConfig = confy::load("config").unwrap();
    let listen_addr = cfg.listen_addr.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    let mut bot = Bot::new(rx, cfg.clone());

    bot.register_plugin(EchoPlugin::new(None));
    bot.register_plugin(QuestionPlugin::new(Some(QuestionPluginConfig {
        sleep_seconds: 30,
    })));
    // bot.register_plugin(ArchivePlugin::new(None));

    std::thread::spawn(|| {
        bot.run();
    });
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tx.clone()))
            .service(handle_event)
    })
    .bind(listen_addr)
    .unwrap()
    .run()
    .await
    .unwrap();
}
