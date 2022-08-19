use chrono::{Local, TimeZone};
use log::debug;
use regex::Regex;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::RwLock;

use crate::models::{Bot, CQEvent, Plugin, PluginSenario};

#[derive(Clone)]
pub struct ArchivePluginConfig;

struct ArchivePluginState {
    is_enable: bool,
}

pub struct ArchivePlugin {
    state: RwLock<ArchivePluginState>,
    _config: ArchivePluginConfig,
}

impl ArchivePlugin {
    pub fn new(config: Option<ArchivePluginConfig>) -> Self {
        ArchivePlugin {
            state: RwLock::new(ArchivePluginState { is_enable: false }),
            _config: config.unwrap_or(ArchivePluginConfig),
        }
    }
    async fn archive(&self, event: CQEvent, bot: &Bot) {
        if !self.state.read().await.is_enable {
            return;
        }
        let CQEvent {
            group_id,
            message_id,
            user_id,
            operator_id,
            ..
        } = event;
        let operator_info = bot
            .api_request(
                "get_group_member_info",
                json!(
                    {
                        "group_id": group_id.unwrap(),
                        "user_id": operator_id.unwrap(),
                    }
                ),
            )
            .await
            .json::<DataExtractor>()
            .await
            .unwrap()
            .data;
        let user_info = bot
            .api_request(
                "get_group_member_info",
                json!(
                    {
                        "group_id": group_id.unwrap(),
                        "user_id": user_id.unwrap(),
                    }
                ),
            )
            .await
            .json::<DataExtractor>()
            .await
            .unwrap()
            .data;
        let recalled_msg_info = bot
            .api_request(
                "get_msg",
                json!(
                    {
                        "message_id": message_id.unwrap(),
                    }
                ),
            )
            .await
            .json::<DataExtractor>()
            .await
            .unwrap()
            .data;
        let recalled_msg_content = recalled_msg_info.message.unwrap();
        let recalled_msg_timestamp = recalled_msg_info.time.unwrap();
        let mut operator_name = operator_info.card.unwrap();
        let mut user_name = user_info.card.unwrap();
        if operator_name == "" {
            operator_name = operator_info.nickname.unwrap();
        }
        if user_name == "" {
            user_name = user_info.nickname.unwrap();
        }
        if operator_id == user_id {
            user_name = "自己".to_string();
        }
        let resp = format!(
            "{operator_name} 撤回了 {user_name} 于 {datetime} 发送的消息：",
            datetime = Local
                .timestamp(recalled_msg_timestamp.into(), 0)
                .naive_local()
        );
        bot.api_request(
            "send_group_msg",
            json!(
                {
                    "group_id": group_id.unwrap(),
                    "message": resp
                }
            ),
        )
        .await;
        bot.api_request(
            "send_group_msg",
            json!({"group_id": group_id.unwrap(), "message": recalled_msg_content}),
        )
        .await;
    }
    async fn toggle(&self, event: CQEvent, bot: &Bot) {
        let re = Regex::new(r"^>archive\s+toggle\s*$").unwrap();
        let msg = event.raw_message.unwrap();
        if re.is_match(&msg) {
            let mut state = self.state.write().await;
            state.is_enable = !state.is_enable;
            if state.is_enable {
                bot.api_request(
                    "send_group_msg",
                    json!({"group_id": event.group_id.unwrap(), "message": "撤回记录已开启"}),
                )
                .await;
            } else {
                bot.api_request(
                    "send_group_msg",
                    json!({"group_id": event.group_id.unwrap(), "message": "撤回记录已关闭"}),
                )
                .await;
            }
        } else {
            debug!("failure");
        }
    }
}

#[async_trait::async_trait]
impl Plugin for ArchivePlugin {
    fn name(&self) -> &'static str {
        "archive"
    }
    fn description(&self) -> &'static str {
        "自动复读已撤回的消息"
    }
    fn help(&self) -> &'static str {
        ">archive toggle 开启或关闭撤回记录"
    }
    fn senario(&self) -> PluginSenario {
        PluginSenario::Group
    }
    async fn handle(&self, event: CQEvent, bot: &Bot) {
        match event.post_type.as_str() {
            "notice" => match event.notice_type.as_ref().unwrap().as_str() {
                "group_recall" => self.archive(event, bot).await,
                _ => (),
            },
            "message" => match event.message_type.as_ref().unwrap().as_str() {
                "group" => self.toggle(event, bot).await,
                _ => (),
            },
            _ => (),
        }
    }
}

#[derive(Deserialize)]
struct DataExtractor {
    data: InnerData,
}

#[derive(Deserialize)]
struct InnerData {
    nickname: Option<String>,
    card: Option<String>,
    message: Option<String>,
    time: Option<i32>,
}
