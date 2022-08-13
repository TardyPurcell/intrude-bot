use regex::Regex;
use reqwest;

use crate::models::{CQEvent, Plugin};

#[derive(Clone)]
pub struct EchoPlugin;

impl EchoPlugin {
    async fn echo(event: CQEvent) {
        let msg = event.raw_message.as_ref().unwrap();
        let group_id = event.group_id.unwrap();
        let re = Regex::new(r"^(?P<cmd>>echo)\s*(?P<content>.*)$").unwrap();
        if !re.is_match(msg) {
            return;
        }
        let content = re.replace_all(&msg, "$content").to_string();
        reqwest::get(format!(
            "http://localhost:5700/send_group_msg?group_id={group_id}&message={content}"
        ))
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
    async fn handle(&self, event: CQEvent) {
        Self::echo(event).await
    }
}
