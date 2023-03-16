use async_trait::async_trait;
use bincode::{Decode, Encode};
use derive_more::Display;
use std::{collections::HashMap, str::FromStr};

use api::basic_types::ChatIntId;
use eyre::{bail, ensure};
use image_search::Arguments;
use log::{debug, error};
use rand::Rng;

use crate::config::ImagerConfig;
use api::{
    proto::{ChatAction, Message},
    response::CommonResponse,
};
use bot::{
    bot::command::BotCommandInfo,
    communicator::Communicate,
    module::{Module, PersistentModule},
    persistence::Persistence,
};

#[derive(Debug, Default)]
pub struct Imager {
    chat_data: ChatData,
    config: ImagerConfig,
}

type ChatData = HashMap<ChatIntId, SearchData>;

#[derive(Debug, Encode, Decode, Default)]
pub struct SearchData {
    last_query: String,
    last_results: Vec<String>,
    seq_index: usize,
    rand_index: usize,
}

#[derive(Debug, Copy, Clone)]
enum Mode {
    Random,
    Sequential,
}

impl From<CommandName> for Mode {
    fn from(name: CommandName) -> Self {
        match name {
            CommandName::Pls => Mode::Random,
            CommandName::Please => Mode::Sequential,
        }
    }
}

impl Imager {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_with_config(config: ImagerConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    fn choose_result(data: &mut SearchData, mode: Mode) -> String {
        match mode {
            Mode::Random => {
                /*data.rand_index = loop {
                    let index = rand::thread_rng().gen_range(0..data.last_results.len());
                    if data.last_results.len() < 3 || data.rand_index != index {
                        break index;
                    }
                };*/
                data.rand_index = rand::thread_rng().gen_range(0..data.last_results.len());
                data.last_results[data.rand_index].clone()
            }
            Mode::Sequential => {
                if data.seq_index >= data.last_results.len() {
                    data.seq_index = 0;
                }
                let result = data.last_results[data.seq_index].clone();
                data.seq_index += 1;
                result
            }
        }
    }

    async fn search(
        data: &mut SearchData,
        query: &str,
        mode: Mode,
        limit: usize,
    ) -> eyre::Result<String> {
        if query.is_empty() {
            ensure!(!data.last_query.is_empty(), "query is empty");
        } else if data.last_query != query {
            data.last_query = query.into();
            data.seq_index = 0;
        }
        let args = Arguments::new(&data.last_query, limit);
        data.last_results = image_search::urls(args.clone()).await?;
        ensure!(!data.last_results.is_empty(), "no results");
        let url = Self::choose_result(data, mode);
        Ok(url)
    }

    async fn search_data(
        &mut self,
        chat_id: ChatIntId,
        query: &str,
        mode: Mode,
    ) -> eyre::Result<String> {
        if let Some(data) = self.chat_data.get_mut(&chat_id) {
            Self::search(data, query, mode, self.config.limit).await
        } else {
            let mut data = SearchData::default();
            let url = Self::search(&mut data, query, Mode::Random, self.config.limit).await?;
            let _ = self.chat_data.insert(chat_id, data);
            Ok(url)
        }
    }
}

#[derive(Debug, Display, Copy, Clone)]
enum CommandName {
    Please,
    Pls,
}

impl FromStr for CommandName {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "please" => Ok(CommandName::Please),
            "Please" => Ok(CommandName::Please),
            "плис" => Ok(CommandName::Please),
            "Плис" => Ok(CommandName::Please),
            "плиз" => Ok(CommandName::Please),
            "Плиз" => Ok(CommandName::Please),
            "Пж" => Ok(CommandName::Please),
            "пж" => Ok(CommandName::Please),
            "Pls" => Ok(CommandName::Pls),
            "pls" => Ok(CommandName::Pls),
            "плс" => Ok(CommandName::Pls),
            "Плс" => Ok(CommandName::Pls),
            "плз" => Ok(CommandName::Pls),
            "Плз" => Ok(CommandName::Pls),
            _ => {
                bail!("failed to recognize '{s}' as a possible command")
            }
        }
    }
}

#[async_trait]
impl Module for Imager {
    async fn try_execute_command(
        &mut self,
        comm: &dyn Communicate,
        cmd: &BotCommandInfo,
        message: &Message,
    ) -> eyre::Result<()> {
        let name = match CommandName::from_str(cmd.name().as_str()) {
            Ok(name) => name,
            Err(err) => {
                debug!("{err}");
                return Ok(());
            }
        };
        loop {
            let (action_sent, url) = tokio::join!(
                comm.send_chat_action(message.chat.id.into(), None, ChatAction::UploadPhoto),
                self.search_data(message.chat.id, cmd.query().as_str(), name.into())
            );
            let url = url?;
            debug!("result for '{}': '{}'", cmd.query(), url);
            match action_sent {
                Ok(CommonResponse::Ok(action_sent)) if !action_sent => {
                    error!("upload_image action not sent");
                }
                Err(err) => {
                    error!("failed to send UploadImage action {err}");
                }
                _ => {}
            };
            let result = comm
                .reply_photo_url(url.as_str(), message.chat.id.into(), message.message_id)
                .await;
            match result {
                Err(err) => error!("failed to send, {err}, retrying..."),
                Ok(CommonResponse::Err(err)) => error!("failed to send, {err}, retrying..."),
                _ => break,
            }
        }
        Ok(())
    }
}

impl Persistence for Imager {
    type Input = Vec<u8>;
    type Output = Vec<u8>;

    fn serialize(&self) -> eyre::Result<Self::Output> {
        Ok(bincode::encode_to_vec(
            &self.chat_data,
            bincode::config::standard(),
        )?)
    }

    fn deserialize(&mut self, input: Self::Input) -> eyre::Result<()>
        where
            Self: Sized,
    {
        self.chat_data = bincode::decode_from_slice::<ChatData, _>(
            input.as_slice(),
            bincode::config::standard(),
        )?
            .0;
        Ok(())
    }
}

impl PersistentModule for Imager {}

#[cfg(test)]
mod test {
    use crate::imager::CommandName;
    use std::str::FromStr;

    #[test]
    fn please_from_str() {
        let cmd = CommandName::from_str("please").unwrap();
        assert!(matches!(cmd, CommandName::Please));
        let cmd = CommandName::from_str("Please").unwrap();
        assert!(matches!(cmd, CommandName::Please));
        let cmd = CommandName::from_str("плис").unwrap();
        assert!(matches!(cmd, CommandName::Please));
        let cmd = CommandName::from_str("Плис").unwrap();
        assert!(matches!(cmd, CommandName::Please));
        let cmd = CommandName::from_str("плиз").unwrap();
        assert!(matches!(cmd, CommandName::Please));
        let cmd = CommandName::from_str("Плиз").unwrap();
        assert!(matches!(cmd, CommandName::Please));
        let cmd = CommandName::from_str("Пж").unwrap();
        assert!(matches!(cmd, CommandName::Please));
        let cmd = CommandName::from_str("пж").unwrap();
        assert!(matches!(cmd, CommandName::Please));
    }
}
