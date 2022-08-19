use serde_json::json;

use regex::Regex;
use tokio::sync::RwLock;

use crate::models::{Bot, CQEvent, Plugin, PluginSenario};

#[derive(Clone)]
struct QuestionPluginState {
    // ignored_cnt: Arc<Mutex<usize>>,
    last_question_timestamp: i64,
}

#[derive(Clone)]
pub struct QuestionPluginConfig {
    // ignore_limit: Arc<Mutex<usize>>,
    pub sleep_seconds: i64,
}

pub struct QuestionPlugin {
    state: RwLock<QuestionPluginState>,
    config: QuestionPluginConfig,
}

impl QuestionPlugin {
    pub fn new(config: Option<QuestionPluginConfig>) -> Self {
        QuestionPlugin {
            state: RwLock::new(QuestionPluginState {
                // ignore_limit: Arc::new(Mutex::new(2)),
                // ignored_cnt: Arc::new(Mutex::new(0)),
                last_question_timestamp: 0,
            }),
            config: config.unwrap_or(QuestionPluginConfig { sleep_seconds: 0 }),
        }
    }
    async fn question(&self, event: CQEvent, bot: &Bot) {
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
        let mut state = self.state.write().await;
        if now_timestamp - state.last_question_timestamp < self.config.sleep_seconds {
            return;
        }
        state.last_question_timestamp = now_timestamp;
        bot.api_request(
            "send_group_msg",
            json!({
                "group_id": group_id,
                "message": msg,
            }),
        )
        .await;
    }
}

#[async_trait::async_trait]
impl Plugin for QuestionPlugin {
    fn name(&self) -> &'static str {
        "question"
    }
    fn description(&self) -> &'static str {
        "自动复读问号"
    }
    fn help(&self) -> &'static str {
        ""
    }
    fn senario(&self) -> PluginSenario {
        PluginSenario::Group
    }
    async fn handle(&self, event: CQEvent, bot: &Bot) {
        match event.post_type.as_str() {
            "message" => match event.message_type.as_ref().unwrap().as_str() {
                "group" => self.question(event, bot).await,
                _ => (),
            },
            _ => (),
        }
    }
}
