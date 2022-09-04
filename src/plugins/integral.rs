use std::{collections::HashMap, error::Error};

use chrono::{Duration, Local, NaiveDateTime};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

use crate::{
    bot::Bot,
    models::{CQEvent, Plugin, PluginSenario},
};

struct IntegralPluginState {
    db: SqlitePool,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct IntegralPluginConfig {
    db_url: String,
}

#[allow(dead_code)]
pub struct IntegralPlugin {
    state: IntegralPluginState,
    config: IntegralPluginConfig,
}

#[async_trait::async_trait]
impl Plugin for IntegralPlugin {
    fn name(&self) -> &'static str {
        "integral"
    }

    fn description(&self) -> &'static str {
        "阻冲之"
    }

    fn help(&self) -> &'static str {
        concat!(
            "用法:\r\n",
            ">integral <cmd>\r\n",
            "\r\n",
            "cmd列表:\r\n",
            "\tderivative\t破戒\r\n",
            "\tpunch\t\t打卡\r\n",
            "\tranking\t查看群内排名\r\n",
            "\tstatus\t\t查看状态\r\n",
            "\r\n",
            "24小时内未打卡会导致计时清零"
        )
    }

    fn senario(&self) -> PluginSenario {
        PluginSenario::Group
    }

    async fn handle(&self, event: CQEvent, bot: &Bot) -> Result<(), Box<dyn Error + Send>> {
        match event.post_type.as_str() {
            "message" => match event.message_type.as_ref().unwrap().as_str() {
                "group" => self.integral(event, bot).await,
                "private" => Ok(()),
                _ => unreachable!(),
            },
            _ => Ok(()),
        }
    }
}

impl IntegralPlugin {
    pub async fn new(config: Option<IntegralPluginConfig>) -> Self {
        let config = config.unwrap_or_default();
        let state = IntegralPluginState {
            db: SqlitePoolOptions::new()
                .connect(&config.db_url)
                .await
                .expect("database connection failed"),
        };
        Self { state, config }
    }
    async fn integral(&self, event: CQEvent, bot: &Bot) -> Result<(), Box<dyn Error + Send>> {
        let cmd = Self::resolve(event.raw_message.unwrap());
        if let None = cmd {
            return Ok(());
        }
        let cmd = cmd.unwrap();
        let user_id = event.user_id.unwrap();
        let group_id = event.group_id.unwrap();
        if let Cmd::Derivative = cmd {
            self.derivative(user_id).await;
            bot.api_request(
                "send_group_msg",
                json!({
                    "group_id": group_id,
                    "message": "不准导！积回去！"
                }),
            )
            .await?;
            return Ok(());
        }
        if let Cmd::Ranking = cmd {
            let list = self.ranking(group_id, bot).await?;
            let mut msg = String::new();
            let mut previous = Duration::zero();
            let mut ranking = 0;
            for (index, entry) in list.iter().enumerate() {
                let user_info = bot
                    .api_request(
                        "get_group_member_info",
                        json!(
                            {
                                "group_id": group_id,
                                "user_id": entry.user_id,
                            }
                        ),
                    )
                    .await?
                    .json::<UserDataExtractor>()
                    .await
                    .unwrap()
                    .data;
                let user_name = match user_info.card {
                    None => user_info.nickname.unwrap(),
                    Some(name) => name,
                };
                if entry.score != previous {
                    ranking = index;
                    previous = entry.score;
                }
                msg.push_str(
                    format!(
                        "{:5}. {:20}\r\n\t\t\t{:>15}\r\n\r\n",
                        ranking,
                        user_name,
                        Self::duration_to_string(entry.score)
                    )
                    .as_str(),
                )
            }
            bot.api_request(
                "send_group_msg",
                json!({
                    "group_id": group_id,
                    "message": msg,
                }),
            )
            .await?;
            return Ok(());
        }
        let res = match cmd {
            Cmd::Punch => self.punch(user_id).await,
            Cmd::Status => self.status(user_id).await,
            Cmd::Derivative => unreachable!(),
            Cmd::Ranking => unreachable!(),
        };
        let user_info = bot
            .api_request(
                "get_group_member_info",
                json!(
                    {
                        "group_id": group_id,
                        "user_id": user_id,
                    }
                ),
            )
            .await?
            .json::<UserDataExtractor>()
            .await
            .unwrap()
            .data;
        let user_name = match user_info.card {
            None => user_info.nickname.unwrap(),
            Some(name) => name,
        };
        let msg = format!("{} 已戒导 {}", user_name, Self::duration_to_string(res));
        bot.api_request(
            "send_group_msg",
            json!({
                "group_id": group_id,
                "message": msg
            }),
        )
        .await?;
        Ok(())
    }
    fn resolve(msg: String) -> Option<Cmd> {
        let re = Regex::new(r"^>integral\s+(?P<cmd>\S+)\s*$").unwrap();
        let cmd = re.replace_all(&msg, "$cmd").to_string();
        match cmd.as_str() {
            "punch" => Some(Cmd::Punch),
            "status" => Some(Cmd::Status),
            "derivative" => Some(Cmd::Derivative),
            "ranking" => Some(Cmd::Ranking),
            _ => None,
        }
    }
    fn duration_to_string(dur: Duration) -> String {
        let weeks = dur.num_weeks();
        let days = dur.num_days() - 7 * dur.num_weeks();
        let hours = dur.num_hours() - 24 * dur.num_days();
        let minutes = dur.num_minutes() - 60 * dur.num_hours();
        let seconds = dur.num_seconds() - 60 * dur.num_minutes();

        let mut map = Vec::new();
        map.push(("w", weeks));
        map.push(("d", days));
        map.push(("h", hours));
        map.push(("m", minutes));
        map.push(("s", seconds));

        let mut ret = String::new();
        for (k, v) in map {
            if v == 0 && k != "s" {
                continue;
            }
            ret.push_str(format!("{v:02}{k}").as_str());
        }
        ret
    }
    async fn punch(&self, user_id: i64) -> Duration {
        let res = self.status(user_id).await;
        self.update_updated_at_db(user_id).await.ok();
        res
    }
    async fn status(&self, user_id: i64) -> Duration {
        let now = Local::now().naive_local();
        match self.get_updated_at_db(user_id).await {
            Ok(res) => {
                if now - res >= Duration::days(1) {
                    self.update_started_at_db(user_id).await.ok();
                    Duration::zero()
                } else {
                    let start_time = self.get_started_at_db(user_id).await.unwrap();
                    now - start_time
                }
            }
            Err(_) => {
                self.add_user_db(user_id).await.ok();
                Duration::zero()
            }
        }
    }
    async fn derivative(&self, user_id: i64) {
        self.status(user_id).await;
        self.update_started_at_db(user_id).await.ok();
    }
    async fn ranking(
        &self,
        group_id: i64,
        bot: &Bot,
    ) -> Result<Vec<RankingListEntry>, Box<dyn Error + Send>> {
        let member_list = bot
            .api_request("get_group_member_list", json!({ "group_id": group_id }))
            .await?
            .json::<MemberListExtractor>()
            .await
            .unwrap()
            .data;
        let mut ret: Vec<RankingListEntry> = Vec::new();
        for entry in member_list {
            ret.push(RankingListEntry {
                user_id: entry.user_id.unwrap(),
                score: self.status(entry.user_id.unwrap()).await,
            });
        }
        ret.sort_by(|a, b| b.score.cmp(&a.score));
        Ok(ret)
    }
    async fn get_started_at_db(
        &self,
        user_id: i64,
    ) -> Result<NaiveDateTime, Box<dyn Error + Send>> {
        sqlx::query!(
            r"SELECT started_at FROM integral_time_card WHERE user_id=$1",
            user_id
        )
        .fetch_one(&self.state.db)
        .await
        .map(|row| row.started_at)
        .map_err(|err| Box::new(err) as Box<dyn Error + Send>)
    }
    async fn get_updated_at_db(
        &self,
        user_id: i64,
    ) -> Result<NaiveDateTime, Box<dyn Error + Send>> {
        sqlx::query!(
            r"SELECT updated_at FROM integral_time_card WHERE user_id=$1",
            user_id
        )
        .fetch_one(&self.state.db)
        .await
        .map(|row| row.updated_at)
        .map_err(|err| Box::new(err) as Box<dyn Error + Send>)
    }

    async fn add_user_db(&self, user_id: i64) -> Result<(), Box<dyn Error + Send>> {
        let now = Local::now().naive_local();
        let long_time_ago = NaiveDateTime::from_timestamp(0, 0);
        sqlx::query!(
            r"INSERT INTO integral_time_card VALUES ($1, $2, $3)",
            user_id,
            now,
            long_time_ago
        )
        .execute(&self.state.db)
        .await
        .map(|_| ())
        .map_err(|err| Box::new(err) as Box<dyn Error + Send>)
    }
    async fn update_started_at_db(&self, user_id: i64) -> Result<(), Box<dyn Error + Send>> {
        let now = Local::now().naive_local();
        sqlx::query!(
            r"UPDATE integral_time_card
            SET started_at = $2
            WHERE user_id = $1",
            user_id,
            now
        )
        .execute(&self.state.db)
        .await
        .map(|_| ())
        .map_err(|err| Box::new(err) as Box<dyn Error + Send>)
    }
    async fn update_updated_at_db(&self, user_id: i64) -> Result<(), Box<dyn Error + Send>> {
        let now = Local::now().naive_local();
        sqlx::query!(
            r"UPDATE integral_time_card
            SET updated_at = $2
            WHERE user_id = $1",
            user_id,
            now
        )
        .execute(&self.state.db)
        .await
        .map(|_| ())
        .map_err(|err| Box::new(err) as Box<dyn Error + Send>)
    }
}

enum Cmd {
    Punch,
    Status,
    Derivative,
    Ranking,
}

#[derive(Deserialize)]
struct UserDataExtractor {
    data: UserData,
}

#[derive(Deserialize)]
struct UserData {
    nickname: Option<String>,
    card: Option<String>,
    user_id: Option<i64>,
}
#[derive(Deserialize)]
struct MemberListExtractor {
    data: Vec<UserData>,
}

struct RankingListEntry {
    user_id: i64,
    score: Duration,
}
