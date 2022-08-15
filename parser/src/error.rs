use core::num::dec2flt::ParseFloatError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid bank name `{0}`")]
    BankNotFound(String),
    #[error("currency with name `{0}` not found")]
    CurrencyNotFound(String),
    #[error("Bank parsing error")]
    BankParseFail,
    #[error("Currency parsing error")]
    CurrencyParseFail(#[from] ParseFloatError),
    #[error("http error")]
    Http(#[from] reqwest::Error),
    #[error("JSON Serialization error")]
    JsonSerialization(#[from] serde_json::Error),
}
