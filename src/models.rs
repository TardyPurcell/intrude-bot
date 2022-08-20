use regex::Regex;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Receiver;

use crate::plugins::{
    ArchivePluginConfig, EchoPluginConfig, QuestionPluginConfig, SaucePluginConfig,
};

#[derive(Deserialize, Serialize)]
pub struct BotConfig {
    pub listen_addr: String,
    pub cq_addr: String,
}
impl Default for BotConfig {
    fn default() -> Self {
        BotConfig {
            listen_addr: "127.0.0.1:5701".to_string(),
            cq_addr: "127.0.0.1:5700".to_string(),
        }
    }
}

#[derive(Default, Deserialize, Serialize)]
pub struct PluginsConfig {
    pub archive: Option<ArchivePluginConfig>,
    pub echo: Option<EchoPluginConfig>,
    pub question: Option<QuestionPluginConfig>,
    pub sauce: Option<SaucePluginConfig>,
}

#[derive(Default, Deserialize, Serialize)]
pub struct AppConfig {
    pub bot: BotConfig,
    pub plugins: PluginsConfig,
}

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

#[derive(PartialEq)]
// #[allow(dead_code)]
pub enum PluginSenario {
    Private,
    Group,
    Both,
}

#[async_trait::async_trait]
pub trait Plugin {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn help(&self) -> &'static str;
    fn senario(&self) -> PluginSenario;
    async fn handle(&self, event: CQEvent, bot: &Bot) -> ();
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
    plugins: Vec<Box<dyn Plugin + Send + Sync>>,
    config: BotConfig,
    event_receiver: Receiver<CQEvent>,
    client: reqwest::Client,
}

impl Bot {
    pub fn new(rx: Receiver<CQEvent>, cfg: BotConfig) -> Self {
        Bot {
            plugins: Vec::new(),
            config: cfg,
            event_receiver: rx,
            client: reqwest::Client::new(),
        }
    }
    pub fn register_plugin(&mut self, plugin: impl Plugin + Send + Sync + 'static) {
        self.plugins.push(Box::new(plugin));
    }
    pub async fn run(&mut self) {
        loop {
            let event = self.event_receiver.recv().await.unwrap();
            self.handle_help(event.clone()).await;
            for plugin in &self.plugins {
                // let config = self.config.clone();
                plugin.handle(event.clone(), self).await;
            }
        }
    }
    pub async fn api_request(&self, api: &str, json: impl Serialize) -> Response {
        self.client
            .post(format!(
                "http://{cq_addr}/{api}",
                cq_addr = self.config.cq_addr
            ))
            .json(&json)
            .send()
            .await
            .unwrap()
    }
    async fn handle_help(&self, event: CQEvent) {
        if event.post_type != "message" {
            return;
        }
        let msg = event.raw_message.as_ref().unwrap();
        let re = Regex::new(r"^(?P<cmd>>help)($|\s+(?P<content>.*)$)").unwrap();
        if !re.is_match(msg) {
            return;
        }
        let CQEvent {
            group_id,
            user_id,
            message_type,
            ..
        } = event;
        let message_type = match message_type.unwrap().as_str() {
            "private" => PluginSenario::Private,
            "group" => PluginSenario::Group,
            _ => unreachable!(),
        };
        let mut resp = String::new();
        let content = re.replace_all(&msg, "$content").to_string();
        match content.as_str() {
            "" => {
                for plugin in self.plugins.iter() {
                    if plugin.senario() == message_type || plugin.senario() == PluginSenario::Both {
                        resp.push_str(
                            format!("{}: {}\r\n", plugin.name(), plugin.description()).as_str(),
                        );
                    }
                }
                if resp.is_empty() {
                    resp.push_str("没有可用的插件");
                }
            }
            _ => {
                for plugin in self.plugins.iter() {
                    if (plugin.senario() == message_type || plugin.senario() == PluginSenario::Both)
                        && plugin.name() == content
                    {
                        resp.push_str(
                            format!(
                                "{}: {}\r\n\r\n{}\r\n",
                                plugin.name(),
                                plugin.description(),
                                plugin.help()
                            )
                            .as_str(),
                        );
                    }
                }
                if resp.is_empty() {
                    resp.push_str("未找到插件或插件不可用");
                }
            }
        }
        let resp = match content.as_str() {
            "" => format!("用法:\r\n>help [插件名]\r\n\r\n插件列表:\r\n{resp}"),
            _ => resp,
        };
        match message_type {
            PluginSenario::Private => {
                self.api_request(
                    "send_msg",
                    SendPrivateMsgReq {
                        user_id: user_id.unwrap(),
                        message: resp,
                    },
                )
                .await;
            }
            PluginSenario::Group => {
                self.api_request(
                    "send_group_msg",
                    SendGroupMsgReq {
                        group_id: group_id.unwrap(),
                        message: resp,
                    },
                )
                .await;
            }
            _ => unreachable!(),
        }
    }
}

unsafe impl Sync for Bot {}

#[derive(Serialize)]
struct SendGroupMsgReq {
    group_id: i64,
    message: String,
}

#[derive(Serialize)]
struct SendPrivateMsgReq {
    user_id: i64,
    message: String,
}
