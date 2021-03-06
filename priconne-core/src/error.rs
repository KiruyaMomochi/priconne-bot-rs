use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse the url")]
    UrlParseError(#[from] url::ParseError),
    #[error("request not complete")]
    ReqwestError(#[from] reqwest::Error),
    #[error("database failure")]
    MongoError(#[from] mongodb::error::Error),
    #[error("error when sending telegram message")]
    TeloxideRequestError(#[from] teloxide::RequestError),
    #[error("telegraph error")]
    TelegraphError(#[from] telegraph_rs::Error),
    #[error("failed to parse json")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("failed to parse time")]
    ParseError(#[from] chrono::ParseError),
    #[error("cannot parse number to int")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("regex error")]
    RegexError(#[from] regex::Error),
    #[error("cron error")]
    CronError(#[from] cron::error::Error),
    #[error("channel send error")]
    SendError,
    #[error("kuchiki error")]
    KuchikiError,
    #[error("no api server")]
    NoApiServer,
    #[error("the article has no title")]
    EmptyTitleError,
}
