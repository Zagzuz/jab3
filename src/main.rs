mod config;

use crate::config::GlobalConfig;
use archivarius::archivarius::Archivarius;
use bot::bot::{config::BotConfig, Bot, State};
use imager::imager::Imager;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::path::Path;
use tokio::{signal, sync::mpsc};

#[tokio::main]
async fn main() {
    let token = dotenv::var("TOKEN").expect("no token in env");

    SimpleLogger::new()
        .with_level(LevelFilter::Off)
        .with_module_level("jab3", LevelFilter::Debug)
        .with_module_level("bot", LevelFilter::Debug)
        .with_module_level("api", LevelFilter::Debug)
        .with_module_level("imager", LevelFilter::Debug)
        .with_module_level("birthminder", LevelFilter::Debug)
        .with_module_level("archivarius", LevelFilter::Debug)
        .init()
        .expect("logger failure");

    let work_dir = dotenv::var("WORK_DIR").unwrap();
    let path = Path::new(work_dir.as_str()).join(Path::new("config.xml"));

    let config = GlobalConfig::from_file(path.as_path()).expect("failed to load config");

    let (tx, rx) = mpsc::channel::<State>(1);

    let bot_config = BotConfig {
        skip_missed_updates: false,
        connector_mode: config.connector_mode,
        allowed_updates: Default::default(),
        update_limit: None,
        polling_timeout: None,
        work_dir: path,
        data_file_name: config.data_file_name,
        ..Default::default()
    };
    let mut bot = Bot::with_config(token.as_str(), rx, bot_config);

    bot.add_module("imager", Imager::new());
    bot.add_module("archivarius", Archivarius::new());
    // bot.add_module("birthminder", Birthminder::new());

    tokio::join!(bot.start(), async {
        match signal::ctrl_c().await {
            Ok(()) => {
                tx.send(State::Shutdown)
                    .await
                    .expect("failed to send shutdown signal");
            }
            Err(err) => {
                panic!("unable to listen for shutdown signal: {}", err);
            }
        };
    });
}
