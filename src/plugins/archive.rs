use crate::models::{CQEvent, Plugin};

pub struct ArchivePlugin;

impl ArchivePlugin {
    pub async fn archive(event: CQEvent) {}
}

impl Plugin for ArchivePlugin {
    fn name(&self) ->  & 'static str {
        "archive"
    }
    fn help(&self) ->  & 'static str {
        "自动复读已撤回的消息"
    }
}
