use async_trait::async_trait;
use bincode::{Decode, Encode};
use derive_more::Display;
use std::{collections::HashMap, str::FromStr};

use api::basic_types::ChatIntId;
use eyre::{bail, ensure};
use image_search::{Arguments, Format};
use log::{debug, error};
use rand::Rng;

use crate::{config::ImagerConfig, error::REPLIED_MESSAGE_NOT_FOUND};
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
    last_format: ImageFormat,
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
            CommandName::Pls | CommandName::Gif => Mode::Random,
            CommandName::Please | CommandName::Gif1 => Mode::Sequential,
        }
    }
}

#[derive(Debug, Default, PartialEq, Encode, Decode, Eq, Copy, Clone)]
enum ImageFormat {
    #[default]
    Pic,
    Gif,
}

impl From<CommandName> for ImageFormat {
    fn from(name: CommandName) -> Self {
        match name {
            CommandName::Pls | CommandName::Please => ImageFormat::Pic,
            CommandName::Gif | CommandName::Gif1 => ImageFormat::Gif,
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
        format: ImageFormat,
        limit: usize,
    ) -> eyre::Result<String> {
        if query.is_empty() {
            ensure!(!data.last_query.is_empty(), "query is empty");
        }
        if (query.is_empty() || data.last_query == query)
            && data.last_format == format
            && !data.last_results.is_empty()
        {
            return Ok(Self::choose_result(data, mode));
        }
        data.last_format = format;
        if !query.is_empty() {
            data.last_query = query.into();
        }
        data.seq_index = 0;
        let args = match format {
            ImageFormat::Pic => Arguments::new(&data.last_query, limit),
            ImageFormat::Gif => Arguments::new(&data.last_query, limit).format(Format::Gif),
        };
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
        format: ImageFormat,
    ) -> eyre::Result<String> {
        if let Some(data) = self.chat_data.get_mut(&chat_id) {
            Self::search(data, query, mode, format, self.config.limit).await
        } else {
            let mut data = SearchData::default();
            let url = Self::search(&mut data, query, mode, format, self.config.limit).await?;
            let _ = self.chat_data.insert(chat_id, data);
            Ok(url)
        }
    }
}

#[derive(Debug, Display, Copy, Clone)]
enum CommandName {
    Please,
    Pls,
    Gif,
    Gif1,
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
            "Gif" => Ok(CommandName::Gif),
            "gif" => Ok(CommandName::Gif),
            "Гиф" => Ok(CommandName::Gif),
            "гиф" => Ok(CommandName::Gif),
            "Gif1" => Ok(CommandName::Gif1),
            "gif1" => Ok(CommandName::Gif1),
            "Гиф1" => Ok(CommandName::Gif1),
            "гиф1" => Ok(CommandName::Gif1),
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
        let (action_sent, url) = tokio::join!(
            comm.send_chat_action(message.chat.id.into(), None, ChatAction::UploadPhoto),
            self.search_data(
                message.chat.id,
                cmd.query().as_str(),
                name.into(),
                name.into()
            )
        );
        let url = url?;
        let mut n = self.config.max_reply_attempts;
        let query = self
            .chat_data
            .get(&message.chat.id)
            .unwrap_or_else(|| {
                panic!("data not found for a search completed just now, message = {message:?}")
            })
            .last_query
            .as_str();
        debug!("result for '{query}': '{url}'");
        let mut reply_id = Some(message.message_id);
        while n > 0 {
            match &action_sent {
                Ok(CommonResponse::Ok(action_sent)) if !action_sent => {
                    error!("upload_image action not sent");
                }
                Err(err) => {
                    error!("failed to send UploadImage action {err}");
                }
                _ => {}
            };
            let result = comm
                .send_photo_url(url.as_str(), message.chat.id.into(), reply_id)
                .await;
            match result {
                Err(err) => error!("failed to send, {err}, retrying..."),
                Ok(CommonResponse::Err(err)) if err.description == REPLIED_MESSAGE_NOT_FOUND => {
                    reply_id = None;
                    continue;
                }
                Ok(CommonResponse::Err(err)) => error!("failed to send, {err}, retrying..."),
                _ => {
                    return Ok(());
                }
            }
            n -= 1;
        }
        bail!(
            "imager failed to send the result after {} consecutive fails, message = {:?}",
            self.config.max_reply_attempts,
            message
        )
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
