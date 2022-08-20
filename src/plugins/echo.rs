use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::models::{Bot, CQEvent, Plugin, PluginSenario};

#[derive(Serialize, Deserialize)]
pub struct EchoPluginConfig;

pub struct EchoPlugin {
    _config: EchoPluginConfig,
}

impl EchoPlugin {
    pub fn new(config: Option<EchoPluginConfig>) -> Self {
        EchoPlugin {
            _config: config.unwrap_or(EchoPluginConfig),
        }
    }
    async fn echo(&self, event: CQEvent, bot: &Bot) {
        let msg = event.raw_message.as_ref().unwrap();
        let group_id = event.group_id.unwrap();
        let re = Regex::new(r"^>echo\s+(?P<content>.+)$").unwrap();
        if !re.is_match(msg) {
            return;
        }
        let content = re.replace_all(&msg, "$content").to_string();
        bot.api_request(
            "send_group_msg",
            &Req {
                group_id,
                message: content,
            },
        )
        .await;
    }
}

#[async_trait::async_trait]
impl Plugin for EchoPlugin {
    fn name(&self) -> &'static str {
        "echo"
    }
    fn description(&self) -> &'static str {
        "复读机"
    }
    fn help(&self) -> &'static str {
        "用法:\r\n>echo <复读内容>"
    }
    fn senario(&self) -> PluginSenario {
        PluginSenario::Group
    }
    async fn handle(&self, event: CQEvent, bot: &Bot) {
        match event.post_type.as_str() {
            "message" => self.echo(event, bot).await,
            _ => (),
        }
    }
}

#[derive(Serialize)]
struct Req {
    group_id: i64,
    message: String,
}
