use std::error::Error;

use regex::Regex;
use serde::{Deserialize, Serialize};
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
            "\tpunch   打卡",
            "\tstatus  查看状态"
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
        match cmd {
            Cmd::Punch => self.punch(event.user_id.unwrap(), bot).await,
            Cmd::Status => self.status(event.user_id.unwrap(), bot).await,
        }
    }
    fn resolve(msg: String) -> Option<Cmd> {
        let re = Regex::new(r"^>integral\s+(?P<cmd>\S+)\s*$").unwrap();
        let cmd = re.replace_all(&msg, "$cmd").to_string();
        match cmd.as_str() {
            "punch" => Some(Cmd::Punch),
            "status" => Some(Cmd::Status),
            _ => None,
        }
    }
    async fn punch(&self, user_id: i64, bot: &Bot) -> Result<(), Box<dyn Error + Send>> {
        Ok(())
    }
    async fn status(&self, user_id: i64, bot: &Bot) -> Result<(), Box<dyn Error + Send>> {
        Ok(())
    }
    async fn get_healthy_days_db(&self, user_id: i64) -> Result<i64, Box<dyn Error + Send>> {
        Ok(0)
    }
}

enum Cmd {
    Punch,
    Status,
}

#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn db_connect() {
        dotenv::dotenv().ok();
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");
        let pool = SqlitePoolOptions::new().connect(&db_url).await.unwrap();
        sqlx::query!(
            r"INSERT INTO integral_time_card VALUES ($1, $2)",
            0_i64,
            114514_i64
        )
        .execute(&pool)
        .await
        .unwrap();
        let res: i64 = sqlx::query!(
            r"SELECT healthy_days FROM integral_time_card WHERE user_id=$1",
            0_i64
        )
        .fetch_one(&pool)
        .await
        .map(|row| row.healthy_days)
        .unwrap()
        .into();
        assert_eq!(res, 114514);
    }
}
