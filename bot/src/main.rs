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
    #[command(description = "get news by id.", parse_with = "split")]
    News { id: i32 },
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
        TelegramCommand::News { id } => tx
            .send(Command::News {
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
    News { id: i32, chat_id: ChatId },
    CartoonAll,
    ArticleAll,
    NewsAll,
    Shutdown,
    Log,
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config: config::BotConfig = serde_yaml::from_reader(std::fs::File::open("config.yaml")?)?;
    let cartoon = config.resources.cartoon.clone();
    let information = config.resources.information.clone();
    let news = config.resources.news.clone();

    let client = config.server.build().unwrap();

    let bot = config.build().await?;
    let (tx, mut rx) = mpsc::unbounded_channel();
    let atx = Arc::new(tx.clone());
    let telegram = config.telegram.clone();
    let repl = telegram.repl(move |cx, command| answer(cx, command, atx.clone()));

    let telegram = config.telegram.clone();
    let debug_chat = telegram.debug_chat.clone();

    let recv = async move {
        while let Some(received) = rx.recv().await {
            let result = match &received {
                Command::Cartoon { id, chat_id } => match client.cartoon_detail(*id).await {
                    Ok(cartoon) => telegram
                        .build()
                        .await?
                        .send_photo(chat_id.clone(), InputFile::Url(cartoon.image_src))
                        .send()
                        .await
                        .map_or_else(|error| Err(Error::from(error)), |_| Ok(())),
                    Err(error) => Err(error),
                },
                Command::News { id, chat_id } => bot
                    .news_by_id(*id, chat_id.clone())
                    .await
                    .map_or_else(|error| Err(Error::from(error)), |_| Ok(())),
                Command::CartoonAll => {
                    bot.cartoon_all(cartoon.limit, cartoon.min, cartoon.chat.clone())
                        .await
                }
                Command::ArticleAll => {
                    bot.announce_all(information.limit, information.min, information.chat.clone())
                        .await
                }
                Command::NewsAll => bot.news_all(news.limit, news.min, news.chat.clone()).await,
                Command::Shutdown => Ok(rx.close()),
                Command::Log => Ok(log::info!("log")),
            };
            if let Err(error) = result {
                telegram
                    .build()
                    .await?
                    .send_message(
                        debug_chat.clone(),
                        format!("Error {} occurs in command {:?}", error, received),
                    )
                    .send()
                    .await?;
            }
        }
        Ok::<(), Error>(())
    };

    let schedule = async move {
        let mut article_action =
            Action::new_local(config.resources.information.schedules.clone(), || {
                tx.clone().send(Command::ArticleAll).unwrap()
            });
        let mut cartoon_action = Action::new_local(config.resources.cartoon.schedules.clone(), || {
            tx.clone().send(Command::CartoonAll).unwrap()
        });
        let mut news_action = Action::new_local(config.resources.news.schedules.clone(), || {
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
    repl.await?;

    Ok(())
}
