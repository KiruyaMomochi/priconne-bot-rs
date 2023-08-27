//! Service module
//!
//! While the [`resource`] module is responsible for fetching and parsing data,
//! this service layer is responsible for managing the resources, continuously
//! fetching and parsing data, and sending messages using [`ChatManager`].

use std::sync::Arc;

use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::bson::doc;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{
    chat::ChatManager,
    client::ResourceClient,
    config::FetchConfig,
    database::AnnouncementCollection,
    insight::Extractor,
    resource::{api::ApiClient, event::Event, news::service::NewsClient, ResourceKind},
    Result,
};

// TODO: We may need to housekeeping the database.
/// Resource collection is generalized to two steps, as in this trait.
///
/// 1. Get metadata from remote
/// 2. Fetch full content of metadata (if required), send a chat and insert them into database.
///
#[async_trait]
pub trait ResourceService<M> {
    /// Collect latest metadata
    async fn collect_latests(&self, priconne: &PriconneService) -> Result<Vec<M>>;
    /// For any given metadata, extract data from it
    async fn work(&self, priconne: &PriconneService, metadata: M) -> Result<()>
    where
        M: 'async_trait;
}

// TODO: Since these values are cloned, we may want to use `Arc` instead. Or their mutation may not be reflected.
/// Central service for Priconne resource management.
/// It contains all the resources and their corresponding services.
///
/// Build a resource service when needed, instead of building all of them at once.
/// That requires keep a full strategy list and a resource list, but have a better
/// generalization.
///
/// Alternative implementation:
/// ```rust,ignore
/// pub news_service: ResourceService<News, NewsClient>,
/// pub information_service: ResourceService<Announce, ApiClient>,
/// pub cartoon_service: ResourceService<Thumbnail, ApiClient>,
/// ```
///
#[derive(Clone)]
pub struct PriconneService {
    pub database: mongodb::Database,
    pub telegraph: telegraph_rs::Telegraph,

    pub client: reqwest::Client,
    pub config: FetchConfig,
    pub extractor: Extractor,
    pub chat_manager: Arc<ChatManager>,
}

impl PriconneService {
    pub fn new(
        database: mongodb::Database,
        chat_manager: ChatManager,
        telegraph: telegraph_rs::Telegraph,
        client: reqwest::Client,
        config: FetchConfig,
        extractor: Extractor,
    ) -> Result<PriconneService> {
        let chat_manager = Arc::new(chat_manager);

        Ok(Self {
            database,
            chat_manager,
            extractor,
            telegraph,
            client,
            config,
        })
    }

    pub async fn serve_and_work<M>(&self, service: impl ResourceService<M>) -> Result<()> {
        let latests = service.collect_latests(self).await;

        for result in latests? {
            service.work(self, result).await?;
        }
        Ok(())
    }

    fn build_api_client(&self) -> ApiClient {
        ApiClient {
            client: self.client.clone(),
            api_server: self.config.server.api[0].clone(),
        }
    }

    fn build_news_client(&self) -> NewsClient {
        NewsClient {
            client: self.client.clone(),
            server: self.config.server.news.clone(),
        }
    }

    pub async fn run_service(&self, kind: ResourceKind) -> Result<()> {
        match kind {
            ResourceKind::Information => {
                let api_client = self.build_api_client().memorize(
                    self.database
                        .collection::<crate::resource::information::Announce>(&kind.to_string()),
                    self.config.strategy.build_for(kind),
                );
                self.serve_and_work(api_client).await?
            }
            ResourceKind::News => {
                let news_client = self.build_news_client().memorize(
                    self.database.collection(&kind.to_string()),
                    self.config.strategy.build_for(kind),
                );
                self.serve_and_work(news_client).await?
            }
            ResourceKind::Cartoon => {
                let api_client = self.build_api_client().memorize(
                    self.database
                        .collection::<crate::resource::cartoon::Thumbnail>(&kind.to_string()),
                    self.config.strategy.build_for(kind),
                );
                self.serve_and_work(api_client).await?
            }
            _ => todo!(),
        };

        Ok(())
    }

    // TODO: This currently queries the announcement resource. In the future, we will have a dedicated
    /// List all incoming events
    /// event collection.
    pub async fn incomming_events(&self) -> Result<Vec<Event>> {
        let collection = AnnouncementCollection(self.database.collection("announcement")).0;
        let announcements = collection
            .find(
                doc! {
                    "events.end": {
                        "$gt": chrono::Utc::now() - chrono::Duration::days(2)
                    }
                },
                None,
            )
            .await?;

        Ok(announcements
            .try_filter_map(|a| async move {
                Ok(Some(
                    a.events
                        .into_iter()
                        .map(move |e| {
                            Event {
                                start: e.start,
                                end: e.end,
                                kind: crate::resource::event::EventKind::Other, // TODO: This is a placeholder
                                title: e.title,
                                announcement_id: a.id,
                                announcement_title: a.data.last().unwrap().title.clone(),
                            }
                        })
                        .collect::<Vec<_>>(),
                ))
            })
            .try_concat()
            .await?)
    }

    /// Build scheduler
    pub async fn add_jobs(self: Arc<Self>, sched: &JobScheduler) -> Result<()> {
        let priconne = self;

        for (kind, cron) in priconne.config.schedule.iter() {
            let kind: ResourceKind = kind.parse()?;
            // First clone (1): provide `priconne` for the following closure
            let priconne = priconne.clone();
            let run = move |_uuid: uuid::Uuid, _lock: JobScheduler| -> std::pin::Pin<Box<dyn futures::Future<Output = ()> + Send>> {
                // Second clone (2) + move before clusoure: priconne(1) is moved to this closure
                // But in order to make the closure `Fn` not `FnOnce`, it is borrowed and cloned here
                // https://github.com/rust-lang/rust/issues/74497#issuecomment-1534485733
                let kind = kind.clone();
                let priconne = priconne.clone();
                Box::pin(async move {
                    priconne
                        .run_service(kind)
                        .await
                        .map_err(|e| {
                            tracing::error!("Error when running service: {}", e);
                            // TODO: Send error message to chat or delete the job
                        })
                        .unwrap();
                })
            };
            let job = Job::new_async(cron.clone(), run)?;
            sched.add(job).await?;
        }

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use std::str::FromStr;

//     use reqwest::{NoProxy, Proxy};
//     use tracing::Level;
//     use tracing_subscriber::EnvFilter;

//     use crate::{resource::information::AjaxAnnounce, utils::HOUR};

//     use super::*;

//     async fn init_service() -> PriconneService {
//         let mongo = mongodb::Client::with_uri_str("mongodb://localhost:27017/")
//             .await
//             .unwrap();
//         let database = mongo.database("priconne_test");

//         let mut client = reqwest::Client::builder().user_agent("pcrinfobot-rs/0.0.1alpha Android");
//         let proxy = "http://127.0.0.1:2090";
//         // if let Ok(proxy) = std::env::var("ALL_PROXY") {
//         client = client.proxy(
//             Proxy::all(proxy)
//                 .unwrap()
//                 .no_proxy(NoProxy::from_string("127.0.0.1,localhost")),
//         );
//         // }
//         let client = client.build().unwrap();

//         let api = ApiClient {
//             client: client.clone(),
//             api_server: api::ApiServer {
//                 id: "PROD1".to_owned(),
//                 url: reqwest::Url::parse("https://api-pc.so-net.tw/").unwrap(),
//                 name: "美食殿堂".to_owned(),
//             },
//         };

//         let chat_manager = ChatManager {
//             bot: teloxide::Bot::with_client(
//                 "5407842045:AAE8essS9PeiQThS-5_Jj7HSfIR_sAcHdKM",
//                 client.clone(),
//             ),
//             post_recipient: teloxide::types::Recipient::ChannelUsername("@pcrtwstat".to_owned()),
//         };

//         let news = NewsClient {
//             client: client.clone(),
//             server: reqwest::Url::parse("http://www.princessconnect.so-net.tw").unwrap(),
//         };

//         let telegraph = telegraph_rs::Telegraph::new("公連資訊")
//             .access_token("73a944775a7c0079385a2697964c335c253896ec7a22acb1922886130f63")
//             .client(client)
//             .create()
//             .await
//             .unwrap();

//         PriconneService::new(database, api, news, chat_manager, telegraph).unwrap()
//     }

//     #[tokio::test]
//     async fn test_latest_information() {
//         let service = init_service().await;
//         let result = service
//             .information_service
//             .fused_stream()
//             .next()
//             .await
//             .unwrap()
//             .unwrap();
//         println!("{result:#?}");
//     }

//     #[tokio::test]
//     async fn test_add_information() {
//         tracing_subscriber::fmt()
//             .with_env_filter(EnvFilter::from_str("priconne=trace").unwrap())
//             .init();

//         let ajax_announce = AjaxAnnounce {
//             announce_id: 2081,
//             language: 1,
//             category: 2,
//             status: 1,
//             platform: 3,
//             slider_flag: 1,
//             replace_time: chrono::DateTime::<chrono::Utc>::from_str("2023-01-31T03:55:00Z").unwrap().timestamp(),
//             from_date: chrono::DateTime::<chrono::Utc>::from_str("2023-01-31T03:55:00Z").unwrap().with_timezone(&chrono::FixedOffset::east_opt(8 * HOUR).unwrap()),
//             to_date: chrono::DateTime::<chrono::Utc>::from_str("2023-02-12T07:59:00Z").unwrap().with_timezone(&chrono::FixedOffset::east_opt(8 * HOUR).unwrap()),
//             priority: 2081,
//             end_date_slider_image: None,
//             link_num: 1,
//             title: crate::resource::information::AnnounceTitle {
//                         title: "【轉蛋】《精選轉蛋》期間限定角色「智（萬聖節）」登場！機率UP活動舉辦預告！".to_owned(),
//                         slider_image: Some(
//                             "https://img-pc.so-net.tw/elements/media/announce/image/6574a1d415a825a20a8ca59a40872563.png".to_owned(),
//                         ),
//                         thumbnail_image: Some(
//                             "https://img-pc.so-net.tw/elements/media/announce/image/e050a44f06047b63f369581e66724361.png".to_owned(),
//                         ),
//                         banner_ribbon: 2,
//                     },
//         };
//         let find_result = MetadataFindResult::from_new(Announce::from(ajax_announce));

//         let service = init_service().await;
//         service
//             .work_announcement(
//                 &service.information_service,
//                 find_result,
//                 AnnouncementSource::Api("PROD1".to_string()),
//             )
//             .await
//             .unwrap();
//     }

//     #[tokio::test]
//     async fn test_last_post() {
//         tracing_subscriber::fmt()
//             .with_env_filter(EnvFilter::from_str("priconne=trace").unwrap())
//             .init();

//         let service = init_service().await;

//         let information = service
//             .information_service
//             .fused_stream()
//             .next()
//             .await
//             .unwrap()
//             .unwrap();
//         let news = service
//             .news_service
//             .fused_stream()
//             .next()
//             .await
//             .unwrap()
//             .unwrap();

//         service
//             .work_announcement(&service.news_service, news, AnnouncementSource::Website)
//             .await
//             .unwrap();
//         service
//             .work_announcement(
//                 &service.information_service,
//                 information,
//                 AnnouncementSource::Api(service.information_service.client.api_server.id.clone()),
//             )
//             .await
//             .unwrap();
//     }

//     async fn test_today_post() {
//         tracing_subscriber::fmt()
//             .with_env_filter(EnvFilter::from_str("priconne=trace").unwrap())
//             .init();

//         let service = init_service().await;
//     }
// }
