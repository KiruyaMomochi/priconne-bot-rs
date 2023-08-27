use mongodb::Collection;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use teloxide::{
    dispatching::{
        dialogue::{self, InMemStorage},
        UpdateFilterExt, UpdateHandler,
    },
    payloads::{SendMessageSetters, SendPhotoSetters},
    prelude::Dispatcher,
    requests::{Request, Requester},
    types::{ChatId, InputFile, MessageId, Recipient, Update},
    utils::command::BotCommands,
    Bot,
};

use crate::{config::TelegramConfig, resource::Announcement, Error, PriconneService};

#[derive(Debug, BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum TelegramCommand {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "get cartoon by id.")]
    Cartoon { id: i32 },
    #[command(description = "get news by id.")]
    News { id: i32 },
    #[command(description = "get information by id.")]
    Information { id: i32 },
    #[command(description = "send all cartoons.")]
    CartoonAll,
    #[command(description = "send all articles.")]
    ArticleAll,
    #[command(description = "send all news.")]
    NewsAll,
}

pub struct ChatManager {
    pub bot: teloxide::Bot,
    pub config: TelegramConfig,
    pub post_recipient: Recipient,
    pub cartoon_recipient: Recipient,
    pub messages: Collection<SendResult>,
}

/// Message send result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendResult {
    pub url: Option<Url>,
    pub recipient: Recipient,
    /// Chat id of the recipient, may be the same as `recipient`,
    /// but we store it for a more stable reference
    pub chat_id: ChatId,
    pub message_id: MessageId,
    pub resource_id: crate::resource::ResourceId,
    pub update_time: chrono::DateTime<chrono::Utc>,
}

pub struct Message {
    pub text: String,
    pub silent: bool,
    pub image_src: Option<Url>,
}

pub trait Sendable {
    fn message(&self) -> Message;
}

impl ChatManager {
    async fn send_to<C>(
        &self,
        message: Message,
        chat_id: C,
    ) -> Result<teloxide::prelude::Message, Error>
    where
        C: Into<Recipient> + Clone,
    {
        let send_result = if let Some(image_src) = message.image_src {
            self.bot
                .send_photo(chat_id.clone(), InputFile::url(image_src))
                .caption(message.text)
                .disable_notification(message.silent)
                .parse_mode(teloxide::types::ParseMode::Html)
                .await?
        } else {
            self.bot
                .send_message(chat_id.clone(), message.text.clone())
                .disable_notification(message.silent)
                .parse_mode(teloxide::types::ParseMode::Html)
                .await?
        };

        Ok(send_result)
    }

    pub async fn send_announcement(
        &self,
        post: &Announcement,
    ) -> Result<teloxide::prelude::Message, Error> {
        let message = self
            .send_to(post.message(), self.post_recipient.clone())
            .await?;
        self.messages
            .insert_one(
                SendResult {
                    recipient: self.post_recipient.clone(),
                    chat_id: message.chat.id,
                    resource_id: crate::resource::ResourceId::Announcement(post.id),
                    update_time: message.date,
                    message_id: message.id,
                    url: message.url(),
                },
                None,
            )
            .await?;
        Ok(message)
    }

    pub async fn send_cartoon<M: Sendable>(
        &self,
        cartoon: &M,
    ) -> Result<teloxide::prelude::Message, Error> {
        self.send_to(cartoon.message(), self.cartoon_recipient.clone())
            .await
    }
}

pub fn dispatcher(
    priconne: &PriconneService,
    bot: &teloxide::Bot,
) -> Dispatcher<Bot, Error, teloxide::dispatching::DefaultKey>
// // https://docs.rs/teloxide/latest/teloxide/repls/trait.CommandReplExt.html
// where
//     L: teloxide::update_listeners::UpdateListener + Send + 'a,
//     L::Err: Debug,
{
    use dptree::deps;

    Dispatcher::builder(bot.clone(), schema())
        .dependencies(deps![priconne.clone(), InMemStorage::<()>::new()])
        .enable_ctrlc_handler()
        .build()
}

pub fn schema() -> UpdateHandler<crate::Error> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<TelegramCommand, _>()
        .branch(case![TelegramCommand::Help].endpoint(help))
        .branch(case![TelegramCommand::CartoonAll].endpoint(cartoon_all))
        .branch(case![TelegramCommand::NewsAll].endpoint(news_all))
        .branch(case![TelegramCommand::ArticleAll].endpoint(article_all))
        .branch(dptree::endpoint(todo_command));

    let message_handler = Update::filter_message().branch(command_handler);

    dialogue::enter::<Update, InMemStorage<_>, (), _>().branch(message_handler)
}

async fn help(bot: teloxide::Bot, msg: teloxide::types::Message) -> crate::Result<()> {
    let text = TelegramCommand::descriptions();
    bot.send_message(msg.chat.id, text.to_string())
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;
    Ok(())
}

async fn cartoon_all(priconne: PriconneService) -> crate::Result<()> {
    priconne
        .run_service(crate::resource::ResourceKind::Cartoon)
        .await?;
    Ok(())
}

async fn article_all(priconne: PriconneService) -> crate::Result<()> {
    priconne
        .run_service(crate::resource::ResourceKind::Information)
        .await?;
    Ok(())
}

async fn news_all(priconne: PriconneService) -> crate::Result<()> {
    priconne
        .run_service(crate::resource::ResourceKind::News)
        .await?;
    Ok(())
}

async fn todo_command(
    bot: teloxide::Bot,
    command: TelegramCommand,
    msg: teloxide::types::Message,
) -> crate::Result<()> {
    bot.send_message(
        msg.chat.id,
        format!("Not implemented command `{command:?}`"),
    )
    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use dptree::{prelude::DependencyMap, Endpoint};

    type WebHandler = Endpoint<'static, DependencyMap, String>;

    fn smiles_handler() -> WebHandler {
        dptree::filter(|req: &'static str| req.starts_with("/smile"))
            .endpoint(|| async { "🙃".to_owned() })
    }

    fn sqrt_handler() -> WebHandler {
        dptree::filter_map(|req: &'static str| {
            if req.starts_with("/sqrt") {
                let (_, n) = req.split_once(' ')?;
                n.parse::<f64>().ok()
            } else {
                None
            }
        })
        // .endpoint(|s: &'static str, n: f64| async move { format!("{s} -> {}", n.sqrt()) })
    }

    fn not_found_handler() -> WebHandler {
        dptree::endpoint(|| async { "404 Not Found".to_owned() })
    }

    #[tokio::test]
    async fn dptree_play() {
        let web_server = dptree::entry()
            .branch(smiles_handler())
            .branch(sqrt_handler())
            .branch(not_found_handler());
        let result = web_server.dispatch(dptree::deps!["/sqrt 16"]).await;
        println!("{result:?}")
    }
}
