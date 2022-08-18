mod models;
mod plugins;
use actix_web::{post, web, App, HttpServer, Responder};
use futures::{channel::mpsc::Sender, SinkExt};
use models::Bot;
use plugins::*;
use serde::{Deserialize, Serialize};

use crate::models::CQEvent;

#[derive(Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub listen_addr: String,
    pub cq_addr: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            listen_addr: "127.0.0.1:5701".to_string(),
            cq_addr: "127.0.0.1:5700".to_string(),
        }
    }
}

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
    "ok"
}

#[actix_web::main]
async fn main() {
    let cfg: AppConfig = confy::load("config").unwrap();
    let listen_addr = cfg.listen_addr.clone();
    let mut bot = Bot::new(cfg.clone());

    bot.register_plugin(EchoPlugin::new(None));
    bot.register_plugin(QuestionPlugin::new(Some(QuestionPluginConfig {
        sleep_seconds: 30,
    })));
    // bot.register_plugin(ArchivePlugin::new(None));

    HttpServer::new(move || App::new().service(handle_event))
        .bind(listen_addr)
        .unwrap()
        .run()
        .await
        .unwrap();
}
