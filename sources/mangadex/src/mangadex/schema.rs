use std::collections::HashMap;
use serde::Deserialize;

use super::{enums::*, query::Uuid};

type LocalizedString = HashMap<String, String>;
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum CustomResult<T> {
    Ok(T),
    Err(ErrorResponse)
}

#[derive(Deserialize, Debug)]
pub struct ErrorResponse {
    pub result: String,
    pub errors: Vec<MDError>
}

impl std::fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.errors.first().unwrap().title)
    }
}

#[derive(Deserialize, Debug)]
pub struct MangaResponse {
    pub result: String,
    pub response: String,
    pub data: Vec<Manga>,
    pub limit: u32,
    pub offset: u32,
    pub total: u32
}

#[derive(Deserialize, Debug)]
pub struct ChapterResponse {
    pub result: String,
    pub response: String,
    pub data: Vec<Chapter>,
    pub limit: u32,
    pub offset: u32,
    pub total: u32
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PageResponse {
    pub result: String,
    pub base_url: String,
    pub chapter: ChapterPages
}

#[derive(Deserialize, Debug)]
pub struct AuthorResponse {
    pub result: String,
    pub response: String,
    pub data: Author
}

#[derive(Deserialize, Debug)]
pub struct MDError {
    pub id: String,
    pub status: u32,
    pub title: String,
    pub detail: Option<String>,
    pub context: Option<String>
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Relationship {
    pub id: String,
    #[serde(rename = "type")]
    pub rel_type: RelationshipType,
    pub related: Option<MangaRelationshipType>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Manga {
    pub id: String,
    #[serde(rename = "type")]
    pub entity_type: String,
    pub attributes: MangaAttr,
    pub relationships: Option<Vec<Relationship>>
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MangaAttr {
    pub title: LocalizedString,
    pub alt_titles: Option<Vec<LocalizedString>>,
    pub description: Option<LocalizedString>,
    pub is_locked: bool,
    pub links: Option<HashMap<String, String>>,
    pub original_language: String,
    pub last_volume: Option<String>,
    pub last_chapter: Option<String>,
    pub publication_demographic: Option<Demographic>,
    pub status: PublicationStatus,
    pub year: Option<u32>,
    pub content_rating: ContentRating,
    pub state: State,
    pub chapter_numbers_reset_on_new_volume: bool,
    pub created_at: String,
    pub updated_at: String,
    pub version: u32,
    pub available_translated_languages: Option<Vec<String>>,
    pub latest_uploaded_chapter: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: String,
    #[serde(rename = "type")]
    pub rel_type: RelationshipType,
    pub attributes: TagAttr,
    pub relationships: Option<Vec<Relationship>>
}

#[derive(Deserialize, Debug)]
pub struct TagAttr {
    pub name: LocalizedString,
    pub description: LocalizedString,
    pub group: TagGroup,
    pub version: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Chapter {
    pub id: String,
    #[serde(rename = "type")]
    pub rel_type: String,
    pub attributes: ChapterAttr,
    pub relationships: Option<Vec<Relationship>>
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChapterAttr {
    pub title: Option<String>,
    pub volume: Option<String>,
    pub chapter: Option<String>,
    pub pages: u32,
    pub translated_language: String,
    pub external_url: Option<String>,
    pub version: u32,
    pub created_at: String,
    pub updated_at: String,
    pub publish_at: String,
    pub readable_at: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChapterPages {
    pub hash: String,
    pub data: Vec<String>,
    pub data_saver: Vec<String>
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub rel_type: RelationshipType,
    pub attributes: AuthorAttr,
    pub relationships: Option<Vec<Relationship>>
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AuthorAttr {
    pub name: String,
    pub image_url: Option<String>,
    pub biography: Option<LocalizedString>,
    pub twitter: Option<String>,
    pub pixiv: Option<String>,
    pub melon_book: Option<String>,
    pub fan_box: Option<String>,
    pub booth: Option<String>,
    pub nico_video: Option<String>,
    pub skeb: Option<String>,
    pub fantia: Option<String>,
    pub tumblr: Option<String>,
    pub youtube: Option<String>,
    pub weibo: Option<String>,
    pub naver: Option<String>,
    pub namicomi: Option<String>,
    pub website: Option<String>,
    pub version: u32,
    pub created_at: String,
    pub updated_at: String
}