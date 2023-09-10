use std::{pin::Pin, sync::Arc};

use axum::{routing::get, Router};

use clap::{Parser, Subcommand};
use futures::future::Either;
use priconne::config::PriconneConfig;
use schemars::schema_for;
use teloxide::prelude::LoggingErrorHandler;
use tokio_cron_scheduler::JobScheduler;
use tracing_subscriber::{fmt::format, EnvFilter};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Output JSON schema of config
    Schema,
    /// Start server
    Serve,
    /// Incoming events
    Events,
}

fn init_logging() {
    tracing_subscriber::fmt()
        .event_format(format().pretty())
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}

#[tokio::main]
async fn main() -> priconne::Result<()> {
    init_logging();
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        match command {
            Commands::Schema => {
                let schema = schema_for!(priconne::config::PriconneConfig);
                let schema = serde_json::to_string_pretty(&schema)?;
                println!("{}", schema);
                return Ok(());
            }
            Commands::Serve => serve().await?,
            Commands::Events => {
                let config = std::fs::File::open("config.yaml")?;
                let config: PriconneConfig = serde_yaml::from_reader(config)?;
                let priconne = config.build().await?;
                let events = priconne.incomming_events().await?;
                println!("{:#?}", events);
            }
        }
    }

    Ok(())
}

async fn serve() -> priconne::Result<()> {
    let config = std::fs::File::open("config.yaml")?;
    let config: PriconneConfig = serde_yaml::from_reader(config)?;
    let priconne = Arc::new(config.build().await?);

    let mut dispatcher = priconne::chat::dispatcher(&priconne, &priconne.chat_manager.bot);

    let mut stop_flag: Pin<Box<dyn futures::Future<Output = ()> + Send>> = Box::pin(async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler")
    });

    let mut app = Router::new().route("/", get(|| async { "Hello, World!" }));

    let dispatcher = match priconne
        .chat_manager
        .config
        .build_webhook_options()
        .await
        .transpose()?
    {
        Some(options) => {
            tracing::info!("Using Telegram webhook");
            let axum_to_router = teloxide::update_listeners::webhooks::axum_to_router(
                priconne.chat_manager.bot.clone(),
                options,
            )
            .await?;

            stop_flag = Box::pin(axum_to_router.1);
            app = axum_to_router.2;

            let dispatcher =
                dispatcher.dispatch_with_listener(axum_to_router.0, LoggingErrorHandler::new());
            Either::Left(dispatcher)
        }
        None => Either::Right(dispatcher.dispatch()),
    };

    let server = axum::Server::bind(&"127.0.0.1:5555".parse()?)
        .serve(app.into_make_service())
        .with_graceful_shutdown(stop_flag);

    let sched = JobScheduler::new().await?;
    priconne.clone().add_jobs(&sched).await?;
    sched.shutdown_on_ctrl_c();
    sched.start().await?; // This immediately returns

    let ((), server_result) = tokio::join!(dispatcher, server);
    server_result?;

    Ok(())
}
