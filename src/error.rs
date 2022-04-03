use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    // #[error("{0}")]
    // SerdeJsonError(String),
    //
    // #[error("{0}")]
    // RequestData(String),
    //
    // #[error("{0}")]
    // Action(String),
    //
    // #[error("{0}")]
    // Serialization(String),
    //
    // #[error("{0}")]
    // CurrencyError(String),
    //
    // #[error(transparent)]
    // Utf8Error(#[from] str::Utf8Error),
    //
    // #[error(transparent)]
    // IoError(#[from] io::Error),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
    // #[error(transparent)]
    // DateError(#[from] chrono::ParseError),
    //
    // #[error("Amount not parseable")]
    // ParseIntError(#[from] std::num::ParseIntError),
    //
    // #[error("action code '{num:?}': {msg:?}")]
    // ActionCode { num: i32, msg: String },
}
