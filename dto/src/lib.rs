use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Carrier<T> {
    pub source: String,
    pub data: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Filter {
    pub language: String,
    pub sort: Order,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Order {
    Ascending,
    Descending,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PublicationStatus {
    Ongoing,
    Completed,
    Hiatus,
    Cancelled,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Author {
    pub name: String,
    pub details: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MangaList {
    pub page: u32,
    pub total_page: u32,
    pub data: Vec<Manga>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Manga {
    pub url: String,
    pub title: String,
    pub authors: Vec<Author>,
    pub original_language: String,
    pub language: String,
    pub description: String,
    pub status: PublicationStatus,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Chapter {
    pub url: String,
    pub title: String,
    pub number: String,
    pub language: String,
}

pub type ChapterPages = Vec<String>;
