use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::bot::Bot;
use crate::models::{CQEvent, Plugin, PluginSenario};

#[derive(Default, Deserialize, Serialize)]
pub struct HOKpPluginConfig {
    pub patterns: Vec<String>,
    pub whitelist: Vec<i64>,
}

pub struct HOKpPlugin {
    config: HOKpPluginConfig,
}

impl HOKpPlugin {
    pub fn new(config: Option<HOKpPluginConfig>) -> Self {
        HOKpPlugin {
            config: config.unwrap_or_default(),
        }
    }
    async fn hokp(&self, event: CQEvent, bot: &Bot) {
        let group_id = event.group_id.unwrap();
        if let None = self.config.whitelist.iter().find(|&&x| x == group_id) {
            return;
        }
        let msg = event.raw_message.unwrap();
        let mut is_hokp = false;
        for pattern in self.config.patterns.iter() {
            let re = Regex::new(pattern).unwrap();
            if re.is_match(&msg) {
                is_hokp = true;
                break;
            }
        }
        if !is_hokp {
            return;
        }
        bot.api_request(
            "send_group_msg",
            json!({
                "group_id": event.group_id.unwrap(),
                "message": " 要不咱玩农吧"
            }),
        )
        .await;
    }
}

#[async_trait::async_trait]
impl Plugin for HOKpPlugin {
    fn name(&self) -> &'static str {
        "hokp"
    }

    fn description(&self) -> &'static str {
        "农批"
    }

    fn help(&self) -> &'static str {
        ""
    }

    fn senario(&self) -> PluginSenario {
        PluginSenario::Group
    }

    async fn handle(&self, event: CQEvent, bot: &Bot) {
        match event.post_type.as_str() {
            "message" => self.hokp(event, bot).await,
            _ => (),
        }
    }
}
