pub mod api;
pub mod news;
pub mod resource;




use futures::StreamExt;

use serde::{Deserialize, Serialize};


use crate::{
    database::PostCollection,
    error::Error,
    insight::{Extractor},
    message::ChatManager,
    resource::{
        cartoon::Thumbnail,
        information::Announce,
        news::News,
        post::{sources::Source, Post},
        update::{ActionBuilder, ResourceFindResult},
    },
};

use self::{
    api::ApiClient,
    news::NewsClient,
    resource::{ResourceClient, ResourceService},
};

/// Resource fetch strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchStrategy {
    /// Stop fetch when continuous posted count is greater than this value.
    pub fuse_limit: i32,
    /// Minimum post id.
    pub ignore_id_lt: i32,
}

impl FetchStrategy {
    pub fn build(&self) -> FetchState<i32> {
        FetchState::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct FetchState<I> {
    pub strategy: FetchStrategy,
    pub fuse_count: I,
}

impl FetchState<i32> {
    pub fn new(strategy: FetchStrategy) -> Self {
        Self {
            strategy,
            fuse_count: 0,
        }
    }

    pub fn keep_going(&mut self, id: i32, is_update: bool) -> bool {
        if !is_update {
            self.fuse_count += 1;
        } else if id >= self.strategy.ignore_id_lt {
            self.fuse_count = 0;
        } else {
            self.fuse_count += 1;
        }

        let result = self.fuse_count < self.strategy.fuse_limit;
        tracing::debug!(
            "id: {}/{}, fuse: {}/{}",
            id,
            self.strategy.ignore_id_lt,
            self.fuse_count,
            self.strategy.fuse_limit
        );
        result
    }

    pub fn should_fetch(&self) -> bool {
        self.fuse_count < self.strategy.fuse_limit
    }
}

// #[derive(Debug)]
pub struct PriconneService {
    // pub client: reqwest::Client,
    pub database: mongodb::Database,
    pub strategy: FetchStrategy,
    pub post_collection: PostCollection,
    pub telegraph: telegraph_rs::Telegraph,
    pub news_service: ResourceService<News, NewsClient>,
    pub information_service: ResourceService<Announce, ApiClient>,
    pub cartoon_service: ResourceService<Thumbnail, ApiClient>,
    pub extractor: Extractor,
    pub chat_manager: ChatManager,
}

impl PriconneService {
    // pub fn new<U: IntoUrl>(
    //     news_server: U,
    //     api_servers: Vec<ApiServer>,) -> Result<PriconneService, Error> {
    //     let client = reqwest::Client::builder()
    //         .user_agent("pcrinfobot-rs/0.0.1alpha Android")
    //         .build()?;

    //     Self::with_client(client, news_server, api_servers)
    // }

    // pub fn with_proxy<U: IntoUrl>(
    //     news_server: U,
    //     api_servers: Vec<ApiServer>,
    //     proxy_scheme: &str,
    // ) -> Result<PriconneService, Error> {
    //     let proxy = reqwest::Proxy::all(proxy_scheme)?;
    //     let client = reqwest::Client::builder()
    //         .proxy(proxy)
    //         .user_agent("pcrinfobot-rs/0.0.1alpha Android")
    //         .build()?;

    //     Self::with_client(client, news_server, api_servers)
    // }

    // pub fn with_client<U: IntoUrl>(
    //     client: reqwest::Client,
    //     news_server: U,
    //     api_servers: Vec<ApiServer>,
    // ) -> Result<Self, Error> {
    //     Ok(Self {
    //         client,
    //         api_server: api_servers.get(0).ok_or(Error::NoApiServer)?.clone(),
    //         api_servers,
    //         news_server: news_server.into_url()?,
    //     })
    // }

    /// Add a new information resource to post collection, extract data and send if needed
    pub async fn add_information(
        &self,
        find_result: ResourceFindResult<Announce>,
        source: Source,
    ) -> Result<(), Error> {
        // TODO: sync missed data
        let resource = find_result.item();
        let post = self
            .post_collection
            .find_resource(resource, &source)
            .await?;

        let action = ActionBuilder::new(&source, &find_result, &post).get_action();
        if action.is_none() {
            return Ok(());
        }

        // ask client to get full article
        // maybe other things like thumbnail for cartoon, todo
        let page = self.information_service.page(resource).await?;

        // extract data
        // TODO: telegraph patch in utils
        let data = self.extractor.extract_post(&page);

        // TODO: wrap to somewhere
        let content = telegraph_rs::dom_to_node(&page.page.content_node).unwrap();
        let content = serde_json::to_string(&content)?;
        let telegraph = self
            .telegraph
            .create_page(&data.title, &content, false)
            .await?;
        let data = data.with_telegraph_url(telegraph.url);

        // generate final message action and execute
        // let post = Post::new(data);
        let post = match post {
            Some(mut post) => {
                post.push(data);
                post
            }
            None => Post::new(data),
        };
        self.post_collection.upsert(&post);

        if action.is_update_only() {
            return Ok(());
        }

        // TODO: use action
        self.chat_manager.send_post(&post).await;

        Ok(())
    }
}

impl FetchStrategy {
    pub const DEFAULT: Self = Self {
        fuse_limit: 5,
        ignore_id_lt: 0,
    };
}
