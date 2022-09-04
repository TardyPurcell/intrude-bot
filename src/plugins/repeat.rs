use std::error::Error;

use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::RwLock;

use crate::{
    bot::Bot,
    models::{CQEvent, Plugin, PluginSenario},
};

#[derive(Default, Debug)]
struct RepeatPluginState {
    target_msg: Option<String>,
    target_cnt: i64,
    last_msg_timestamp: i64,
}
#[derive(Default, Deserialize, Serialize)]
pub struct RepeatPluginConfig {
    threshold: i64,
    pub sleep_seconds: i64,
}
pub struct RepeatPlugin {
    state: RwLock<RepeatPluginState>,
    config: RepeatPluginConfig,
}

#[async_trait::async_trait]
impl Plugin for RepeatPlugin {
    fn name(&self) -> &'static str {
        "repeat"
    }

    fn description(&self) -> &'static str {
        "人云亦云"
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
                    self.set_state(event.clone()).await?;
                    {
                        let now_timestamp = chrono::Utc::now().timestamp();
                        let state = self.state.read().await;
                        if now_timestamp - state.last_msg_timestamp < self.config.sleep_seconds {
                            debug!(
                                "plugin sleeping. {} seconds remaining. returning...",
                                self.config.sleep_seconds + state.last_msg_timestamp
                                    - now_timestamp
                            );
                            return Ok(());
                        }
                    }
                    self.do_repeat(event, bot).await
                }
                _ => Ok(()),
            },
            _ => Ok(()),
        }
    }
}

impl RepeatPlugin {
    pub fn new(config: Option<RepeatPluginConfig>) -> Self {
        RepeatPlugin {
            state: RwLock::new(Default::default()),
            config: config.unwrap_or_default(),
        }
    }
    async fn set_state(&self, event: CQEvent) -> Result<(), Box<dyn Error + Send>> {
        let mut state = self.state.write().await;
        match state.target_msg {
            None => {
                state.target_msg = event.raw_message;
                state.target_cnt = 1;
                debug!("state: {:?}", state);
            }
            Some(ref msg) => {
                let new_msg = event.raw_message.as_ref().unwrap();
                if new_msg == msg {
                    state.target_cnt += 1;
                } else {
                    state.target_msg = event.raw_message;
                    state.target_cnt = 1;
                }
                debug!("state: {:?}", state);
            }
        }
        Ok(())
    }
    async fn do_repeat(&self, event: CQEvent, bot: &Bot) -> Result<(), Box<dyn Error + Send>> {
        let mut state = self.state.write().await;
        if let Some(ref msg) = state.target_msg {
            let new_msg = event.raw_message.as_ref().unwrap();
            if new_msg == msg && state.target_cnt >= self.config.threshold {
                bot.api_request(
                    "send_group_msg",
                    json!({
                        "group_id": event.group_id.unwrap(),
                        "message": new_msg,
                    }),
                )
                .await?;
                state.target_msg = None;
                state.target_cnt = 0;
                let now_timestamp = chrono::Utc::now().timestamp();
                state.last_msg_timestamp = now_timestamp;
            }
        }
        Ok(())
    }
}
