mod error;
mod event;
mod page;
mod service;
mod tagging;
mod utils;

mod database;
pub mod resource;
pub mod message;

pub use error::Error;
pub use page::Page;
pub use service::*;
pub use tagging::{extract_tag, RegexTagger};

// Use of a mod or pub mod is not actually necessary.
pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

mod client {
    pub fn ua() -> String {
        format!(
            "priconne-bot-rs/{} {} {}",
            crate::built_info::PKG_VERSION,
            crate::built_info::TARGET,
            "Android"
        )
    }
}
