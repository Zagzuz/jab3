use crate::{bot::command::BotCommandInfo, communicator::Communicate, persistence::Persistence};
use api::proto::Message;
use async_trait::async_trait;

#[async_trait]
pub trait Module {
    async fn try_execute_command(
        &mut self,
        comm: &dyn Communicate,
        cmd: &BotCommandInfo,
        message: &Message,
    ) -> eyre::Result<()>;
}

pub trait PersistentModule: Module + Persistence + Send {}
