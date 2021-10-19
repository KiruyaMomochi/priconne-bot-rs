use std::{collections::HashMap, str::FromStr};

use cron::Schedule;
use priconne_core::{Client, Error, Tagger};
use reqwest::Url;
use resource::Bot;
use scheduler::Schedules;
use serde::{Deserialize, Serialize};
use teloxide::prelude::{Request, Requester};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    pub tags: TaggerConfig,
    pub client: ClientConfig,
    pub mongo: MongoConfig,
    pub server: ServerConfig,
    pub telegram: TelegramConfig,
    pub telegraph: TelegraphConfig,
    pub resources: ResourceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TaggerConfig(HashMap<String, Vec<String>>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    user_agent: String,
    proxy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoConfig {
    connection_string: String,
    database: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    news: String,
    api: Vec<ApiServerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiServerConfig {
    id: String,
    url: String,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    pub webhook_url: Option<String>,
    pub listen_addr: Option<String>,
    pub token: String,
    pub debug_chat: teloxide::types::ChatId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegraphConfig {
    short_name: String,
    access_token: String,
    author_name: Option<String>,
    author_url: Option<String>,
}

macro_rules! schedule_config {
    ($($var:ident),*) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct ScheduleConfig {
            $(
                $var: Vec<String>,
            )*
        }

        impl ScheduleConfig {
            $(
                pub fn $var(&self) -> Result<Schedules, cron::error::Error> {
                    let schedules: Result<Vec<Schedule>, _> = self
                        .$var
                        .iter()
                        .map(|x| cron::Schedule::from_str(&x))
                        .collect();
                    Ok(Schedules::new(schedules?))
                }
            )*
        }
    }
}

macro_rules! article_resource_config {
    ($name:ident) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct $name {
            pub schedules: Schedules,
            pub chat: teloxide::types::ChatId,
            pub min: i32,
            pub limit: i32,
        }
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub information: InformationResourceConfig,
    pub cartoon: CartoonResourceConfig,
    pub news: NewsResourceConfig,
}

article_resource_config!(InformationResourceConfig);
article_resource_config!(CartoonResourceConfig);
article_resource_config!(NewsResourceConfig);

macro_rules! set_some {
    ($var:ident, $self:expr => $($field:ident),*) => {
        $(
            if let Some($field) = &$self.$field {
                $var = $var.$field($field);
            }
        )*
    };
}

impl ClientConfig {
    pub fn build(&self) -> Result<reqwest::Client, Error> {
        let mut client = reqwest::Client::builder().user_agent(&self.user_agent);

        if let Some(proxy) = &self.proxy {
            let proxy = reqwest::Proxy::all(proxy)?;
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
    pub async fn build(&self) -> Result<teloxide::Bot, Error> {
        let bot = teloxide::Bot::new(self.token.to_owned());
        if let Some(url) = &self.webhook_url {
            let url = Url::parse(&url)?;
            bot.set_webhook(url).send().await?;
        }
        Ok(bot)
    }

    pub async fn with_client(&self, client: reqwest::Client) -> Result<teloxide::Bot, Error> {
        let bot = teloxide::Bot::with_client(self.token.to_owned(), client);
        if let Some(url) = &self.webhook_url {
            let url = Url::parse(&url)?;
            bot.set_webhook(url).send().await?;
        }
        Ok(bot)
    }

    pub async fn listener(
        &self,
    ) -> impl teloxide::dispatching::update_listeners::UpdateListener<std::convert::Infallible>
    {
        let listen_addr = self.listen_addr.as_ref().expect("Webhook address not set");
        crate::telegram::listen_webhook(&listen_addr).await
    }
}

impl TaggerConfig {
    pub fn build(&self) -> Result<Tagger, regex::Error> {
        let mut tag_rules = Vec::<(regex::Regex, String)>::new();
        for (tag, regexs) in &self.0 {
            for regex in regexs {
                tag_rules.push((regex::Regex::new(regex)?, tag.to_owned()));
            }
        }
        Ok(Tagger { tag_rules })
    }
}

impl ServerConfig {
    pub fn build(&self) -> Result<Client, Error> {
        if self.api.len() == 0 {
            return Err(Error::NoApiServer);
        }

        Client::new(self.news.to_owned(), self.api[0].url.to_owned())
    }

    pub fn with_client(&self, client: reqwest::Client) -> Result<Client, Error> {
        if self.api.len() == 0 {
            return Err(Error::NoApiServer);
        }

        Client::with_client(self.news.to_owned(), self.api[0].url.to_owned(), client)
    }
}

impl MongoConfig {
    pub async fn build(&self) -> Result<mongodb::Client, mongodb::error::Error> {
        mongodb::Client::with_uri_str(&self.connection_string).await
    }

    pub async fn database(&self) -> Result<mongodb::Database, mongodb::error::Error> {
        let client = mongodb::Client::with_uri_str(&self.connection_string).await?;
        Ok(client.database(&self.database))
    }
}

impl BotConfig {
    pub async fn build(&self) -> Result<Bot<Client>, Error> {
        use teloxide::prelude::RequesterExt;

        let client = self.client.build()?;
        Ok(Bot::<Client> {
            client: self.server.with_client(client.clone())?,
            mongo_database: self.mongo.database().await?,
            telegraph: self.telegraph.with_client(client.clone()).await?,
            bot: self.telegram.with_client(client.clone()).await?.auto_send(),
            tagger: self.tags.build()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use super::*;

    #[test]
    fn test_deserialize_bot_config() {
        let config = File::open("tests/config.yaml").unwrap();
        let bot_config: BotConfig = serde_yaml::from_reader(config).unwrap();

        assert_eq!(bot_config.server.api.len(), 5);
        assert_eq!(bot_config.client.proxy, Some("127.0.0.1:8565".to_string()));
        assert_eq!(bot_config.telegram.webhook_url, Some("https://example.com/webhook".to_string()));
        assert_eq!(bot_config.telegram.token, "123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz".to_string());
        assert_eq!(bot_config.telegram.listen_addr, Some("127.0.0.1:5555".to_string()));
        assert_eq!(bot_config.mongo.connection_string, "mongodb://localhost:27017".to_string());
        assert_eq!(bot_config.mongo.database, "test".to_string());
        assert_eq!(bot_config.tags.0.len(), 2);
    }
}
