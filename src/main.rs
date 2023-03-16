mod config;

use archivarius::archivarius::Archivarius;
use bot::bot::{config::BotConfig, Bot, State};
use imager::imager::Imager;
use log::LevelFilter;
use simple_logger::SimpleLogger;
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

    // let config = GlobalConfig::from_file("config.xml").expect("failed to load config");

    let (tx, rx) = mpsc::channel::<State>(1);

    let bot_config = BotConfig {
        skip_missed_updates: false,
        backup_path: "jab.data".into(),
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
                panic!("unable to listen for shutdown signal: {err}");
            }
        };
    });
}
