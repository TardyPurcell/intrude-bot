use chrono::{Local, TimeZone};
use serde::Deserialize;
use serde_json::json;

use crate::models::{Bot, CQEvent, Plugin, PluginSenario};

#[derive(Clone)]
pub struct ArchivePluginConfig;

pub struct ArchivePlugin {
    _config: ArchivePluginConfig,
}

impl ArchivePlugin {
    pub fn new(config: Option<ArchivePluginConfig>) -> Self {
        ArchivePlugin {
            _config: config.unwrap_or(ArchivePluginConfig),
        }
    }
    async fn archive(&self, event: CQEvent, bot: &Bot) {
        if event.notice_type.as_ref().unwrap() != "group_recall" {
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
}

#[async_trait::async_trait]
impl Plugin for ArchivePlugin {
    fn name(&self) -> &'static str {
        "archive"
    }
    fn help(&self) -> &'static str {
        "自动复读已撤回的消息"
    }
    fn senario(&self) -> PluginSenario {
        PluginSenario::Group
    }
    async fn handle(&self, event: CQEvent, bot: &Bot) {
        self.archive(event, bot).await;
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
