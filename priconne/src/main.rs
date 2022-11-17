



use priconne::{built_info, api::ApiServer};
use reqwest::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ua = format!(
        "priconne-bot-rs/{} {} {}",
        built_info::PKG_VERSION,
        built_info::TARGET,
        "Android"
    );
    println!("ua: {}", ua);

    let _client = reqwest::Client::builder().user_agent(ua).build()?;

    let dbc = mongodb::Client::with_uri_str("mongodb+srv://staging:89qzo3qtcJ4IYMC1@mongodb.gnqxc.mongodb.net/?retryWrites=true&w=majority").await?;
    let _database = dbc.database("priconne-bot-develop");
    let _api = ApiServer {
        id: "PROD1".to_string(),
        url: Url::parse("https://api-pc.so-net.tw/").unwrap(),
        name: "美食殿堂".to_string(),
    };

    // let service = PriconneService {
    //     client,
    //     database,
    //     api_servers: vec![api.clone()],
    //     api_server: api,
    //     news_server: Url::parse("http://www.princessconnect.so-net.tw").unwrap(),
    //     strategy: FetchStrategy::DEFAULT,
    //     handler: Box::new(|event| {
    //         println!("{:?}", event);
    //     }),
    //     news_collection: todo!(),
    //     announce_collection: todo!(),
    //     cartoon_thumbnail_collection: todo!(),
    // };

    // service.news_received.subscribe(|news| {
    //     println!("{:?}", news);
    // });
    // service.check_new_news().await;

    // if let Some(news) = Box::into_pin(service.news_stream()).next().await {
    //     println!("{:?}", news);
    // }

    Ok(())
}
