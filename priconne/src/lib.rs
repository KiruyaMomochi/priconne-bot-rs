#![feature(trait_alias)]

mod error;
mod page;
mod utils;

mod database;

pub mod config;
pub mod insight;
pub mod message;
pub mod client;
pub mod service;

pub mod resource;

pub use error::{Error, Result};
pub use page::Page;

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
