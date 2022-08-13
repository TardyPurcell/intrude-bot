use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CQEvent {
    pub time: i64,
    pub self_id: i64,
    pub post_type: String,

    // 消息上报
    pub message_type: Option<String>,
    pub sub_type: Option<String>,
    pub message_id: Option<i32>,
    pub user_id: Option<i64>,
    pub raw_message: Option<String>,
    pub font: Option<i64>,
    pub group_id: Option<i64>,

    // 请求上报
    pub request_type: Option<String>,

    // 通知上报
    pub notice_type: Option<String>,

    // 元事件上报
    pub meta_event_type: Option<String>,
}

#[async_trait::async_trait]
pub trait Plugin: PluginClone {
    fn name(&self) -> &'static str;
    fn help(&self) -> &'static str;
    fn event_type(&self) -> &'static str;
    async fn handle(&self, event: CQEvent) -> ();
}

pub trait PluginClone {
    fn clone_box(&self) -> Box<dyn Plugin + Send>;
}

impl<T> PluginClone for T
where
    T: 'static + Plugin + Clone + Send,
{
    fn clone_box(&self) -> Box<dyn Plugin + Send> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Plugin + Send> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Clone)]
pub struct Bot {
    plugins: Vec<Box<dyn Plugin + Send>>,
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
    async fn handle_group_message(&self, event: CQEvent) {
        for plugin in &self.plugins {
            if plugin.event_type() == "message group" {
                plugin.handle(event.clone()).await;
            }
        }
    }
    async fn handle_private_message(&self, _event: CQEvent) {}
    pub fn new() -> Self {
        Bot {
            plugins: Vec::new(),
        }
    }
    pub async fn handle_request(&self, _event: CQEvent) {}
    pub async fn handle_notice(&self, _event: CQEvent) {}
    pub async fn handle_meta_event(&self, _event: CQEvent) {}
}
