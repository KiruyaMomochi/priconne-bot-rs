#![feature(trait_alias)]
// #![feature(min_specialization)]

mod error;
mod page;
mod utils;

pub mod database;

pub mod chat;
pub mod client;
pub mod config;
pub mod insight;
pub mod service;

pub mod resource;

pub use error::{Error, Result};
pub use page::Page;
pub use service::PriconneService;

// Use of a mod or pub mod is not actually necessary.
pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub fn ua() -> String {
    format!(
        "priconne-bot-rs/{} {} {}",
        crate::built_info::PKG_VERSION,
        crate::built_info::TARGET,
        "Android"
    )
}
