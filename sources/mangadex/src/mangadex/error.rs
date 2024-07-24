use std::fmt::Display;

use super::schema;

/// Enum that represents the kind of error in this crate
#[derive(Debug)]
pub enum Error {
    Request(reqwest::Error),
    Serde(serde_json::Error),
    Api(String)
}

impl std::error::Error for Error{}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Request(err) => write!(f, "Request Error: {}", err),
            Error::Serde(err) => write!(f, "Error parsing JSON: {}", err),
            Error::Api(msg) => write!(f, "Error getting API response: {}", msg)
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::Request(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Serde(value)
    }
}

impl From<schema::ErrorResponse> for Error {
    fn from(value: schema::ErrorResponse) -> Self {
        let msg = value.errors.first()
            .unwrap()
            .title
            .clone();

        Error::Api(msg)
    }
}
