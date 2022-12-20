// #[serde_as]
#[derive(Clone, Debug)]
pub struct MessageData {
    /// Text
    pub text: String,
    /// Silent?
    pub silent: bool,
}

impl MessageData {
    pub async fn send(
        self,
        bot: teloxide::Bot,
        chat_id: Recipient,
    ) -> Result<teloxide::types::Message, Error> {
        let text = self.build_message();
        let send_result = bot
            .send_message(chat_id, text)
            .parse_mode(ParseMode::Html)
            .disable_notification(self.silent)
            .send()
            .await?;

        Ok(send_result)
    }
}

impl Post {
    pub fn update_announce(
        &mut self,
        announce: Announce,
        information: &InformationPage,
        api_id: String,
        telegraph_url: &str,
    ) {
        self.sources.announce.insert(api_id, announce.announce_id);
        Self::insert_tags_announce(&mut self.tags, information);

        self.update_time = information
            .date
            .map_or(announce.replace_time, |d| d.with_timezone(&chrono::Utc));

        self.telegraph = Some(telegraph_url.to_string());
        self.title = information.title.clone();
        self.mapped_title = map_titie(&self.title);
        self.events = information.events.clone();
    }
}