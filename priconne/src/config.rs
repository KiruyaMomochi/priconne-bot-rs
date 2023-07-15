//! Configuration
//!
//! This module contains configuration for priconne.

use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use teloxide::{requests::Requester, types::Recipient};
use tracing::{event, info, span, Level};

use crate::{
    client::FetchStrategy,
    insight::{tagging::RegexTagger, Extractor},
    message::ChatManager,
    resource::{api::ApiServer, ResourceId},
    service::PriconneService,
};

/// This is useful for setting values in builder.
macro_rules! set_some {
    ($var:ident, $self:expr => $($field:ident),*) => {
        $(
            if let Some($field) = &$self.$field {
                $var = $var.$field($field);
            }
        )*
    };
}

// TODO: In the future we may have some Config traits, but it's an over-engieering for now.
// Because, we need consider circumstances with async or not, and wrap with result or not...

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PriconneConfig {
    /// Tagging rules
    pub tags: TaggerConfig,
    /// Client configuration, such as proxy, user agent, etc.
    pub client: ClientConfig,
    /// MongoDB configuration
    pub mongo: MongoConfig,
    /// Telegram bot configuration
    pub telegram: TelegramConfig,
    /// Telegraph configuration
    pub telegraph: TelegraphConfig,
    pub fetch: FetchConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FetchConfig {
    /// Url endpoins to fetch resources
    pub server: ServerConfig,
    /// Fetch schedule
    pub schedule: HashMap<String, Vec<String>>,
    /// Fetch strategy
    pub strategy: StrategyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MongoConfig {
    connection_string: String,
    database: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ClientConfig {
    user_agent: Option<String>,
    proxy: Option<String>,
    no_proxy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TelegraphConfig {
    short_name: String,
    access_token: String,
    author_name: Option<String>,
    author_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ServerConfig {
    pub news: String,
    pub api: Vec<ApiServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StrategyConfig {
    base: FetchStrategy,
    #[serde(flatten)]
    overrides: HashMap<String, FetchStrategy>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct TaggerConfig(HashMap<String, Vec<String>>);

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TelegramConfig {
    pub webhook_url: Option<String>,
    pub listen_addr: Option<String>,
    pub name: String,
    pub token: String,
    // #[schemars(with = "i64")]
    // pub debug_chat: teloxide::types::ChatId,
    pub recipient: RecipientConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RecipientConfig {
    #[schemars(with = "RemoteRecipient")]
    pub debug: Recipient,
    #[schemars(with = "RemoteRecipient")]
    pub post: Recipient,
    #[schemars(with = "RemoteRecipient")]
    pub cartoon: Recipient,
}

/// A unique identifier for the target chat or username of the target channel
/// (in the format `@channelusername`).
#[derive(JsonSchema)]
#[schemars(untagged)]
pub enum RemoteRecipient {
    /// A chat identifier.
    #[schemars(with = "i32")]
    Id(teloxide::types::ChatId),

    /// A channel username (in the format @channelusername).
    ChannelUsername(String),
}

impl TaggerConfig {
    pub fn build(&self) -> Result<RegexTagger, regex::Error> {
        let mut tag_rules = Vec::<(regex::Regex, String)>::new();
        for (tag, regexs) in &self.0 {
            for regex in regexs {
                tag_rules.push((regex::Regex::new(regex)?, tag.to_owned()));
            }
        }
        Ok(RegexTagger { tag_rules })
    }
}

impl MongoConfig {
    pub async fn client(&self) -> Result<mongodb::Client, mongodb::error::Error> {
        mongodb::Client::with_uri_str(&self.connection_string).await
    }

    pub async fn build(&self) -> Result<mongodb::Database, mongodb::error::Error> {
        let client = mongodb::Client::with_uri_str(&self.connection_string).await?;
        Ok(client.database(&self.database))
    }
}

impl ClientConfig {
    pub fn build(&self) -> Result<reqwest::Client, crate::Error> {
        let mut client =
            reqwest::Client::builder().user_agent(self.user_agent.clone().unwrap_or(crate::ua()));

        if let Some(proxy) = self
            .proxy
            .clone()
            .or_else(|| std::env::var("ALL_PROXY").ok())
        {
            let proxy = reqwest::Proxy::all(proxy)?;
            let proxy = match &self.no_proxy {
                Some(no_proxy) => proxy.no_proxy(reqwest::NoProxy::from_string(no_proxy)),
                None => proxy.no_proxy(reqwest::NoProxy::from_env()),
            };
            info!("Using proxy {:?}", proxy);
            client = client.proxy(proxy);
        }

        let client = client.build()?;

        Ok(client)
    }
}

impl TelegraphConfig {
    pub async fn build(&self) -> Result<telegraph_rs::Telegraph, telegraph_rs::Error> {
        let mut telegraph =
            telegraph_rs::Telegraph::new(&self.short_name).access_token(&self.access_token);

        set_some!(telegraph, self => author_name, author_url);

        let telegraph = telegraph.create().await?;
        Ok(telegraph)
    }

    pub async fn with_client(
        &self,
        client: reqwest::Client,
    ) -> Result<telegraph_rs::Telegraph, telegraph_rs::Error> {
        let mut telegraph = telegraph_rs::Telegraph::new(&self.short_name)
            .access_token(&self.access_token)
            .client(client);

        set_some!(telegraph, self => author_name, author_url);

        let telegraph = telegraph.create().await?;
        Ok(telegraph)
    }
}

impl TelegramConfig {
    pub async fn build(&self) -> Result<teloxide::Bot, crate::Error> {
        let bot = teloxide::Bot::new(self.token.to_owned());
        Ok(bot)
    }

    pub async fn with_client(
        &self,
        client: reqwest::Client,
    ) -> Result<teloxide::Bot, crate::Error> {
        let bot = teloxide::Bot::with_client(self.token.to_owned(), client);
        if let Some(url) = &self.webhook_url {
            let url = reqwest::Url::parse(url)?;
            bot.set_webhook(url).await?;
        }
        Ok(bot)
    }
}

impl StrategyConfig {
    pub fn build_for(&self, resource: &str) -> FetchStrategy {
        // let name = resource.name().to_owned();
        let name = resource;
        let mut result = self.base.clone();
        if let Some(over) = self.overrides.get(name) {
            result = result.override_by(over)
        }

        result
    }
}

impl PriconneConfig {
    pub async fn build(&self) -> Result<PriconneService, crate::Error> {
        let client = self.client.build()?;
        let telegraph = self.telegraph.with_client(client.clone()).await?;
        let database = self.mongo.build().await?;
        let bot = self.telegram.with_client(client.clone()).await?;
        let tagger = self.tags.build()?;
        let extractor = Extractor { tagger };

        Ok(PriconneService {
            telegraph,
            client,
            config: self.fetch.clone(),
            extractor,
            chat_manager: ChatManager {
                bot,
                post_recipient: self.telegram.recipient.post.clone(),
                cartoon_recipient: self.telegram.recipient.cartoon.clone(),
                messages: database.collection("messages"),
            },
            database,
        })
    }
}

#[cfg(test)]
mod tests {
    use teloxide::types::ChatId;

    use super::*;
    use std::fs::File;

    #[test]
    fn test_deserialize_bot_config() {
        let config = File::open("tests/config.yaml").unwrap();
        let bot_config: PriconneConfig = serde_yaml::from_reader(config).unwrap();

        assert_eq!(bot_config.fetch.server.api.len(), 2);
        assert_eq!(bot_config.client.proxy, Some("127.0.0.1:8565".to_string()));
        assert_eq!(
            bot_config.telegram.recipient.debug,
            Recipient::Id(ChatId(0))
        );
        assert_eq!(
            bot_config.telegram.token,
            "123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz".to_string()
        );
        assert_eq!(
            bot_config.telegram.listen_addr,
            Some("127.0.0.1:5555".to_string())
        );
        assert_eq!(
            bot_config.mongo.connection_string,
            "mongodb://localhost:27017".to_string()
        );
        assert_eq!(bot_config.mongo.database, "test".to_string());
        assert_eq!(bot_config.tags.0.len(), 2);
    }

    #[test]
    fn test_build_schedule() {
        let config = File::open("tests/config.yaml").unwrap();
        let bot_config: PriconneConfig = serde_yaml::from_reader(config).unwrap();

        let news_schedule = bot_config
            .fetch
            .schedule
            .get("news")
            .expect("key news")
            .clone();
        let mut schedule = tokio_cron_scheduler::Job::new(news_schedule, |_uuid, _lock| {})
            .expect("create schedule");
        let job_data = schedule.job_data().unwrap();
        assert_eq!(
            job_data.schedule().unwrap().to_string(),
            "* 1 5-23 * * * * | * 1 0,2,4 * * * *".to_string()
        )
    }

    #[tokio::test]
    async fn test_build_priconne() {
        let config = File::open("tests/config.yaml").unwrap();
        let bot_config: PriconneConfig = serde_yaml::from_reader(config).unwrap();
        let priconne = bot_config.build().await.unwrap();
        assert_eq!(
            priconne.chat_manager.post_recipient,
            Recipient::ChannelUsername("@pcrtwstat".to_string())
        )
    }
}
