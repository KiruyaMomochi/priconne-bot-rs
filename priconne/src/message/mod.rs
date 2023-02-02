
use mongodb::bson;

use teloxide::{
    payloads::SendMessageSetters,
    requests::{Request, Requester},
    types::Recipient,
    RequestError,
};



pub struct ChatManager {
    pub bot: teloxide::Bot,
    pub post_recipient: Recipient,
}

pub struct SendResult {
    pub url: String,
    pub recipient: Recipient,
}

pub struct Message {
    pub post_id: bson::oid::ObjectId,
    pub text: String,
    pub silent: bool,
    pub results: Vec<SendResult>,
}

pub trait PostMessage {
    fn message(&self) -> Message;
}

impl ChatManager {
    pub fn post_recipient(&self) -> Recipient {
        self.post_recipient.clone()
    }

    async fn send_to<C, M>(&self, mut message: Message, chat_id: C) -> Result<Message, RequestError>
    where
        C: Into<Recipient> + Clone,
    {
        let send_result = self
            .bot
            .send_message(chat_id.clone(), message.text.clone())
            .disable_notification(message.silent)
            .send()
            .await?;
        message.results.push(SendResult {
            url: send_result.url().unwrap().to_string(),
            recipient: chat_id.into(),
        });

        Ok(message)
    }

    pub async fn send_post<M>(&self, post: &M)
    where
        M: PostMessage,
    {
        self.send_to::<Recipient, M>(post.message(), self.post_recipient())
            .await;
    }
}
