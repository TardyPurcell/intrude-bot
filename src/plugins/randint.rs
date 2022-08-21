use rand::Rng;
use regex::Regex;
use serde_json::json;

use crate::models::{CQEvent, Plugin, PluginSenario};
use crate::bot::Bot;

pub struct RandintPlugin;
impl RandintPlugin {
    pub fn new() -> Self {
        RandintPlugin {}
    }
    async fn randint(&self, event: CQEvent, bot: &Bot) {
        let msg = event.raw_message.as_ref().unwrap();
        let re = Regex::new(r"^>randint\s+(?P<min>\d+)\s+(?P<max>\d+)\s*$").unwrap();
        if !re.is_match(msg) {
            return;
        }
        let min = re.replace_all(&msg, "$min").parse::<u128>().unwrap();
        let max = re.replace_all(&msg, "$max").parse::<u128>().unwrap();
        if min > max {
            bot.api_request(
                "send_group_msg",
                json!({
                    "group_id": event.group_id.as_ref().unwrap(),
                    "message": "homo特有的10比9大",
                }),
            )
            .await;
            return;
        }
        let rand = rand::thread_rng().gen_range(min..=max);
        bot.api_request(
            "send_group_msg",
            json!({
                "group_id": event.group_id.as_ref().unwrap(),
                "message": format!("{}", rand),
            }),
        )
        .await;
    }
}

#[async_trait::async_trait]
impl Plugin for RandintPlugin {
    fn name(&self) -> &'static str {
        "randint"
    }
    fn description(&self) -> &'static str {
        "随机非负整数"
    }
    fn help(&self) -> &'static str {
        "用法:\r\n>randint <min> <max>\r\n\tmin: 最小值\r\n\tmax: 最大值\r\n\r\n返回一个[min, max]之间的随机非负整数\r\n注意: min, max在u128范围内"
    }
    fn senario(&self) -> PluginSenario {
        PluginSenario::Group
    }
    async fn handle(&self, event: CQEvent, bot: &Bot) {
        match event.post_type.as_str() {
            "message" => match event.message_type.as_ref().unwrap().as_str() {
                "group" => self.randint(event, bot).await,
                _ => (),
            },
            _ => (),
        }
    }
}