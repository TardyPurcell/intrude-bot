use regex::Regex;
use reqwest;
use serde::Serialize;

use crate::{
    models::{CQEvent, Plugin},
    AppConfig,
};

#[derive(Clone)]
pub struct EchoPluginConfig;

#[derive(Clone)]
pub struct EchoPlugin {
    _config: EchoPluginConfig,
}

impl EchoPlugin {
    pub fn new(config: Option<EchoPluginConfig>) -> Self {
        EchoPlugin {
            _config: config.unwrap_or(EchoPluginConfig),
        }
    }
    async fn echo(event: CQEvent, config: AppConfig) {
        let cq_addr = config.cq_addr;
        let msg = event.raw_message.as_ref().unwrap();
        let group_id = event.group_id.unwrap();
        let re = Regex::new(r"^(?P<cmd>>echo)\s+(?P<content>.*)$").unwrap();
        if !re.is_match(msg) {
            return;
        }
        let content = re.replace_all(&msg, "$content").to_string();
        reqwest::Client::new()
            .post(format!("http://{cq_addr}/send_group_msg"))
            .json(&Req {
                group_id,
                message: content,
            })
            .send()
            .await
            .unwrap();
    }
}

#[async_trait::async_trait]
impl Plugin for EchoPlugin {
    fn name(&self) -> &'static str {
        "echo"
    }
    fn help(&self) -> &'static str {
        "复读机"
    }
    fn event_type(&self) -> &'static str {
        "message group"
    }
    async fn handle(&self, event: CQEvent, config: AppConfig) {
        Self::echo(event, config).await
    }
}

#[derive(Serialize)]
struct Req {
    group_id: i64,
    message: String,
}
