use clap::{Parser, Subcommand};
use priconne::{
    built_info,
    client::{MemorizedResourceClient, ResourceClient},
    config::PriconneConfig,
    resource::{
        api::ApiClient,
        cartoon::service::{self, CartoonService},
    },
    service::{PriconneService, ResourceService},
};
use schemars::schema_for;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Output JSON schema of config
    Schema,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    if let Some(Commands::Config {
        command: ConfigCommands::Schema,
    }) = cli.command
    {
        let schema = schema_for!(priconne::config::PriconneConfig);
        let schema = serde_json::to_string_pretty(&schema)?;
        println!("{}", schema);
        return Ok(());
    }

    let ua = format!(
        "priconne-bot-rs/{} {} {}",
        built_info::PKG_VERSION,
        built_info::TARGET,
        "Android"
    );
    println!("ua: {ua}");

    let _client = reqwest::Client::builder().user_agent(ua).build()?;

    let dbc = mongodb::Client::with_uri_str("mongodb://localhost").await?;
    let _database = dbc.database("priconne-bot-develop");

    let config = std::fs::File::open("config.yaml")?;
    let config: PriconneConfig = serde_yaml::from_reader(config)?;
    let priconne = config.build().await?;

    let client = ApiClient {
        client: priconne.client.clone(),
        api_server: priconne.config.server.api[0].clone(),
    };
    let mut straegy = priconne.config.strategy.build_for("information");
    straegy.ignore_time_lt = Some(chrono::Utc::now() - chrono::Duration::days(1));
    let memorized = client.memorize(
        // collection: priconne.database.collection("api_cache").into(),
        priconne.database.collection("api_cache"),
        straegy,
    );
    priconne.serve_and_work(memorized).await?;

    // let _api = ApiServer {
    //     id: "PROD1".to_string(),
    //     url: Url::parse("https://api-pc.so-net.tw/").unwrap(),
    //     name: "美食殿堂".to_string(),
    // };

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
