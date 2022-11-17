mod information;
mod cartoon;
mod news;
mod sent;

use resource::{
    article::cartoon::Thumbnail,
    article::information::Announce,
    article::news::News,
    event::EventPeriod,
    message::SentMessage
};

mod collections {
    pub const SENT_MESSAGE: &str = "sent_message";
    pub const CARTOON: &str = "cartoon";
    pub const INFORMATION: &str = "announce";
    pub const NEWS: &str = "news";
}

#[derive(Debug, Clone)]
pub struct Db {
    pub database: mongodb::Database,
}

impl Db {
    pub fn new(database: mongodb::Database) -> Self {
        Self { database }
    }

    fn sent_messages(&self) -> mongodb::Collection<SentMessage> {
        self.database.collection(collections::SENT_MESSAGE)
    }
    fn cartoons(&self) -> mongodb::Collection<Thumbnail> {
        self.database.collection(collections::CARTOON)
    }
    fn announces(&self) -> mongodb::Collection<Announce> {
        self.database.collection(collections::INFORMATION)
    }
    fn news(&self) -> mongodb::Collection<News> {
        self.database.collection(collections::NEWS)
    }
}

fn run() {}
