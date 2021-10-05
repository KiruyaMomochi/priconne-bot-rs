use std::{collections::HashMap, str::FromStr};

use cron::Schedule;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use teloxide::prelude::{Request, Requester};

use crate::{error::Error, schedule::Schedules};

#[derive(Debug, Serialize, Deserialize)]
pub struct BotConfig {
    pub tags: TaggerConfig,
    pub client: ClientConfig,
    pub mongo: MongoConfig,
    pub server: ServerConfig,
    pub telegram: TelegramConfig,
    pub telegraph: TelegraphConfig,
    pub schedule: ScheduleConfig,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TaggerConfig(HashMap<String, Vec<String>>);

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientConfig {
    user_agent: String,
    proxy: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MongoConfig {
    connection_string: String,
    database: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    news: String,
    api: Vec<ApiServerConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiServerConfig {
    id: String,
    url: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TelegramConfig {
    pub webhook_url: Option<String>,
    pub listen_addr: Option<String>,
    pub token: String,
    pub information_chat: teloxide::types::ChatId,
    pub cartoon_chat: teloxide::types::ChatId,
    pub debug_chat: teloxide::types::ChatId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TelegraphConfig {
    short_name: String,
    access_token: String,
    author_name: Option<String>,
    author_url: Option<String>,
}

macro_rules! schedule_config {
    ($($var:ident),*) => {
        #[derive(Debug, Serialize, Deserialize)]
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

schedule_config! {article, cartoon, news}

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
    pub fn build(&self) -> Result<crate::message::Tagger, regex::Error> {
        let mut tag_rules = Vec::<(regex::Regex, String)>::new();
        for (tag, regexs) in &self.0 {
            for regex in regexs {
                tag_rules.push((regex::Regex::new(regex)?, tag.to_owned()));
            }
        }
        Ok(crate::message::Tagger { tag_rules })
    }
}

impl ServerConfig {
    pub fn build(&self) -> Result<crate::client::Client, Error> {
        if self.api.len() == 0 {
            return Err(Error::NoApiServer);
        }

        crate::client::Client::new(self.news.to_owned(), self.api[0].url.to_owned())
    }

    pub fn with_client(&self, client: reqwest::Client) -> Result<crate::client::Client, Error> {
        if self.api.len() == 0 {
            return Err(Error::NoApiServer);
        }

        crate::client::Client::with_client(self.news.to_owned(), self.api[0].url.to_owned(), client)
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
    pub async fn build(&self) -> Result<crate::bot::Bot<crate::client::Client>, Error> {
        use teloxide::prelude::RequesterExt;

        let client = self.client.build()?;
        Ok(crate::bot::Bot::<crate::client::Client> {
            client: self.server.with_client(client.clone())?,
            mongo_database: self.mongo.database().await?,
            telegraph: self.telegraph.with_client(client.clone()).await?,
            bot: self.telegram.with_client(client.clone()).await?.auto_send(),
            tagger: self.tags.build()?,
        })
    }
}
