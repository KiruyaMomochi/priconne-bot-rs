//! Service module
//!
//! While the [`resource`] module is responsible for fetching and parsing data,
//! this service layer is responsible for managing the resources, continuously
//! fetching and parsing data, and sending messages using [`ChatManager`].

use async_trait::async_trait;

use crate::{
    config::FetchConfig,
    error::Error,
    insight::{tagging::RegexTagger, Extractor},
    message::ChatManager,
};

// pub trait ServiceBuilder {
//     type Metadata: ResourceMetadata;
//     type CollectionService: ResourceCollectionService<Self::Metadata>;
//     type Service: ResourceService<Self::Metadata>;
//     fn build_collection_service(&self, priconne: &PriconneService) -> Self::CollectionService;
//     fn build_service(&self, priconne: &PriconneService) -> Self::Service;
// }

#[async_trait]
pub trait ResourceService<M> {
    /// Collect latest metadata
    async fn collect_latests(&self, priconne: &PriconneService) -> Result<Vec<M>, Error>;
    /// For any given metadata, extract data from it
    async fn work(&self, priconne: &PriconneService, metadata: M) -> Result<(), Error>
    where
        M: 'async_trait;
}

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
pub struct PriconneService {
    pub database: mongodb::Database,
    pub telegraph: telegraph_rs::Telegraph,

    pub client: reqwest::Client,
    pub config: FetchConfig,
    pub extractor: Extractor,
    pub chat_manager: ChatManager,
}

impl PriconneService {
    pub fn new(
        database: mongodb::Database,
        chat_manager: ChatManager,
        telegraph: telegraph_rs::Telegraph,
        client: reqwest::Client,
        config: FetchConfig,
    ) -> Result<PriconneService, Error> {
        let extractor = Extractor {
            tagger: RegexTagger { tag_rules: vec![] },
        };

        Ok(Self {
            database,
            chat_manager,
            extractor,
            telegraph,
            client,
            config,
        })
    }

    pub async fn serve_and_work<M>(&self, service: impl ResourceService<M>) -> Result<(), Error> {
        let latests = service.collect_latests(self).await;

        for result in latests? {
            service.work(self, result).await?;
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
