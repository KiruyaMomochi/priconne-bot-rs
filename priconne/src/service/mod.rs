pub mod api;
pub mod news;
pub mod post;
pub mod resource;

use std::iter;

use futures::StreamExt;

use serde::{Deserialize, Serialize};
use teloxide::types::Recipient;

use crate::{
    database::PostCollection,
    error::Error,
    message::MessageBuilder,
    resource::{
        cartoon::Thumbnail,
        information::Announce,
        news::News,
        post::{sources, Post},
        Resource,
    },
};

use self::{api::ApiClient, news::NewsClient, resource::ResourceService};

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

#[derive(Debug)]
pub enum UpdateEvent {
    News(News),
    Announce(Announce),
    Cartoon(Thumbnail),
}

// #[derive(Debug)]
pub struct PriconneService {
    // pub client: reqwest::Client,
    pub database: mongodb::Database,
    pub strategy: FetchStrategy,
    pub post_collection: PostCollection,
    // pub post_collection: PostCollection,
    pub handler: Box<dyn Fn(UpdateEvent) + Sync + Send>,
    // pub announce_service: ResourceService<Announce, ApiClient>,
    pub telegraph: telegraph_rs::Telegraph,
    pub message_builder: MessageBuilder,
    pub bot: teloxide::Bot,
    pub chat_id: Recipient,
    // pub api: ApiClient,
    // pub news: NewsClient,
    pub news_service: ResourceService<News, NewsClient>,
    pub information_service: ResourceService<Announce, ApiClient>,
    pub cartoon_service: ResourceService<Thumbnail, ApiClient>,
}

enum Action {
    None,
    UpdateOnly,
    Edit,
    Send,
}

impl Action {
    pub fn send(&self) -> bool {
        match self {
            Action::Send => true,
            _ => false,
        }
    }
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

    // fn client(&self) -> &reqwest::Client {
    //     &self.client
    // }

    // pub fn get<U: IntoUrl>(&self, url: U) -> RequestBuilder {
    //     self.client().get(url)
    // }

    // pub async fn handle_new_post(&self, post: PostKind) {
    //     let source = post.into_source();
    //     let found_post = self.post_collection.find(&post.title(), source).await.unwrap();
    //     if let Some(found_post) = found_post {
    //         found_post.update(post);
    //         // send it to log
    //     } else {
    //         self.post_collection.insert(post);
    //         // send it to channel
    //     }
    // }

    fn need_update_send_announce(&self, announce: &Announce, api_id: &str, post: Post) -> Action {
        if post.message_id.is_none() {
            return Action::Send;
        }

        let db_id = match post.source.announce.get(api_id) {
            Some(db_id) => db_id,
            None => return Action::UpdateOnly,
        };

        if *db_id != announce.announce_id {
            return Action::Edit;
        }

        if announce.replace_time > post.update_time {
            return Action::Edit;
        }

        Action::None
    }

    pub async fn send_message(&self) {}

    pub async fn check_announce(&self) -> Result<(), Error> {
        // Fetch new or updated announces
        // Currently, these updated announces are not updated in our database
        let latests = self.information_service.latests().await?;

        // Check in post db
        for announce in latests.iter() {
            let post = self
                .post_collection
                .find_resource(announce, &self.information_service)
                .await?;

            // fetch information?
            // if true, fetch it and upload to telegraph, then update the post
            let mut fetch_information = false;

            // create a post if it doesn't exist
            if post.is_none() {
                // to create a post, we must fetch it first
                // since "fetch or not" depends on post update not exists, this may then changed to a "intent" like "flag"

                
                // send post to channel
                //     let sender = self.message_builder.build_message_announce(
                //     announce,
                //     &information,
                //     api_id.clone(),
                //     &telegraph,
                // );
                // let post = sender.send(self.bot.clone(), self.chat_id.clone()).await?;
                // self.post_collection.upsert(post).await?;
            }

            if fetch_information {

                let (information, node) = self
                    .information_service
                    .client
                    .information(announce.announce_id)
                    .await?;

                // get the telegraph page
                let telegraph = self
                    .telegraph
                    .create_page_doms(&information.title, iter::once(node), false)
                    .await?;
            }

            // update the post by fetching it
            // send message
            // update db
        }

        // Send or not

        // Update announce db

        // Update post db

        Ok(())
    }
}

impl FetchStrategy {
    pub const DEFAULT: Self = Self {
        fuse_limit: 5,
        ignore_id_lt: 0,
    };
}
