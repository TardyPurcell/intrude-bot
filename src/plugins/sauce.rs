use regex::Regex;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;

use crate::bot::Bot;
use crate::models::{CQEvent, Plugin, PluginSenario};

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct SaucePluginConfig {
    api_key: Option<String>,
}
pub struct SaucePlugin {
    config: SaucePluginConfig,
}
impl SaucePlugin {
    pub fn new(config: Option<SaucePluginConfig>) -> Self {
        SaucePlugin {
            config: config.unwrap_or_default(),
        }
    }
    async fn sauce(&self, event: CQEvent, bot: &Bot) -> Result<(), Box<dyn Error + Send>> {
        let msg = event.raw_message.as_ref().unwrap();
        let re =
            Regex::new(r"^>sauce\s*\[CQ:image,[^\]]*url=(?P<img_url>[^,\]]+)[^\]]*\]\s*$").unwrap();
        if !re.is_match(msg) {
            return Ok(());
        }
        let img_url = re.replace_all(&msg, "$img_url").to_string();
        let resp = reqwest::Client::new()
            .get("https://saucenao.com/search.php")
            .query(&[
                ("db", "999"),
                ("output_type", "2"),
                ("numres", "1"),
                ("api_key", self.config.api_key.as_ref().unwrap()),
                ("url", &img_url),
            ])
            .send()
            .await
            .map_err(|err| Box::new(err) as Box<dyn Error + Send>)?
            .json::<SauceResponse>()
            .await
            .unwrap();
        if resp.results.len() == 0 {
            bot.api_request(
                "send_group_msg",
                json!({
                    "group_id": event.group_id.as_ref().unwrap(),
                    "message": "没有找到结果",
                }),
            )
            .await?;
            return Ok(());
        }
        for result in resp.results {
            let msg = format!(
                "相似度 {similarity}\r\n[CQ:image,file={img_url}]\r\n{result_url}",
                similarity = result.header.similarity,
                img_url = result.header.thumbnail,
                result_url = match result.data.ext_urls {
                    Some(urls) => urls.join("\r\n"),
                    None => String::new(),
                }
            );

            bot.api_request(
                "send_group_msg",
                json!({
                    "group_id": event.group_id.unwrap(),
                    "message": msg,
                }),
            )
            .await?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl Plugin for SaucePlugin {
    fn name(&self) -> &'static str {
        "sauce"
    }
    fn description(&self) -> &'static str {
        "SauceNAO以图搜图"
    }
    fn help(&self) -> &'static str {
        "用法:\r\n>sauce <图片>"
    }
    fn senario(&self) -> PluginSenario {
        PluginSenario::Group
    }
    async fn handle(&self, event: CQEvent, bot: &Bot) -> Result<(), Box<dyn Error + Send>> {
        match event.post_type.as_str() {
            "message" => self.sauce(event, bot).await,
            _ => Ok(()),
        }
    }
}

#[derive(Deserialize)]
struct SauceResponse {
    // header: Header,
    results: Vec<SauceResult>,
}

#[derive(Deserialize)]
struct SauceResult {
    header: SauceResultHeader,
    data: SauceResultData,
}

#[derive(Deserialize)]
struct SauceResultHeader {
    similarity: String,
    thumbnail: String,
}

#[derive(Deserialize)]
struct SauceResultData {
    ext_urls: Option<Vec<String>>,
}
