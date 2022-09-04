use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::{
    bot::{Bot, BotConfig},
    plugins::{
        ArchivePluginConfig, EchoPluginConfig, HOKpPluginConfig, IntegralPluginConfig,
        QuestionPluginConfig, RandintPluginConfig, RepeatPluginConfig, SaucePluginConfig,
    },
};

#[derive(Default, Deserialize, Serialize)]
pub struct PluginsConfig {
    pub archive: Option<ArchivePluginConfig>,
    pub echo: Option<EchoPluginConfig>,
    pub question: Option<QuestionPluginConfig>,
    pub sauce: Option<SaucePluginConfig>,
    pub randint: Option<RandintPluginConfig>,
    pub hokp: Option<HOKpPluginConfig>,
    pub repeat: Option<RepeatPluginConfig>,
    pub integral: Option<IntegralPluginConfig>,
}

#[derive(Default, Deserialize, Serialize)]
pub struct AppConfig {
    pub bot: BotConfig,
    pub plugins: PluginsConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CQEvent {
    pub time: i64,
    pub self_id: i64,
    pub post_type: String,

    // 消息上报
    pub message_type: Option<String>,

    // 请求上报
    pub request_type: Option<String>,

    // 通知上报
    pub notice_type: Option<String>,

    // 元事件上报
    pub meta_event_type: Option<String>,

    // ...
    pub sub_type: Option<String>,
    pub message_id: Option<i32>,
    pub user_id: Option<i64>,
    pub raw_message: Option<String>,
    pub font: Option<i64>,
    pub group_id: Option<i64>,
    pub operator_id: Option<i64>,
}

#[derive(PartialEq)]
// #[allow(dead_code)]
pub enum PluginSenario {
    Private,
    Group,
    Both,
}

#[async_trait::async_trait]
pub trait Plugin {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn help(&self) -> &'static str;
    fn senario(&self) -> PluginSenario;
    async fn handle(&self, event: CQEvent, bot: &Bot) -> Result<(), Box<dyn Error + Send>>;
}
