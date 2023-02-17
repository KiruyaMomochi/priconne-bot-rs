pub mod api;
pub mod config;
pub mod news;
pub mod resource;
pub mod update;

use std::fmt::Debug;

use chrono::{TimeZone, Utc};
use futures::StreamExt;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::{debug, trace};

use crate::{
    database::{Post, PostCollection},
    error::Error,
    insight::{tagging::RegexTagger, Extractor},
    message::ChatManager,
    resource::{
        cartoon::Thumbnail,
        information::Announce,
        news::News,
        post::{sources::Source, PostPageResponse},
        Resource, ResourceMetadata,
    },
    utils,
};

use update::{ActionBuilder, ResourceFindResult};

use self::{
    api::ApiClient,
    config::{FetchConfig, ServerConfig, StrategyConfig},
    news::NewsClient,
    resource::{ResourceClient, ResourceService},
};

/// Resource fetch strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchStrategy {
    /// Stop fetch when continuous posted count is greater than this value.
    pub fuse_limit: Option<i32>,
    /// Minimum post id.
    pub ignore_id_lt: Option<i32>,
    /// Minimum update time,
    pub ignore_time_lt: Option<chrono::DateTime<chrono::Utc>>,
}

impl FetchStrategy {
    pub fn build(&self) -> FetchState<i32> {
        FetchState::new(self.clone())
    }
    pub fn override_by(self, rhs: &Self) -> Self {
        Self {
            fuse_limit: rhs.fuse_limit.or(self.fuse_limit),
            ignore_id_lt: rhs.ignore_id_lt.or(self.ignore_id_lt),
            ignore_time_lt: rhs.ignore_time_lt.or(self.ignore_time_lt),
        }
    }
}

impl FetchStrategy {
    pub const DEFAULT: Self = Self {
        fuse_limit: Some(1),
        ignore_id_lt: None,
        ignore_time_lt: None,
    };
}

impl Default for FetchStrategy {
    fn default() -> Self {
        Self::DEFAULT
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

    pub fn keep_going<R: ResourceMetadata<IdType = i32>>(
        &mut self,
        resource: &R,
        is_update: bool,
    ) -> bool {
        let id = resource.id();
        let update_time = resource.update_time();

        let mut keep_going = true;
        if let Some(ignore_id_lt) = self.strategy.ignore_id_lt {
            if id < ignore_id_lt {
                keep_going = false;
            }
        }
        if let Some(ignore_time_lt) = self.strategy.ignore_time_lt {
            if update_time < ignore_time_lt {
                keep_going = false;
            }
        }
        if self.strategy.fuse_limit.is_none() {
            return keep_going;
        }

        if !is_update {
            self.fuse_count += 1;
        }
        if keep_going {
            self.fuse_count = 0;
        } else {
            self.fuse_count += 1;
        }

        let result = self.fuse_count < self.strategy.fuse_limit.unwrap_or(0);
        tracing::debug!(
            "id: {}/{:?}, fuse: {}/{:?}",
            id,
            self.strategy.ignore_id_lt,
            self.fuse_count,
            self.strategy.fuse_limit
        );
        result
    }

    pub fn should_fetch(&self) -> bool {
        match self.strategy.fuse_limit {
            Some(fuse_limit) => self.fuse_count < self.fuse_count,
            None => true,
        }
    }
}

// #[derive(Debug)]
pub struct PriconneService {
    // pub client: reqwest::Client,
    pub database: mongodb::Database,
    pub post_collection: PostCollection,
    pub telegraph: telegraph_rs::Telegraph,
    // Alternative implementation:
    // pub news_service: ResourceService<News, NewsClient>,
    // pub information_service: ResourceService<Announce, ApiClient>,
    // pub cartoon_service: ResourceService<Thumbnail, ApiClient>,

    // Build a resource service when needed, instead of building all of them at once.
    // That requires keep a full strategy list and a resource list, but have a better
    // generalization.
    pub client: reqwest::Client,
    pub config: FetchConfig,
    pub extractor: Extractor,
    pub chat_manager: ChatManager,
}

impl PriconneService {
    // let client = reqwest::Client::builder()
    // .user_agent("pcrinfobot-rs/0.0.1alpha Android")
    // .build()?;

    // pub fn new(
    //     database: mongodb::Database,
    //     api: ApiClient,
    //     news: NewsClient,
    //     chat_manager: ChatManager,
    //     telegraph: telegraph_rs::Telegraph,
    // ) -> Result<PriconneService, Error> {
    //     let post_collection = PostCollection(database.collection("posts"));

    //     let cartoon_service = ResourceService::new(
    //         api.clone(),
    //         FetchStrategy::DEFAULT,
    //         database.collection("cartoon"),
    //     );

    //     let news_service =
    //         ResourceService::new(news, FetchStrategy::DEFAULT, database.collection("news"));

    //     let information_service = ResourceService::new(
    //         api,
    //         FetchStrategy::DEFAULT,
    //         database.collection("information"),
    //     );

    //     let extractor = Extractor {
    //         tagger: RegexTagger { tag_rules: vec![] },
    //     };

    //     Ok(Self {
    //         database,
    //         cartoon_service,
    //         news_service,
    //         information_service,
    //         post_collection,
    //         chat_manager,
    //         extractor,
    //         telegraph,
    //     })
    // }
    pub fn service(&self, resource: Resource) {
        let strategy = self.config.strategy.build_for(resource);
        let collection = self.database.collection(resource.name());
        let client = self.client.clone();

        // TODO: WTF
        let b: ResourceService =
            match resource {
                Resource::Announce => ResourceService::<Announce, _>::new(
                    Box::new(ApiClient {
                        client,
                        api_server: self.config.server.api[0].clone(),
                    }),
                    strategy,
                    collection,
                ),
                Resource::News => ResourceService::<Thumbnail, _>::new(
                    Box::new(NewsClient {
                        client,
                        server: self.config.server.news.clone(),
                    }),
                    strategy,
                    collection,
                ),
                Resource::Cartoon => ResourceService::<News, _>::new(
                    Box::new(ApiClient {
                        client,
                        api_server: self.config.server.api[0].clone(),
                    }),
                    strategy,
                    collection,
                ),
            };
    }

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
    pub async fn add_resource<R, C, Response>(
        &self,
        service: &ResourceService<R, C>,
        find_result: ResourceFindResult<R>,
        // TODO: I still don't like to pass source explictly, any better way?
        source: Source,
    ) -> Result<(), Error>
    where
        R: ResourceMetadata<IdType = i32>
            + std::fmt::Debug
            + Sync
            + Send
            + Unpin
            + Serialize
            + DeserializeOwned,
        C: ResourceClient<R, Response = PostPageResponse<Response>> + Sync + Send,
        Response: crate::insight::PostPage,
        Response::ExtraData: Serialize + DeserializeOwned + Debug,
    {
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
        let page = service.page(resource).await?;

        // extract data
        // TODO: telegraph patch in utils
        let mut data = self.extractor.extract_post(&page);

        // TODO: wrap to somewhere
        let content_node = page.page.content().clone();
        let attrs = content_node.as_element().unwrap().clone().attributes;
        trace!("optimizing {attrs:?}");
        let content_node = utils::optimize_for_telegraph(content_node);

        let mut content = telegraph_rs::doms_to_nodes(content_node.children()).unwrap();
        if let Ok(data_json) = serde_json::to_string_pretty(&data.extra) {
            content.push(telegraph_rs::Node::NodeElement(telegraph_rs::NodeElement {
                tag: "br".to_string(),
                attrs: None,
                children: None,
            }));
            content.push(telegraph_rs::Node::NodeElement(telegraph_rs::NodeElement {
                tag: "br".to_string(),
                attrs: None,
                children: None,
            }));
            content.push(telegraph_rs::Node::NodeElement(telegraph_rs::NodeElement {
                tag: "code".to_string(),
                attrs: None,
                children: Some(vec![telegraph_rs::Node::Text(data_json.to_string())]),
            }));
        }

        let content = serde_json::to_string(&content)?;
        // tracing::trace!("{}", content);
        let telegraph = self
            .telegraph
            .create_page(&data.title, &content, false)
            .await?;
        data.telegraph_url = Some(telegraph.url);
        trace!("{data:?}");

        // generate final message action and execute
        // let post = Post::new(data);
        let post = match post {
            Some(mut post) => {
                post.push(data);
                post
            }
            None => Post::new(data),
        };
        self.post_collection.upsert(&post).await?;

        if action.is_update_only() {
            return Ok(());
        }

        // TODO: use action
        trace!("sending post {post:?}");
        self.chat_manager.send_post(&post).await;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Region {
    JP,
    EN,
    TW,
    CN,
    KR,
    TH,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use reqwest::{NoProxy, Proxy};
    use tracing::Level;
    use tracing_subscriber::EnvFilter;

    use crate::{resource::information::AjaxAnnounce, utils::HOUR};

    use super::*;

    async fn init_service() -> PriconneService {
        let mongo = mongodb::Client::with_uri_str("mongodb://localhost:27017/")
            .await
            .unwrap();
        let database = mongo.database("priconne_test");

        let mut client = reqwest::Client::builder().user_agent("pcrinfobot-rs/0.0.1alpha Android");
        let proxy = "http://127.0.0.1:2090";
        // if let Ok(proxy) = std::env::var("ALL_PROXY") {
        client = client.proxy(
            Proxy::all(proxy)
                .unwrap()
                .no_proxy(NoProxy::from_string("127.0.0.1,localhost")),
        );
        // }
        let client = client.build().unwrap();

        let api = ApiClient {
            client: client.clone(),
            api_server: api::ApiServer {
                id: "PROD1".to_owned(),
                url: reqwest::Url::parse("https://api-pc.so-net.tw/").unwrap(),
                name: "美食殿堂".to_owned(),
            },
        };

        let chat_manager = ChatManager {
            bot: teloxide::Bot::with_client(
                "5407842045:AAE8essS9PeiQThS-5_Jj7HSfIR_sAcHdKM",
                client.clone(),
            ),
            post_recipient: teloxide::types::Recipient::ChannelUsername("@pcrtwstat".to_owned()),
        };

        let news = NewsClient {
            client: client.clone(),
            server: reqwest::Url::parse("http://www.princessconnect.so-net.tw").unwrap(),
        };

        let telegraph = telegraph_rs::Telegraph::new("公連資訊")
            .access_token("73a944775a7c0079385a2697964c335c253896ec7a22acb1922886130f63")
            .client(client)
            .create()
            .await
            .unwrap();

        PriconneService::new(database, api, news, chat_manager, telegraph).unwrap()
    }

    #[tokio::test]
    async fn test_latest_information() {
        let service = init_service().await;
        let result = service
            .information_service
            .fused_stream()
            .next()
            .await
            .unwrap()
            .unwrap();
        println!("{result:#?}");
    }

    #[tokio::test]
    async fn test_add_information() {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_str("priconne=trace").unwrap())
            .init();

        let ajax_announce = AjaxAnnounce {
            announce_id: 2081,
            language: 1,
            category: 2,
            status: 1,
            platform: 3,
            slider_flag: 1,
            replace_time: chrono::DateTime::<chrono::Utc>::from_str("2023-01-31T03:55:00Z").unwrap().timestamp(),
            from_date: chrono::DateTime::<chrono::Utc>::from_str("2023-01-31T03:55:00Z").unwrap().with_timezone(&chrono::FixedOffset::east_opt(8 * HOUR).unwrap()),
            to_date: chrono::DateTime::<chrono::Utc>::from_str("2023-02-12T07:59:00Z").unwrap().with_timezone(&chrono::FixedOffset::east_opt(8 * HOUR).unwrap()),
            priority: 2081,
            end_date_slider_image: None,
            link_num: 1,
            title: crate::resource::information::AnnounceTitle {
                        title: "【轉蛋】《精選轉蛋》期間限定角色「智（萬聖節）」登場！機率UP活動舉辦預告！".to_owned(),
                        slider_image: Some(
                            "https://img-pc.so-net.tw/elements/media/announce/image/6574a1d415a825a20a8ca59a40872563.png".to_owned(),
                        ),
                        thumbnail_image: Some(
                            "https://img-pc.so-net.tw/elements/media/announce/image/e050a44f06047b63f369581e66724361.png".to_owned(),
                        ),
                        banner_ribbon: 2,
                    },
        };
        let find_result = ResourceFindResult::from_new(Announce::from(ajax_announce));

        let service = init_service().await;
        service
            .add_resource(
                &service.information_service,
                find_result,
                Source::Announce("PROD1".to_string()),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_last_post() {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_str("priconne=trace").unwrap())
            .init();

        let service = init_service().await;

        let information = service
            .information_service
            .fused_stream()
            .next()
            .await
            .unwrap()
            .unwrap();
        let news = service
            .news_service
            .fused_stream()
            .next()
            .await
            .unwrap()
            .unwrap();

        service
            .add_resource(&service.news_service, news, Source::News)
            .await
            .unwrap();
        service
            .add_resource(
                &service.information_service,
                information,
                Source::Announce(service.information_service.client.api_server.id.clone()),
            )
            .await
            .unwrap();
    }

    async fn test_today_post() {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_str("priconne=trace").unwrap())
            .init();

        let service = init_service().await;
    }
}
