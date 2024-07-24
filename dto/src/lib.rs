use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum PublicationStatus {
    Ongoing,
    Completed,
    Hiatus,
    Cancelled
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Author {
    pub name: String,
    pub details: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Manga {
    pub url: String,
    pub title: String,
    pub authors: Vec<Author>,
    pub original_language: String,
    pub language: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Chapter {
    pub url: String,
    pub title: String,
    pub number: String,
    pub original_language: String,
    pub language: String,
    pub pages_url: Vec<String>
}
