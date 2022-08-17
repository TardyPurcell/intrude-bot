use futures::lock::Mutex;
use std::sync::Arc;

use regex::Regex;
use reqwest;

use crate::{
    models::{CQEvent, Plugin},
    AppConfig,
};

#[derive(Clone)]
struct QuestionPluginState {
    // ignored_cnt: Arc<Mutex<usize>>,
    last_question_timestamp: Arc<Mutex<i64>>,
}

#[derive(Clone)]
pub struct QuestionPluginConfig {
    // ignore_limit: Arc<Mutex<usize>>,
    pub sleep_seconds: i64,
}

#[derive(Clone)]
pub struct QuestionPlugin {
    state: QuestionPluginState,
    config: Arc<Mutex<QuestionPluginConfig>>,
}

impl QuestionPlugin {
    pub fn new(config: Option<QuestionPluginConfig>) -> Self {
        QuestionPlugin {
            state: QuestionPluginState {
                // ignore_limit: Arc::new(Mutex::new(2)),
                // ignored_cnt: Arc::new(Mutex::new(0)),
                last_question_timestamp: Arc::new(Mutex::new(0)),
            },
            config: Arc::new(Mutex::new(
                config.unwrap_or(QuestionPluginConfig { sleep_seconds: 10 }),
            )),
        }
    }
    async fn question(&self, event: CQEvent, app_config: AppConfig) {
        let cq_addr = app_config.cq_addr;
        let msg = event.raw_message.as_ref().unwrap();
        let group_id = event.group_id.unwrap();
        let re = Regex::new(r"^[\?？¿⁇❓❔]+$").unwrap();
        if !re.is_match(msg) {
            return;
        }
        // let mut ignored_cnt = self.state.ignored_cnt.lock().await;
        // *ignored_cnt += 1;
        // if *ignored_cnt <= *self.state.ignore_limit.lock().await {
        // return;
        // }
        // *ignored_cnt = 0;
        let now_timestamp = chrono::Utc::now().timestamp();
        let mut last_question_timestamp = self.state.last_question_timestamp.lock().await;
        if now_timestamp - *last_question_timestamp < self.config.lock().await.sleep_seconds {
            return;
        }
        *last_question_timestamp = now_timestamp;
        reqwest::get(format!(
            "http://{cq_addr}/send_group_msg?group_id={group_id}&message={msg}"
        ))
        .await
        .unwrap();
    }
}

#[async_trait::async_trait]
impl Plugin for QuestionPlugin {
    fn name(&self) -> &'static str {
        "question"
    }
    fn help(&self) -> &'static str {
        "自动复读问号"
    }
    fn event_type(&self) -> &'static str {
        "message group"
    }
    async fn handle(&self, event: CQEvent, config: AppConfig) {
        self.question(event, config).await;
    }
}
