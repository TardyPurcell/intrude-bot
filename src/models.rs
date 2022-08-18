use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::AppConfig;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CQEvent {
    pub time: i64,
    pub self_id: i64,
    pub post_type: String,

    // 消息上报
    pub message_type: Option<String>,

    // 请求上报
    pub request_type: Option<String>,

    // 通知上报
    pub notice_type: Option<String>,

    // 元事件上报
    pub meta_event_type: Option<String>,

    // ...
    pub sub_type: Option<String>,
    pub message_id: Option<i32>,
    pub user_id: Option<i64>,
    pub raw_message: Option<String>,
    pub font: Option<i64>,
    pub group_id: Option<i64>,
    pub operator_id: Option<i64>,
}

pub enum CQEventType {
    Message(CQEvent),
    Request(CQEvent),
    Notice(CQEvent),
    MetaEvent(CQEvent),
}

#[async_trait::async_trait]
pub trait Plugin {
    fn name(&self) -> &'static str;
    fn help(&self) -> &'static str;
    fn event_type(&self) -> &'static str;
    async fn handle(&self, event: CQEvent, config: AppConfig) -> ();
}

// pub trait PluginClone {
//     fn clone_box(&self) -> Box<dyn Plugin + Send>;
// }

// impl<T> PluginClone for T
// where
//     T: 'static + Plugin + Clone + Send,
// {
//     fn clone_box(&self) -> Box<dyn Plugin + Send> {
//         Box::new(self.clone())
//     }
// }

// impl Clone for Box<dyn Plugin + Send> {
//     fn clone(&self) -> Self {
//         self.clone_box()
//     }
// }

// #[derive(Clone)]
pub struct Bot {
    plugins: Vec<Box<dyn Plugin + Send>>,
    config: AppConfig,
}

impl Bot {
    pub fn register_plugin(&mut self, plugin: impl Plugin + 'static + Send) {
        self.plugins.push(Box::new(plugin));
    }
    pub async fn handle_message(&self, event: CQEvent) {
        match event.message_type.clone().unwrap().as_str() {
            "group" => self.handle_group_message(event).await,
            "private" => self.handle_private_message(event).await,
            _ => (),
        }
    }
    async fn handle_group_help(&self, event: CQEvent) {
        let msg = event.raw_message.as_ref().unwrap();
        let group_id = event.group_id.unwrap();
        let re = Regex::new(r"^(?P<cmd>>help)($|\s+(?P<content>.*)$)").unwrap();
        if !re.is_match(msg) {
            return;
        }
        let mut resp = String::new();
        let content = re.replace_all(&msg, "$content").to_string();
        match content.as_str() {
            "" => {
                for plugin in self.plugins.iter() {
                    if plugin.event_type() == "message group"
                        || plugin.event_type() == "notice group"
                    {
                        resp.push_str(format!("{}: {}\r\n", plugin.name(), plugin.help()).as_str());
                    }
                }
                if resp.is_empty() {
                    resp.push_str("没有可用的插件");
                }
            }
            _ => {
                for plugin in self.plugins.iter() {
                    if plugin.name() == content {
                        resp.push_str(format!("{}: {}\r\n", plugin.name(), plugin.help()).as_str());
                    }
                }
                if resp.is_empty() {
                    resp.push_str("没有可用的插件");
                }
            }
        }
        let cq_addr = &self.config.cq_addr;
        reqwest::Client::new()
            .post(format!("http://{cq_addr}/send_group_msg"))
            .json(&SendGroupMsgReq {
                group_id,
                message: resp,
            })
            .send()
            .await
            .unwrap();
    }
    async fn handle_group_message(&self, event: CQEvent) {
        // println!("{:?}", event);
        self.handle_group_help(event.clone()).await;
        for plugin in &self.plugins {
            if plugin.event_type() == "message group" {
                plugin.handle(event.clone(), self.config.clone()).await;
            }
        }
    }
    async fn handle_private_message(&self, _event: CQEvent) {}
    pub fn new(cfg: AppConfig) -> Self {
        Bot {
            plugins: Vec::new(),
            config: cfg,
        }
    }
    pub async fn handle_request(&self, _event: CQEvent) {}
    pub async fn handle_notice(&self, event: CQEvent) {
        // println!("{:?}", event);
        for plugin in &self.plugins {
            if plugin.event_type() == "notice group" || plugin.event_type() == "notice private" {
                plugin.handle(event.clone(), self.config.clone()).await;
            }
        }
    }
    pub async fn handle_meta_event(&self, _event: CQEvent) {}
}

#[derive(Serialize)]
struct SendGroupMsgReq {
    group_id: i64,
    message: String,
}
