use rand::Rng;
use regex::Regex;
use serde_json::json;
use std::error::Error;

use crate::bot::Bot;
use crate::models::{CQEvent, Plugin, PluginSenario};

pub struct RandintPlugin;
impl RandintPlugin {
    pub fn new() -> Self {
        RandintPlugin {}
    }
    async fn randint(&self, event: CQEvent, bot: &Bot) -> Result<(), Box<dyn Error + Send>> {
        let msg = event.raw_message.as_ref().unwrap();
        let re = Regex::new(r"^>randint\s+(?P<min>\d+)\s+(?P<max>\d+)\s*$").unwrap();
        if !re.is_match(msg) {
            return Ok(());
        }
        let min = re.replace_all(&msg, "$min").parse::<u128>();
        let max = re.replace_all(&msg, "$max").parse::<u128>();
        if let Err(_) = min {
            return Ok(());
        }
        if let Err(_) = max {
            return Ok(());
        }
        let min = min.unwrap();
        let max = max.unwrap();
        if min > max {
            bot.api_request(
                "send_group_msg",
                json!({
                    "group_id": event.group_id.as_ref().unwrap(),
                    "message": "homo特有的10比9大",
                }),
            )
            .await?;
            return Ok(());
        }
        let rand = rand::thread_rng().gen_range(min..=max);
        bot.api_request(
            "send_group_msg",
            json!({
                "group_id": event.group_id.as_ref().unwrap(),
                "message": format!("{}", rand),
            }),
        )
        .await?;
        Ok(())
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
    async fn handle(&self, event: CQEvent, bot: &Bot) -> Result<(), Box<dyn Error + Send>> {
        match event.post_type.as_str() {
            "message" => match event.message_type.as_ref().unwrap().as_str() {
                "group" => self.randint(event, bot).await,
                _ => Ok(()),
            },
            _ => Ok(()),
        }
    }
}
