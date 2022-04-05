use thiserror::Error;

/// Errors that can happen while using this crate.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Only objects can be flattened")]
    FirstLevelMustBeAnObject,

    #[error("Flattening the object will overwrite the key '{0}'")]
    KeyWillBeOverwritten(String),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}
