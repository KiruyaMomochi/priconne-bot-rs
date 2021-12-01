use reqwest::{IntoUrl, RequestBuilder, Url};

use crate::error::Error;

#[derive(Debug, Clone)]
pub struct Client {
    client: reqwest::Client,
    pub api_server: Url,
    pub news_server: Url,
}

impl Client {
    pub fn new<U: IntoUrl>(news_server: U, information_server: U) -> Result<Client, Error> {
        let client = reqwest::Client::builder()
            .user_agent("pcrinfobot-rs/0.0.1alpha Android")
            .build()?;

        Self::with_client(news_server, information_server, client)
    }

    pub fn with_proxy<U: IntoUrl>(
        news_server: U,
        information_server: U,
        proxy_scheme: &str,
    ) -> Result<Client, Error> {
        let proxy = reqwest::Proxy::all(proxy_scheme)?;
        let client = reqwest::Client::builder()
            .proxy(proxy)
            .user_agent("pcrinfobot-rs/0.0.1alpha Android")
            .build()?;

        Self::with_client(news_server, information_server, client)
    }

    pub fn with_client<U: IntoUrl>(
        news_server: U,
        api_server: U,
        client: reqwest::Client,
    ) -> Result<Self, Error> {
        Ok(Self {
            client,
            api_server: api_server.into_url()?,
            news_server: news_server.into_url()?,
        })
    }

    fn client(&self) -> &reqwest::Client {
        &self.client
    }

    pub fn get<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.client().get(url)
    }

    pub fn api_server(&self) -> &Url {
        &self.api_server
    }
}
