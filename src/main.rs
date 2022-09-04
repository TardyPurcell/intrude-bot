mod bot;
mod models;
mod plugins;
use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use bot::Bot;
use log::{info, warn};
use models::AppConfig;
use plugins::*;
use tokio::sync::mpsc::{self, Sender};

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
    if event.post_type.as_str() == "meta_event" {
        return HttpResponse::NoContent().finish();
    }
    tx.send(event).await.unwrap();
    return HttpResponse::NoContent().finish();
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let cfg_str = std::fs::read_to_string("config.toml").unwrap_or_else(|_| {
        warn!("config.toml not found, using default config");
        let ret = toml::to_string(&AppConfig::default()).unwrap();
        std::fs::write("config.toml", &ret).unwrap();
        ret
    });
    let cfg: AppConfig = toml::from_str(&cfg_str).expect("config.toml is invalid");
    let listen_addr = cfg.bot.listen_addr.clone();
    let (tx, rx) = mpsc::channel(100);
    let mut bot = Bot::new(rx, cfg.bot);

    bot.register_plugin(EchoPlugin::new(cfg.plugins.echo));
    bot.register_plugin(QuestionPlugin::new(cfg.plugins.question));
    bot.register_plugin(ArchivePlugin::new(cfg.plugins.archive));
    bot.register_plugin(SaucePlugin::new(cfg.plugins.sauce));
    bot.register_plugin(RandintPlugin::new(cfg.plugins.randint));
    bot.register_plugin(HOKpPlugin::new(cfg.plugins.hokp));
    bot.register_plugin(RepeatPlugin::new(cfg.plugins.repeat));
    bot.register_plugin(IntegralPlugin::new(cfg.plugins.integral).await);

    let bot_thread = tokio::spawn(async move {
        bot.run().await;
    });
    info!("bot started.");
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
    bot_thread.abort();
}
