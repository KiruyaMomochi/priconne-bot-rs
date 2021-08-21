use tokio_cron_scheduler::{Job, JobScheduler};

mod api;
mod article;
mod bot;
mod client;
mod config;
mod database;
mod error;
mod message;
mod page;
mod schedule;
mod telegram;
pub mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sched = JobScheduler::new();

    let job = Job::new("0/10 * * * * *", |uuid, _l| {
        println!("Task 1 {:#?}", uuid);
    })?;

    let _ret = sched.add(job).unwrap();

    sched.add(
        Job::new("5/10 * * * * *", |uuid, _l| {
            println!("Task 2 {:#?}", uuid);
        })
        .unwrap(),
    );

    return Ok(());

    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .try_init()?;

    let config = std::fs::read_to_string("config.toml")?;
    let config: config::BotConfig = toml::from_str(&config)?;

    println!("{:#?}", config);

    let bot = config.build().await?;
    let _client = &bot.client;

    return Ok(());

    // let proxy = reqwest::Proxy::all("http://127.0.0.1:10809").unwrap();
    // let client = reqwest::Client::builder()
    //     .proxy(proxy)
    //     .user_agent("pcrinfobot-rs/0.1.0 Android")
    //     .build()
    //     .unwrap();

    // let news_server = "http://www.princessconnect.so-net.tw/";
    // let information_server = "https://api-pc.so-net.tw/";

    // let telegraph = telegraph_rs::Telegraph::new("@pcrtw")
    //     .client(client.clone())
    //     .access_token("b968da509bb76866c35425099bc0989a5ec3b32997d55286c657e6994bbb")
    //     .create()
    //     .await
    //     .unwrap();

    // let bot = teloxide::Bot::new("1270643903:AAHYqGhXd3jYQLDueoYYQmugTGZy9CFrhjw").auto_send();

    // teloxide::repl_with_listener(
    //     bot,
    //     |message| async move {
    //         if message.update.chat.is_private() {
    //             message.answer_dice().await?;
    //         }
    //         respond(())
    //     },
    //     webhook(
    //         &bot,
    //         "https://www.konosubafd.shop/newbotrust",
    //         "127.0.0.1:3033",
    //     )
    //     .await,
    // )
    // .await;

    Ok(())
}

// #[cfg(test)]
// mod test {
//     use std::pin::Pin;

//     use futures::StreamExt;
//     use kuchiki::traits::TendrilSink;
//     use log::info;
//     use teloxide::prelude::RequesterExt;

//     use crate::{
//         bot::Bot,
//         client::{CartoonExt, Client, InformationClient},
//         error::Error,
//         page::{glossary::Glossary, Page},
//         tagger,
//     };

//     fn init() {
//         let _ = env_logger::builder()
//             .filter_level(log::LevelFilter::Info)
//             .is_test(true)
//             .try_init();
//     }

//     #[tokio::test]
//     async fn test_fuck() -> Result<(), Error> {
//         init();

//         let proxy = reqwest::Proxy::all("http://127.0.0.1:10809").unwrap();
//         let client = reqwest::Client::builder()
//             .proxy(proxy)
//             .user_agent("pcrinfobot-rs/0.1.0 Android")
//             .build()
//             .unwrap();

//         let news_server = "http://www.princessconnect.so-net.tw/";
//         let information_server = "https://api-pc.so-net.tw/";
//         let news_client =
//             Client::with_client(news_server, information_server, client.clone()).unwrap();

//         let telegraph = telegraph_rs::Telegraph::new("@pcrtw")
//             .client(client.clone())
//             .access_token("b968da509bb76866c35425099bc0989a5ec3b32997d55286c657e6994bbb")
//             .create()
//             .await
//             .unwrap();

//         let bot =
//             teloxide::Bot::with_client("1270643903:AAHYqGhXd3jYQLDueoYYQmugTGZy9CFrhjw", client)
//                 .auto_send();

//         let mongo_client = mongodb::Client::with_uri_str("mongodb+srv://staging:89qzo3qtcJ4IYMC1@mongodb.gnqxc.mongodb.net/priconne-bot?retryWrites=true&w=majority").await.unwrap();

//         let db = mongo_client.database("pcrtwinfo");

//         let tagger = tagger!("公會小屋", "限定復刻" => "復刻");

//         // let announce = &information_client.ajax(1).await.unwrap().announce_list[0];
//         let fuck = Bot {
//             client: news_client,
//             mongo_database: db,
//             telegraph,
//             bot,
//             tagger,
//             chat_id: teloxide::types::ChatId::ChannelUsername("@pcrtwstat".to_string()),
//         };

//         // let news = fuck.client.news_list(0).await.unwrap();
//         // fuck.news(&news[0]).await?;
//         fuck.news_all().await?;

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_html() -> Result<(), Error> {
//         let proxy = reqwest::Proxy::all("http://127.0.0.1:10809").unwrap();
//         let client = reqwest::Client::builder()
//             .proxy(proxy)
//             .user_agent("pcrinfobot-rs/0.1.0 Android")
//             .build()
//             .unwrap();

//         let news_server = "http://www.princessconnect.so-net.tw/";
//         let information_server = "https://api-pc.so-net.tw/";
//         let news_client =
//             Client::with_client(news_server, information_server, client.clone()).unwrap();

//         let news_list = news_client.ajax_announce(8964).await?;

//         // let path = std::path::Path::new("news.html");
//         // let document = kuchiki::parse_html().from_utf8().from_file(&path).unwrap();
//         // let news = NewsList::from_document(document);

//         println!("{:#?}", news_list);

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_fuck_2() -> Result<(), Error> {
//         init();
//         info!("log test!");

//         let proxy = reqwest::Proxy::all("http://127.0.0.1:10809").unwrap();
//         let client = reqwest::Client::builder()
//             .proxy(proxy)
//             .user_agent("pcrinfobot-rs/0.1.0 Android")
//             .build()
//             .unwrap();

//         let news_server = "http://www.princessconnect.so-net.tw/";
//         let information_server = "https://api-pc.so-net.tw/";
//         let information_client =
//             Client::with_client(news_server, information_server, client.clone()).unwrap();

//         let telegraph = telegraph_rs::Telegraph::new("@pcrtw")
//             .client(client.clone())
//             .access_token("b968da509bb76866c35425099bc0989a5ec3b32997d55286c657e6994bbb")
//             .create()
//             .await
//             .unwrap();

//         let bot =
//             teloxide::Bot::with_client("1270643903:AAHYqGhXd3jYQLDueoYYQmugTGZy9CFrhjw", client)
//                 .auto_send();

//         let mongo_client = mongodb::Client::with_uri_str("mongodb+srv://staging:89qzo3qtcJ4IYMC1@mongodb.gnqxc.mongodb.net/priconne-bot?retryWrites=true&w=majority").await.unwrap();

//         let db = mongo_client.database("pcrtwinfo");

//         let tagger = tagger!("公會小屋", "限定復刻" => "復刻");

//         // let announce = &information_client.ajax(1).await.unwrap().announce_list[0];
//         let fuck = Bot {
//             client: information_client,
//             mongo_database: db,
//             telegraph,
//             bot,
//             tagger,
//             chat_id: teloxide::types::ChatId::ChannelUsername("@pcrtwstat".to_string()),
//         };

//         fuck.announce_all().await?;

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_cartoon() -> Result<(), Error> {
//         init();
//         info!("log test!");

//         let proxy = reqwest::Proxy::all("http://127.0.0.1:10809").unwrap();
//         let client = reqwest::Client::builder()
//             .proxy(proxy)
//             .user_agent("pcrinfobot-rs/0.1.0 Android")
//             .build()
//             .unwrap();

//         let news_server = "http://www.princessconnect.so-net.tw/";
//         let information_server = "https://api-pc.so-net.tw/";
//         let information_client =
//             Client::with_client(news_server, information_server, client.clone()).unwrap();

//         let telegraph = telegraph_rs::Telegraph::new("@pcrtw")
//             .client(client.clone())
//             .access_token("b968da509bb76866c35425099bc0989a5ec3b32997d55286c657e6994bbb")
//             .create()
//             .await
//             .unwrap();

//         let bot =
//             teloxide::Bot::with_client("1270643903:AAHYqGhXd3jYQLDueoYYQmugTGZy9CFrhjw", client)
//                 .auto_send();

//         let mongo_client = mongodb::Client::with_uri_str("mongodb+srv://staging:89qzo3qtcJ4IYMC1@mongodb.gnqxc.mongodb.net/priconne-bot?retryWrites=true&w=majority").await.unwrap();

//         let db = mongo_client.database("pcrtwinfo");

//         let tagger = tagger!("公會小屋", "限定復刻" => "復刻");

//         // let announce = &information_client.ajax(1).await.unwrap().announce_list[0];
//         let fuck = Bot {
//             client: information_client,
//             mongo_database: db,
//             telegraph,
//             bot,
//             tagger,
//             chat_id: teloxide::types::ChatId::ChannelUsername("@pcrtwstat".to_string()),
//         };

//         let thumbnails = fuck.client.cartoon_stream();
//         let mut thumbnails = unsafe { Pin::new_unchecked(thumbnails) };
//         thumbnails.next().await.unwrap();
//         let thumbnail = thumbnails.next().await.unwrap();
//         fuck.cartoon(&thumbnail).await?;

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_glossary() -> Result<(), Error> {
//         // let path = Path::new("glossary.html");
//         // std::fs::File::open(path)
//         let document = kuchiki::parse_html()
//             .from_utf8()
//             .from_file("glossary.html")
//             .unwrap();
//         let glossary = Glossary::from_document(document)?;

//         let proxy = reqwest::Proxy::all("http://127.0.0.1:10809").unwrap();
//         let client = reqwest::Client::builder()
//             .proxy(proxy)
//             .user_agent("pcrinfobot-rs/0.1.0 Android")
//             .build()
//             .unwrap();

//         let glossary = glossary
//             .0
//             .into_iter()
//             .fold(String::new(), |string, (k, v)| {
//                 string + "- **" + &k + "**:\\\n    " + &v + "\n"
//             });
//         println!("{}", glossary);

//         Ok(())
//     }
// }
