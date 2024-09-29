pub mod enums;
pub mod query;
pub mod schema;

use color_eyre::{eyre::eyre, Result};
use query::{chapter::ChapterQuery, manga::SearchQuery};
use schema::CustomResult;

use serde::de::DeserializeOwned;
use ureq::{self, Agent, AgentBuilder};

pub struct Mangadex {
    base_url: String,
    agent: Agent,
}

impl Mangadex {
    pub fn new(user_agent: &str) -> Mangadex {
        let agent = AgentBuilder::new().user_agent(user_agent).build();
        let base_url = String::from("https://api.mangadex.org");

        Mangadex { base_url, agent }
    }

    /// Function for sending a GET method to MangaDex API, then deserialize it to T
    fn get<T>(&self, url: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let response: CustomResult<T> = self.agent.get(url).call()?.into_json()?;
        match response {
            CustomResult::Ok(r) => Ok(r),
            CustomResult::Error(e) => Err(eyre!(format!(
                "API Error: {}",
                e.errors.first().unwrap().title
            ))),
        }
    }

    /// Function for searching manga from MangaDex API
    pub(crate) fn search(&self, query: &SearchQuery) -> Result<schema::MangaListContainer> {
        let query_string = serde_qs::to_string(query)?;
        let url = format!("{}/manga?{}", self.base_url, query_string);

        self.get::<schema::MangaListContainer>(&url)
    }

    /// Function for fetching a manga's chapter list from MangaDex API
    pub(crate) fn chapters(
        &self,
        id: &str,
        query: &ChapterQuery,
    ) -> Result<schema::MangaFeedContainer> {
        let query_string = serde_qs::to_string(query)?;
        let url = format!("{}/manga/{}/feed?{}", self.base_url, id, query_string);

        self.get::<schema::MangaFeedContainer>(&url)
    }

    /// Function for fetching a manga details
    pub(crate) fn manga(&self, id: &str) -> Result<schema::MangaContainer> {
        self.get::<schema::MangaContainer>(&format!("{}/manga/{}", self.base_url, id))
    }

    /// Function for fetching a chapter details
    pub(crate) fn chapter(&self, id: &str) -> Result<schema::ChapterContainer> {
        self.get::<schema::ChapterContainer>(&format!("{}/chapter/{}", self.base_url, id))
    }

    /// Function for fetching a chapter's page hash from MangaDex API
    pub(crate) fn page_hash(&self, id: &str) -> Result<schema::PageHash> {
        let url = format!("{}/at-home/server/{}", self.base_url, id);

        self.get::<schema::PageHash>(&url)
    }

    /// Function for fetching a chapter's page hash from MangaDex API
    pub(crate) fn author(&self, id: &str) -> Result<schema::Author> {
        let url = format!("{}/author/{}", self.base_url, id);

        Ok(self.get::<schema::AuthorInfo>(&url)?.data)
    }
}

#[cfg(test)]
pub mod tests {
    use super::query::{chapter, manga};
    use super::*;

    #[test]
    fn manga_search_valid() {
        let query = manga::SearchQuery::new("5Toubun no hanayome");
        let client =
            Mangadex::new("Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0");

        let result = client.search(&query);
        println!("{:#?}", result.unwrap());
    }

    #[test]
    fn manga_search_invalid() {
        let query = manga::SearchQuery::new("aifjaodfaodjf");
        let client =
            Mangadex::new("Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0");

        let result = client.search(&query);
        assert!(result.is_ok())
    }

    #[test]
    fn get_chapters_valid() {
        let query = chapter::ChapterQuery::default();
        let client =
            Mangadex::new("Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0");

        let result = client.chapters("a2febd3e-6252-46eb-bd63-01d51deaaec5", &query);
        assert!(result.is_ok())
    }

    #[test]
    fn get_chapter_invalid() {
        let query = chapter::ChapterQuery::default();
        let client =
            Mangadex::new("Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0");

        let result = client.chapters("6252-46eb-bd63-01d51deaaec5", &query);
        assert!(result.is_err())
    }

    #[test]
    fn get_page_hash() {
        let client =
            Mangadex::new("Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0");

        let result = client.page_hash("1ec5c533-22fa-4422-873d-27549f48389d");
        assert!(result.is_ok())
    }

    #[test]
    fn get_author_valid() {
        let client =
            Mangadex::new("Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0");

        let result = client.author("07a6a131-6567-4472-a08e-3ce84b5fc33a");
        assert!(result.is_ok())
    }

    #[test]
    fn get_author_invalid() {
        let client =
            Mangadex::new("Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0");

        let result = client.author("22fa-4422-873d-27549f48389d");
        assert!(result.is_err())
    }
}
