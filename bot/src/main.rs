mod config;
mod telegram;

use priconne_core::Error;
use resource::cartoon::CartoonClient;
use scheduler::Action;
use std::sync::Arc;
use teloxide::utils::command::BotCommand;
use teloxide::{
    prelude::{Request, Requester, UpdateWithCx},
    types::{ChatId, InputFile},
};
use tokio::sync::mpsc::{self, UnboundedSender};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .try_init()?;

    run().await
}

#[derive(Debug, BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum TelegramCommand {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "get cartoon by id.", parse_with = "split")]
    Cartoon { id: i32 },
    #[command(description = "send all cartoons.")]
    CartoonAll,
    #[command(description = "send all articles.")]
    ArticleAll,
    #[command(description = "send all news.")]
    NewsAll,
    #[command(description = "stop.")]
    Shutdown,
}

async fn answer(
    cx: UpdateWithCx<teloxide::Bot, teloxide::prelude::Message>,
    command: TelegramCommand,
    tx: Arc<UnboundedSender<Command>>,
) -> Result<(), Error> {
    match command {
        TelegramCommand::Help => {
            cx.answer(TelegramCommand::descriptions()).send().await?;
        }
        TelegramCommand::Cartoon { id } => tx
            .send(Command::Cartoon {
                id,
                chat_id: cx.update.chat_id().into(),
            })
            .map_err(|_| Error::SendError)?,
        TelegramCommand::CartoonAll => {
            tx.send(Command::CartoonAll).map_err(|_| Error::SendError)?
        }
        TelegramCommand::ArticleAll => {
            tx.send(Command::ArticleAll).map_err(|_| Error::SendError)?
        }
        TelegramCommand::NewsAll => tx.send(Command::NewsAll).map_err(|_| Error::SendError)?,
        TelegramCommand::Shutdown => tx.send(Command::Shutdown).map_err(|_| Error::SendError)?,
    };

    Ok(())
}

#[derive(Debug)]
enum Command {
    Cartoon { id: i32, chat_id: ChatId },
    CartoonAll,
    ArticleAll,
    NewsAll,
    Shutdown,
    Log,
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, mut rx) = mpsc::unbounded_channel();

    let config: config::BotConfig = serde_yaml::from_reader(std::fs::File::open("config.yaml")?)?;

    let listener = config.telegram.listener().await;
    let tg = config.telegram.build().await?;
    let _telegraph = config.telegraph.build().await?;
    let client = config.server.build().unwrap();
    let bot = config.build().await?;

    let atx = Arc::new(tx.clone());

    let repl = teloxide::commands_repl_with_listener(
        tg.clone(),
        "priconne-telegram-bot 🦀",
        move |cx, command| answer(cx, command, atx.clone()),
        listener,
    );

    let cartoon_chat = config.telegram.cartoon_chat.clone();
    let information_chat = config.telegram.information_chat.clone();
    let recv = async move {
        while let Some(received) = rx.recv().await {
            match received {
                Command::Cartoon { id, chat_id } => {
                    let cartoon = client.cartoon_detail(id).await?;
                    tg.send_photo(chat_id, InputFile::Url(cartoon.image_src))
                        .send()
                        .await
                        .unwrap();
                }
                Command::CartoonAll => bot.cartoon_all(5, 269, cartoon_chat.clone()).await?,
                Command::ArticleAll => bot.announce_all(5, 1434, information_chat.clone()).await?,
                Command::NewsAll => bot.news_all(5, 1332, information_chat.clone()).await?,
                Command::Shutdown => break,
                Command::Log => log::info!("log"),
            }
        }
        Ok::<(), Error>(())
    };

    let schedule = async move {
        let mut article_action = Action::new(config.schedule.article()?, || {
            tx.clone().send(Command::ArticleAll).unwrap()
        });
        let mut cartoon_action = Action::new(config.schedule.cartoon()?, || {
            tx.clone().send(Command::CartoonAll).unwrap()
        });
        let mut news_action = Action::new(config.schedule.news()?, || {
            tx.clone().send(Command::NewsAll).unwrap()
        });

        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            article_action.tick();
            cartoon_action.tick();
            news_action.tick();
        }

        #[allow(unreachable_code)]
        Ok::<(), cron::error::Error>(())
    };

    tokio::spawn(schedule);
    tokio::spawn(recv);
    repl.await;

    Ok(())
}
