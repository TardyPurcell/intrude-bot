use regex::Regex;
use reqwest;

use crate::models::{CQEvent, Plugin};

#[derive(Clone)]
pub struct QuestionPlugin;

impl QuestionPlugin {
    pub async fn question(event: CQEvent) {
        let msg = event.raw_message.as_ref().unwrap();
        let group_id = event.group_id.unwrap();
        let re = Regex::new(r"^[\?？¿⁇❓❔]+$").unwrap();
        if !re.is_match(msg) {
            return;
        }
        reqwest::get(format!(
            "http://localhost:5700/send_group_msg?group_id={group_id}&message={msg}"
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
    async fn handle(&self, event: CQEvent) {
        Self::question(event).await;
    }
}
