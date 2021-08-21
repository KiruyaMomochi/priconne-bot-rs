use std::{convert::Infallible, net::SocketAddr};

use http::StatusCode;
use teloxide::{
    dispatching::{
        stop_token::AsyncStopToken,
        update_listeners::{self, StatefulListener},
    },
    prelude::*,
    types::Update,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use url::Url;
use warp::Filter;

async fn handle_rejection(error: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    log::error!("Cannot process the request due to: {:?}", error);
    Ok(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn webhook(
    bot: &AutoSend<teloxide::Bot>,
    webhook_url: &str,
    listen_addr: &str,
) -> impl update_listeners::UpdateListener<Infallible> {
    set_webhook(bot, webhook_url);
    listen_webhook(listen_addr).await
}

pub async fn set_webhook(
    bot: &AutoSend<teloxide::Bot>,
    webhook_url: &str,
) -> Result<(), teloxide::RequestError> {
    let url = Url::parse(webhook_url).unwrap();
    bot.set_webhook(url).await;
    Ok(())
}

pub async fn listen_webhook(
    listen_addr: &str,
) -> impl update_listeners::UpdateListener<Infallible> {
    let (tx, rx) = mpsc::unbounded_channel();

    let server = warp::post()
        .and(warp::body::json())
        .map(move |json: serde_json::Value| {
            if let Ok(update) = Update::try_parse(&json) {
                tx.send(Ok(update))
                    .expect("cannot send an incoming update from the webhook")
            };
            StatusCode::OK
        })
        .recover(handle_rejection);

    let (stop_token, stop_flag) = AsyncStopToken::new_pair();
    let (_addr, fut) = warp::serve(server)
        .bind_with_graceful_shutdown(listen_addr.parse::<SocketAddr>().unwrap(), stop_flag);

    tokio::spawn(fut);
    let stream = UnboundedReceiverStream::new(rx);

    fn streamf<S, T>(state: &mut (S, T)) -> &mut S {
        &mut state.0
    }

    StatefulListener::new(
        (stream, stop_token),
        streamf,
        |state: &mut (_, AsyncStopToken)| state.1.clone(),
    )
}
