use regex::Regex;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Receiver;

use crate::models::{Plugin, CQEvent, PluginSenario};

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
                            format!("{:10}:\t{}\r\n", plugin.name(), plugin.description()).as_str(),
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