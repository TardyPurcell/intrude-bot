use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;
use tokio::sync::RwLock;

use crate::bot::Bot;
use crate::models::{CQEvent, Plugin, PluginSenario};

#[derive(Default, Deserialize, Serialize)]
struct HOKpPluginState {
    // ignored_cnt: Arc<Mutex<usize>>,
    last_msg_timestamp: i64,
}

#[derive(Default, Deserialize, Serialize)]
pub struct HOKpPluginConfig {
    pub not_hokp_patterns: Vec<String>,
    pub hokp_patterns: Vec<String>,
    pub whitelist: Vec<i64>,
    pub sleep_seconds: i64,
}

pub struct HOKpPlugin {
    state: RwLock<HOKpPluginState>,
    config: HOKpPluginConfig,
}

impl HOKpPlugin {
    pub fn new(config: Option<HOKpPluginConfig>) -> Self {
        HOKpPlugin {
            state: RwLock::new(HOKpPluginState {
                last_msg_timestamp: 0,
            }),
            config: config.unwrap_or_default(),
        }
    }
    async fn hokp(&self, event: CQEvent, bot: &Bot) -> Result<(), Box<dyn Error + Send>> {
        let msg = event.raw_message.unwrap();
        let mut not_hokp = false;
        for pattern in self.config.not_hokp_patterns.iter() {
            let re = Regex::new(pattern).unwrap();
            if re.is_match(&msg) {
                not_hokp = true;
                break;
            }
        }
        if !not_hokp {
            return Ok(());
        }
        bot.api_request(
            "send_group_msg",
            json!({
                "group_id": event.group_id.unwrap(),
                "message": "要不咱玩农吧"
            }),
        )
        .await?;
        let mut state = self.state.write().await;
        let now_timestamp = chrono::Utc::now().timestamp();
        state.last_msg_timestamp = now_timestamp;
        Ok(())
    }

    async fn anti_hokp(&self, event: CQEvent, bot: &Bot) -> Result<(), Box<dyn Error + Send>> {
        let msg = event.raw_message.unwrap();
        let mut is_hokp = false;
        for pattern in self.config.hokp_patterns.iter() {
            let re = Regex::new(pattern).unwrap();
            if re.is_match(&msg) {
                is_hokp = true;
                break;
            }
        }
        if !is_hokp {
            return Ok(());
        }
        bot.api_request(
            "send_group_msg",
            json!({
                "group_id": event.group_id.unwrap(),
                "message": "农批收收味"
            }),
        )
        .await?;
        let mut state = self.state.write().await;
        let now_timestamp = chrono::Utc::now().timestamp();
        state.last_msg_timestamp = now_timestamp;
        Ok(())
    }
    fn filter(&self, group_id: i64) -> bool {
        self.config
            .whitelist
            .iter()
            .find(|&&x| x == group_id)
            .is_some()
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

    async fn handle(&self, event: CQEvent, bot: &Bot) -> Result<(), Box<dyn Error + Send>> {
        match event.post_type.as_str() {
            "message" => match event.message_type.as_ref().unwrap().as_str() {
                "group" => {
                    let group_id = event.group_id.clone().unwrap();
                    if !self.filter(group_id) {
                        return Ok(());
                    }
                    let now_timestamp = chrono::Utc::now().timestamp();
                    let state = self.state.read().await;
                    if now_timestamp - state.last_msg_timestamp < self.config.sleep_seconds {
                        return Ok(());
                    }
                    self.hokp(event.clone(), bot).await?;
                    self.anti_hokp(event, bot).await
                }
                "private" => Ok(()),
                _ => unreachable!(),
            },
            _ => Ok(()),
        }
    }
}
