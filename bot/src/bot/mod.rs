use crate::{
    bot::{command::BotCommandInfo, config::BotConfig},
    communicator::{Communicate, Communicator},
    connector::{
        polling::{PollingConnector, PollingConnectorConfig},
        webhook::{WebhookConnector, WebhookConnectorConfig},
        Connector, ConnectorMode,
    },
    module::PersistentModule,
    persistence::Persistence,
};
use api::{
    basic_types::UpdateId,
    proto::{Message, Update},
};
use bincode::{Decode, Encode};
use compact_str::{CompactString, ToCompactString};
use eyre::bail;
use futures_util::future::try_join_all;
use log::{debug, error, info, warn};
use std::{
    collections::HashMap,
    io::{Read, Write},
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};
use tokio::sync::mpsc::{error::TryRecvError, Receiver};

pub mod command;
pub mod config;

pub struct Bot {
    last_update_id: UpdateId,
    connector: Box<dyn Connector>,
    communicator: Communicator,
    modules: HashMap<CompactString, BinPersistentModule>,
    work_dir: PathBuf,
    state_rx: Receiver<State>,
    skip_missed_updates: bool,
    data_file_name: CompactString,
}

#[derive(Debug)]
pub enum State {
    Shutdown,
}

type BinPersistentModule = Box<dyn PersistentModule<Input = Vec<u8>, Output = Vec<u8>>>;

impl Bot {
    pub fn new(token: &str, state_rx: Receiver<State>) -> Self {
        Self::with_config(token, state_rx, Default::default())
    }

    pub fn with_config(token: &str, state_rx: Receiver<State>, config: BotConfig) -> Self {
        let connector: Box<dyn Connector> = match config.connector_mode {
            ConnectorMode::Polling => {
                let connector_config = PollingConnectorConfig {
                    allowed_updates: config.allowed_updates.into_iter().collect(),
                    limit: config.update_limit,
                    timeout: config.polling_timeout,
                    drop_pending_updates: config.skip_missed_updates,
                };
                Box::new(PollingConnector::with_config(token, connector_config))
            }
            ConnectorMode::Webhook => {
                let ip_address = dotenv::var("IP_V4_ADDR").unwrap().to_compact_string();
                let connector_config = WebhookConnectorConfig {
                    https_url: Some(ip_address.clone()),
                    ip_address: Some(ip_address),
                    drop_pending_updates: config.skip_missed_updates,
                    allowed_updates: config.allowed_updates.into_iter().collect(),
                    ..Default::default()
                };
                Box::new(WebhookConnector::with_config(token, connector_config))
            }
        };

        Self {
            connector,
            communicator: Communicator::new(token),
            last_update_id: 0,
            modules: Default::default(),
            work_dir: config.work_dir,
            state_rx,
            skip_missed_updates: config.skip_missed_updates,
            data_file_name: config.data_file_name,
        }
    }

    /// Each module has to have a unique name
    pub fn add_module(
        &mut self,
        name: &str,
        module: impl PersistentModule<Output = Vec<u8>, Input = Vec<u8>> + 'static,
    ) {
        if self.modules.contains_key(name) {
            error!("failed to insert '{name}' as the module with that name is present already");
        } else {
            self.modules.insert(name.into(), Box::new(module));
        }
    }

    async fn handle_message_update(&mut self, message: Message) -> eyre::Result<()> {
        let Ok(cmd) = BotCommandInfo::try_from(&message) else {
            return Ok(());
        };

        match JabCommandName::from_str(cmd.name()) {
            Ok(name) => {
                match name {
                    JabCommandName::Del => {
                        self.communicator.del(&message).await?;
                    }
                };
                return Ok(());
            }
            Err(err) => {
                debug!("{err}");
            }
        };

        try_join_all(
            self.modules
                .values_mut()
                .map(|m| m.try_execute_command(&self.communicator, &cmd, &message)),
        )
        .await?;

        Ok(())
    }

    fn check_is_old_update(&mut self, id: UpdateId) -> bool {
        if self.last_update_id >= id {
            true
        } else if self.last_update_id != 0 && self.last_update_id < id - 1 {
            error!(
                "some updates skipped! last update id = {}, new update id = {}",
                self.last_update_id, id
            );
            self.last_update_id = id;
            false
        } else {
            self.last_update_id = id;
            false
        }
    }

    pub fn comm(&self) -> &dyn Communicate {
        &self.communicator
    }

    fn load_data(&mut self) -> eyre::Result<()> {
        let path = self.work_dir.join(Path::new(&self.data_file_name));
        let mut file = std::fs::File::options().read(true).open(path.as_path())?;
        let metadata = std::fs::metadata(path)?;
        let mut buffer = vec![0; metadata.len() as usize];
        file.read_exact(&mut buffer)?;
        self.deserialize(buffer)?;
        Ok(())
    }

    fn save_data(&self) -> eyre::Result<()> {
        let data = self.serialize()?;
        let path = self.work_dir.join(Path::new(&self.data_file_name));
        let mut file = std::fs::File::options()
            .write(true)
            .create(true)
            .open(path)?;
        file.write_all(data.as_slice())?;
        Ok(())
    }

    pub async fn start(mut self) {
        self.load_data().unwrap_or_else(|err| {
            error!(
                "failed to load bot data, path = {:?}, {}",
                self.work_dir, err
            )
        });

        self.connector
            .on_startup()
            .await
            .expect("connector failed on startup");

        let mut interval = tokio::time::interval(Duration::from_millis(1000));

        loop {
            match self.state_rx.try_recv() {
                Err(TryRecvError::Disconnected) => {
                    panic!("bot signal channel died");
                }
                Ok(State::Shutdown) => {
                    info!("shutdown signal received, saving bot data..");
                    if let Err(err) = self.save_data() {
                        error!("failed to save bot data, {err}");
                    }
                    return;
                }
                _ => {}
            };

            let updates = match self.connector.fetch_updates().await {
                Ok(updates) => updates,
                Err(err) => {
                    error!("{err}");
                    continue;
                }
            };

            if updates.is_empty() {
                continue;
            }

            if self.last_update_id == 0 && self.skip_missed_updates {
                self.last_update_id = updates.into_iter().map(|u| u.id).max().unwrap();
                continue;
            }

            for update in updates {
                if self.check_is_old_update(update.id) {
                    continue;
                }
                debug!("update received: {update:?}");
                match update.data {
                    Update::MessageUpdate(message) => {
                        if let Err(report) = self.handle_message_update(message).await {
                            error!("{}", report);
                        }
                    }
                    _ => {}
                };
            }

            interval.tick().await;
        }
    }
}

enum JabCommandName {
    Del,
}

impl FromStr for JabCommandName {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "del" => Ok(JabCommandName::Del),
            _ => {
                bail!("jab failed to recognize '{s}' as a possible command");
            }
        }
    }
}

#[derive(Encode, Decode)]
struct PersistenceData {
    modules: HashMap<String, Vec<u8>>,
    last_update_id: UpdateId,
}

impl Persistence for Bot {
    type Input = Vec<u8>;
    type Output = Vec<u8>;

    fn serialize(&self) -> eyre::Result<Self::Output> {
        let mut modules = HashMap::new();
        for (name, module) in &self.modules {
            modules.insert(name.to_string(), module.serialize()?);
        }

        let data = PersistenceData {
            modules,
            last_update_id: self.last_update_id,
        };

        Ok(bincode::encode_to_vec(data, bincode::config::standard())?)
    }

    fn deserialize(&mut self, input: Self::Input) -> eyre::Result<()> {
        let data = bincode::decode_from_slice::<PersistenceData, _>(
            input.as_slice(),
            bincode::config::standard(),
        )?
        .0;

        self.last_update_id = data.last_update_id;

        for (input_name, input_data) in data.modules {
            if let Some(module) = self.modules.get_mut(input_name.as_str()) {
                module.deserialize(input_data)?;
            } else {
                warn!("loaded '{input_name}' data, but the module itself is not present");
            }
        }

        Ok(())
    }
}
