mod page;
pub mod service;
pub use page::*;
use reqwest::Url;

use crate::message::{Message, Sendable};

pub struct Cartoon {
    pub id: i32,
    pub episode: String,
    pub title: String,
    pub image_src: Url,
}

impl Cartoon {
    pub fn caption(&self) -> String {
        format!(
            "<b>第 {episode} 話</b>: {title}\n{image_src} <code>#{id}</code>",
            episode = self.episode,
            title = self.title,
            image_src = self.image_src,
            id = self.id
        )
    }
}

impl Sendable for Cartoon {
    fn message(&self) -> Message {
        Message {
            text: self.caption(),
            silent: false,
            image_src: Some(self.image_src.clone()),
        }
    }
}
