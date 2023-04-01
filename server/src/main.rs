use axum::Router;

use std::net::SocketAddr;
use teloxide::{
    prelude::Requester, repls::CommandReplExt, respond, types::Message,
    utils::command::BotCommands, Bot,
};

use tracing::Level;

#[derive(Debug, BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum TelegramCommand {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "get cartoon by id.")]
    Cartoon { id: i32 },
    #[command(description = "get news by id.")]
    News { id: i32 },
    #[command(description = "get information by id.")]
    Information { id: i32 },
    #[command(description = "send all cartoons.")]
    CartoonAll,
    #[command(description = "send all articles.")]
    ArticleAll,
    #[command(description = "send all news.")]
    NewsAll,
    #[command(description = "stop.")]
    Shutdown,
}

fn init_logging() {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
}

#[tokio::main]
async fn main() {
    // listening on port 3000
    let port = 3000;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("listening on {}", addr);

    // if we are in Codespace, then build url from enviroment variable
    let webhook_url = match std::env::var("CODESPACE_NAME") {
        Ok(name) => url::Url::parse(&format!("https://{name}-{port}.githubpreview.dev/tghook")),
        Err(_) => todo!("Not in Codespace"),
    }
    .unwrap();

    let bot = Bot::new("5407842045:AAE8essS9PeiQThS-5_Jj7HSfIR_sAcHdKM");

    // create router
    let (listener, stop_flag, router) =
        teloxide::dispatching::update_listeners::webhooks::axum_to_router(
            bot.clone(),
            teloxide::dispatching::update_listeners::webhooks::Options::new(
                addr, // This should not be used by teloxide
                webhook_url,
            ),
        )
        .await
        .unwrap();

    // build our application with a route
    let app = Router::new()
        // Nest teloxide router
        .nest("/", router);

    tokio::join!(
        // run the server
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .with_graceful_shutdown(stop_flag),
        // run the bot
        TelegramCommand::repl_with_listener(
            bot,
            |bot: Bot, msg: Message, cmd: TelegramCommand| async move {
                bot.send_message(msg.chat.id, format!("{cmd:?}")).await?;
                respond(()) 
            },
            listener
        )
    )
    .0
    .unwrap();
}

#[cfg(test)]
mod tests {
    use teloxide::{
        prelude::{Request, Requester},
        types::{Chat, ChatKind, Message, UpdateKind},
    };

    #[test]
    fn closure_mut_tuple_matching() {
        let mut tuple = (1, 2);

        fn func((a, _b): &mut (i32, i32)) -> &mut i32 {
            a
        }
        *func(&mut tuple) = 3;

        assert_eq!(tuple, (3, 2));
    }

    #[tokio::test]
    async fn bot_test() {
        let bot = teloxide::Bot::new("1214140516:AAG9sZ4Ex76qZ4f3qnisCyz-Tbwq20V6ei8");
        let update = bot.get_updates().send().await.unwrap()[0].clone();
        if let UpdateKind::Message(Message {
            chat:
                Chat {
                    kind: ChatKind::Private(_chat),
                    ..
                },
            ..
        }) = update.clone().kind
        {
            let json = serde_json::to_string(&update).unwrap();
            println!("{json}");
        }
    }
}
