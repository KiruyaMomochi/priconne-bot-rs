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
    CronError(#[from] tokio_cron_scheduler::JobSchedulerError),
    #[error("addr parse error")]
    AddrParseError(#[from] std::net::AddrParseError),
    #[error("axum error")]
    AxumError(#[from] axum::Error),
    #[error("hyper error")]
    HyperError(#[from] hyper::Error),
    #[error("IO Error")]
    IOError(#[from] std::io::Error),
    #[error("serde_yaml Error")]
    SerdeYamlError(#[from] serde_yaml::Error),
    #[error("tokio cron `{0}` error")]
    SendError(String),
    #[error("kuchikiki error")]
    KuchikiError,
    #[error("no api server")]
    NoApiServer,
    #[error("the article has no title")]
    EmptyTitleError,
    #[error("source is invalid")]
    InvalidSource,
    #[error("failed to parse string to resource kind {0}")]
    ParseResourceKindsError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
