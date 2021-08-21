mod tagger;

pub use tagger::{map_titie, Tagger};

pub trait MessageBuilder {
    fn build_message(&self) -> String;
}

