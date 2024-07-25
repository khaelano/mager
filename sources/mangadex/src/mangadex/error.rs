use std::{fmt::Display, io};

use super::schema;

/// Enum that represents the kind of error in this crate
#[derive(Debug)]
pub enum Error {
    OldRequest(reqwest::Error),
    Request(ureq::Error),
    Serde(serde_json::Error),
    Api(String),
    Io(io::Error)
}

impl std::error::Error for Error{}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::OldRequest(err) => write!(f, "Request Error: {}", err),
            Error::Serde(err) => write!(f, "Error parsing JSON: {}", err),
            Error::Api(msg) => write!(f, "Error getting API response: {}", msg),
            Error::Request(err) => write!(f, "Request Error: {}", err),
            Error::Io(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::OldRequest(value)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Io(value)
    }
}

impl From<ureq::Error> for Error {
    fn from(value: ureq::Error) -> Self {
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
