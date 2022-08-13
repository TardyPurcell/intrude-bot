use serde::{Deserialize, Serialize};

use crate::{
    models::{CQEvent, Plugin},
    AppConfig,
};

#[derive(Clone)]
pub struct ArchivePlugin;

impl ArchivePlugin {
    pub async fn archive(event: CQEvent, config: AppConfig) {
        let cq_addr = config.cq_addr;
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
        let group_id = group_id.unwrap();
        let message_id = message_id.unwrap();
        let user_id = user_id.unwrap();
        let operator_id = operator_id.unwrap();
        let operator_info = reqwest::get(format!(
            "http://{cq_addr}/get_group_member_info?group_id={group_id}&user_id={operator_id}"
        ))
        .await
        .unwrap()
        .json::<DataExtractor>()
        .await
        .unwrap()
        .data;
        let user_info = reqwest::get(format!(
            "http://{cq_addr}/get_group_member_info?group_id={group_id}&user_id={user_id}"
        ))
        .await
        .unwrap()
        .json::<DataExtractor>()
        .await
        .unwrap()
        .data;
        let recalled_msg =
            reqwest::get(format!("http://{cq_addr}/get_msg?message_id={message_id}"))
                .await
                .unwrap()
                .json::<DataExtractor>()
                .await
                .unwrap()
                .data
                .message
                .unwrap();
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
        let resp = format!("{operator_name} 撤回了 {user_name} 的消息：{recalled_msg}");
        println!("{}", resp);
        reqwest::get(format!(
            "http://{cq_addr}/send_group_msg?group_id={group_id}&message={resp}"
        ))
        .await
        .unwrap();
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
    fn event_type(&self) -> &'static str {
        "notice group"
    }
    async fn handle(&self, event: CQEvent, config: AppConfig) {
        Self::archive(event, config).await;
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
}

#[derive(Serialize)]
struct Req {
    group_id: i64,
    message: String,
}
