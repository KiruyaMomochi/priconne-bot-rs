
struct UnboundedWebhook<L: UpdateListener<Infallible>> {
    pub tx: UnboundedSender<Result<Update, Infallible>>,
    pub stop_flag: stop_token::AsyncStopFlag,
    pub listener: L,
}

fn unbounded_receiver_listener(
    rx: UnboundedReceiver<Result<Update, Infallible>>,
    stop_token: AsyncStopToken,
) -> impl UpdateListener<Infallible> {
    let stream = UnboundedReceiverStream::new(rx);
    let state = (stream, stop_token);

    fn first_of<S, T>((s, _): &mut (S, T)) -> &mut S {
        s
    }
    fn stop_token_of<S, T: Clone>((_, t): &mut (S, T)) -> T {
        t.clone()
    }

    StatefulListener::new(
        state,
        first_of::<_, AsyncStopToken>,
        stop_token_of::<_, AsyncStopToken>,
    )
}

fn unbounded_webhook() -> UnboundedWebhook<impl UpdateListener<Infallible>> {
    let (tx, rx) = mpsc::unbounded_channel::<Result<Update, Infallible>>();
    let (stop_token, stop_flag) = AsyncStopToken::new_pair();

    let listener = unbounded_receiver_listener(rx, stop_token);
    UnboundedWebhook {
        tx,
        stop_flag,
        listener,
    }
}

async fn telegram_webhook(
    Extension(sender): Extension<Arc<Sender<Result<Update, Infallible>>>>,
    Json(update): Json<Update>,
) -> impl IntoResponse {
    tracing::debug!("Received update: {:?}", update);
    sender.send(Ok(update)).await.unwrap();
}
