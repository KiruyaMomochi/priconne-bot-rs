use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct BotConfig {
    pub tags: TaggerConfig,
    pub client: ClientConfig,
    pub mongo: MongoConfig,
    pub server: ServerConfig,
    pub telegram: TelegramConfig,
    pub telegraph: TelegraphConfig,
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
    webhook_url: Option<String>,
    listen_addr: Option<String>,
    token: String,
    information_chat: teloxide::types::ChatId,
    cartoon_chat: teloxide::types::ChatId,
    debug_chat: teloxide::types::ChatId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TelegraphConfig {
    short_name: String,
    access_token: String,
    author_name: Option<String>,
    author_url: Option<String>,
}

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
    pub fn build(self) -> Result<reqwest::Client, Error> {
        let mut client = reqwest::Client::builder().user_agent(self.user_agent);

        if let Some(proxy) = self.proxy {
            let proxy = reqwest::Proxy::all(proxy)?;
            client = client.proxy(proxy);
        }

        let client = client.build()?;

        Ok(client)
    }
}

impl TelegraphConfig {
    pub async fn build(self) -> Result<telegraph_rs::Telegraph, telegraph_rs::Error> {
        let mut telegraph =
            telegraph_rs::Telegraph::new(&self.short_name).access_token(&self.access_token);

        set_some!(telegraph, self => author_name, author_url);

        let telegraph = telegraph.create().await?;
        Ok(telegraph)
    }

    pub async fn with_client(
        self,
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
    pub fn build(self) -> teloxide::Bot {
        let bot = teloxide::Bot::new(self.token);
        bot
    }

    pub fn with_client(self, client: reqwest::Client) -> teloxide::Bot {
        let bot = teloxide::Bot::with_client(self.token, client);
        bot
    }

    pub async fn webhook(
        self,
    ) -> impl teloxide::dispatching::update_listeners::UpdateListener<std::convert::Infallible>
    {
        crate::telegram::listen_webhook(&self.listen_addr.expect("Webhook address not set")).await
    }
}

impl TaggerConfig {
    pub fn build(self) -> Result<crate::message::Tagger, regex::Error> {
        let mut tag_rules = Vec::<(regex::Regex, String)>::new();
        for (tag, regexs) in self.0 {
            for regex in regexs {
                tag_rules.push((regex::Regex::new(&regex)?, tag.to_owned()));
            }
        }
        Ok(crate::message::Tagger { tag_rules })
    }
}

impl ServerConfig {
    pub fn with_client(self, client: reqwest::Client) -> Result<crate::client::Client, Error> {
        if self.api.len() == 0 {
            return Err(Error::NoApiServer);
        }

        crate::client::Client::with_client(self.news, self.api[0].url.to_owned(), client)
    }
}

impl MongoConfig {
    pub async fn build(self) -> Result<mongodb::Client, mongodb::error::Error> {
        mongodb::Client::with_uri_str(self.connection_string).await
    }

    pub async fn database(self) -> Result<mongodb::Database, mongodb::error::Error> {
        let client = mongodb::Client::with_uri_str(self.connection_string).await?;
        Ok(client.database(&self.database))
    }
}

impl BotConfig {
    pub async fn build(self) -> Result<crate::bot::Bot<crate::client::Client>, Error> {
        use teloxide::prelude::RequesterExt;

        let client = self.client.build()?;
        Ok(crate::bot::Bot::<crate::client::Client> {
            chat_id: self.telegram.debug_chat.to_owned(),
            client: self.server.with_client(client.clone())?,
            mongo_database: self.mongo.database().await?,
            telegraph: self.telegraph.with_client(client.clone()).await?,
            bot: self.telegram.with_client(client.clone()).auto_send(),
            tagger: self.tags.build()?,
        })
    }
}
