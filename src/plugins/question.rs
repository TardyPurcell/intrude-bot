use regex::Regex;
use reqwest;

use crate::{
    models::{CQEvent, Plugin},
    AppConfig,
};

#[derive(Clone)]
pub struct QuestionPlugin;

impl QuestionPlugin {
    pub async fn question(event: CQEvent, config: AppConfig) {
        let cq_addr = config.cq_addr;
        let msg = event.raw_message.as_ref().unwrap();
        let group_id = event.group_id.unwrap();
        let re = Regex::new(r"^[\?？¿⁇❓❔]+$").unwrap();
        if !re.is_match(msg) {
            return;
        }
        reqwest::get(format!(
            "http://{cq_addr}/send_group_msg?group_id={group_id}&message={msg}"
        ))
        .await
        .unwrap();
    }
}

#[async_trait::async_trait]
impl Plugin for QuestionPlugin {
    fn name(&self) -> &'static str {
        "question"
    }
    fn help(&self) -> &'static str {
        "自动复读问号"
    }
    fn event_type(&self) -> &'static str {
        "message group"
    }
    async fn handle(&self, event: CQEvent, config: AppConfig) {
        Self::question(event, config).await;
    }
}
