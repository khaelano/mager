pub mod enums;
pub mod error;
pub mod query;
pub mod schema;

use error::Error;
use query::{chapter::ChapterQuery, manga::MangaQuery};
use schema::{Author, AuthorInfo, CustomResult, MangaFeed, MangaList, PageHash};

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
    fn get<T>(&self, url: &str) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let response: CustomResult<T> = self.agent.get(url).call()?.into_json()?;
        match response {
            CustomResult::Ok(r) => Ok(r),
            CustomResult::Error(e) => Err(Error::Api(format!(
                "API Error: {}",
                e.errors.first().unwrap().title
            ))),
        }
    }

    /// Function for searching manga from MangaDex API
    pub fn search(&self, query: &MangaQuery) -> Result<MangaList, Error> {
        let query_string = serde_qs::to_string(query).unwrap();
        let url = format!("{}/manga?{}", self.base_url, query_string);

        Ok(self.get::<MangaList>(&url)?)
    }

    /// Function for fetching a manga's chapter list from MangaDex API
    pub fn chapters(&self, id: &str, query: &ChapterQuery) -> Result<MangaFeed, Error> {
        let query_string = serde_qs::to_string(query).unwrap();
        let url = format!("{}/manga/{}/feed?{}", self.base_url, id, query_string);

        Ok(self.get::<MangaFeed>(&url)?)
    }

    /// Function for fetching a chapter's page hash from MangaDex API
    pub fn page_hash(&self, id: &str) -> Result<PageHash, Error> {
        let url = format!("{}/at-home/server/{}", self.base_url, id);

        self.get::<PageHash>(&url)
    }

    /// Function for fetching a chapter's page hash from MangaDex API
    pub fn author(&self, id: &str) -> Result<Author, Error> {
        let url = format!("{}/author/{}", self.base_url, id);

        Ok(self.get::<AuthorInfo>(&url)?.data)
    }
}

#[cfg(test)]
pub mod tests {
    use super::query::{chapter, manga};
    use super::*;

    #[test]
    fn manga_search_valid() {
        let query = manga::MangaQuery::new("5Toubun no hanayome");
        let client =
            Mangadex::new("Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0");

        let result = client.search(&query);
        println!("{:#?}", result.unwrap());
    }

    #[test]
    fn manga_search_invalid() {
        let query = manga::MangaQuery::new("aifjaodfaodjf");
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
