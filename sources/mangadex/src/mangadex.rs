pub mod enums;
pub mod schema;
pub mod error;
pub mod query;

use schema::{ChapterResponse, MangaResponse, PageResponse, CustomResult};
use error::Error;
use query::{chapter::ChapterQuery, manga::MangaQuery};

use reqwest::{Client, ClientBuilder};

pub struct Mangadex {
    base_url: String,
    client: Client,
}

impl Mangadex {
    pub fn new(user_agent: &str) -> Mangadex {
        let client = ClientBuilder::new()
            .user_agent(user_agent)
            .build()
            .unwrap();

        let base_url = String::from("https://api.mangadex.org");

        Mangadex {
            base_url,
            client
        }
    }

    /// Function for searching manga from MangaDex API
    pub async fn search(&self, query: &MangaQuery) -> Result<MangaResponse, Error> {
        let query_string = serde_qs::to_string(query).unwrap();
        let url = format!("{}/manga?{}", self.base_url, query_string);

        let raw_response = self.client.get(url)
            .send().await?
            .bytes().await?;

        let response: CustomResult<MangaResponse> = serde_json::from_slice(&raw_response)?;

        match response {
            CustomResult::Ok(o) => Result::Ok(o),
            CustomResult::Err(e) => Result::Err(e.into())
        }
    }

    /// Function for fetching a manga's chapter list from MangaDex API
    pub async fn get_chapters(&self, id: &str, query: &ChapterQuery) -> Result<ChapterResponse, Error> {
        let query_string = serde_qs::to_string(query).unwrap();
        let url = format!("{}/manga/{}/feed?{}", self.base_url, id, query_string);

        let raw_response = self.client.get(url)
            .send().await?
            .bytes().await?;

        let response: CustomResult<ChapterResponse> = serde_json::from_slice(&raw_response)?;

        match response {
            CustomResult::Ok(o) => Result::Ok(o),
            CustomResult::Err(e) => Result::Err(e.into())
        }
    }

    /// Function for fetching a chapter's page hash from MangaDex API
    pub async fn get_page_hash(&self, id: &str) -> Result<PageResponse, Error> {
        let url = format!("{}/at-home/server/{}", self.base_url, id);

        let raw_response = self.client.get(url)
            .send().await?
            .bytes().await?;

        let response: CustomResult<PageResponse> = serde_json::from_slice(&raw_response)?;

        match response {
            CustomResult::Ok(o) => Result::Ok(o),
            CustomResult::Err(e) => Result::Err(e.into())
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::query::{manga, chapter};
    use super::*;

    #[test]
    fn manga_search_valid() {
        let query = manga::MangaQuery::new("5Toubun no hanayome");
        let client = Mangadex::new("Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0");

        let runtime = tokio::runtime::Runtime::new()
            .unwrap();

        let result = runtime.block_on(client.search(&query));
        assert!(result.is_ok())
    }

    #[test]
    fn manga_search_invalid() {
        let query = manga::MangaQuery::new("aifjaodfaodjf");
        let client = Mangadex::new("Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0");

        let runtime = tokio::runtime::Runtime::new()
            .unwrap();

        let result = runtime.block_on(client.search(&query));
        assert!(result.is_ok())
    }

    #[test]
    fn get_chapters_valid() {
        let query = chapter::ChapterQuery::default();
        let client = Mangadex::new("Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0");

        let runtime = tokio::runtime::Runtime::new()
            .unwrap();

        let result = runtime.block_on(client.get_chapters("a2febd3e-6252-46eb-bd63-01d51deaaec5", &query));
        assert!(result.is_ok())
    }

    #[test]
    fn get_chapter_invalid() {
        let query = chapter::ChapterQuery::default();
        let client = Mangadex::new("Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0");

        let runtime = tokio::runtime::Runtime::new()
            .unwrap();

        let result = runtime.block_on(client.get_chapters("lma0-6252-46eb-bd63-01d51deaaec5", &query));
        assert!(result.is_err())
    }

    #[test]
    fn get_page_hash() {
        let client = Mangadex::new("Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0");

        let runtime = tokio::runtime::Runtime::new()
            .unwrap();

        let result = runtime.block_on(client.get_page_hash("1ec5c533-22fa-4422-873d-27549f48389d"));
        assert!(result.is_ok())
    }
}
