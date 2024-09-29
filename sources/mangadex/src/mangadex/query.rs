use serde::{Serialize, Serializer};

use super::enums::*;

pub type Uuid = String;

#[derive(Debug, Serialize)]
pub enum Order {
    #[serde(rename(serialize = "asc"))]
    Ascending,
    #[serde(rename(serialize = "desc"))]
    Descending,
}

#[derive(Debug, Serialize)]
#[serde(rename_all(serialize = "UPPERCASE"))]
pub enum LogicMode {
    And,
    Or,
}

pub mod manga {
    use super::*;

    #[derive(Debug, Serialize)]
    #[serde(rename_all(serialize = "camelCase"))]
    pub struct SortingOrder {
        title: Order,
        year: Order,
        created_at: Order,
        updated_at: Order,
        latest_uploaded_chapter: Order,
        followed_count: Order,
        relevance: Order,
        rating: Order,
    }

    impl SortingOrder {
        pub fn ascending() -> Self {
            SortingOrder {
                title: Order::Ascending,
                year: Order::Ascending,
                created_at: Order::Ascending,
                updated_at: Order::Ascending,
                latest_uploaded_chapter: Order::Ascending,
                followed_count: Order::Ascending,
                relevance: Order::Ascending,
                rating: Order::Ascending,
            }
        }

        pub fn descending() -> Self {
            SortingOrder {
                title: Order::Descending,
                year: Order::Descending,
                created_at: Order::Descending,
                updated_at: Order::Descending,
                latest_uploaded_chapter: Order::Descending,
                followed_count: Order::Descending,
                relevance: Order::Descending,
                rating: Order::Descending,
            }
        }
    }

    impl From<dto::Order> for SortingOrder {
        fn from(value: dto::Order) -> Self {
            match value {
                dto::Order::Ascending => Self::ascending(),
                dto::Order::Descending => Self::descending(),
            }
        }
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all(serialize = "camelCase"))]
    pub struct SearchQuery {
        limit: u32,
        offset: u32,
        title: Option<String>,
        author_or_artist: Option<Uuid>,
        authors: Option<Vec<Uuid>>,
        artists: Option<Vec<Uuid>>,
        year: Option<u32>,
        included_tags: Option<Vec<Uuid>>,
        included_tags_mode: LogicMode,
        excluded_tags: Option<Vec<Uuid>>,
        excluded_tags_mode: LogicMode,
        status: Vec<PublicationStatus>,
        original_language: Option<Vec<String>>,
        excluded_original_language: Option<Vec<String>>,
        available_translated_language: Option<Vec<String>>,
        publication_demographic: Vec<Demographic>,
        ids: Option<Vec<Uuid>>,
        content_rating: Vec<ContentRating>,
        created_at_since: Option<String>,
        updated_at_since: Option<String>,
        order: SortingOrder,
        includes: Option<Vec<String>>,
        #[serde(serialize_with = "serialize_bool")]
        has_available_chapters: bool,
        group: Option<Uuid>,
    }

    impl SearchQuery {
        pub fn new(title: &str) -> Self {
            SearchQuery {
                title: Some(title.to_string()),
                ..Default::default()
            }
        }

        pub fn set_offset(mut self, offset: u32) -> Self {
            self.offset = offset;
            self
        }

        pub fn set_order(mut self, order: SortingOrder) -> Self {
            self.order = order;
            self
        }

        pub fn set_limit(mut self, limit: u32) -> Self {
            self.limit = limit;
            self
        }
    }

    impl Default for SearchQuery {
        fn default() -> Self {
            SearchQuery {
                limit: 20,
                offset: 0,
                title: None,
                author_or_artist: None,
                authors: None,
                artists: None,
                year: None,
                included_tags: None,
                included_tags_mode: LogicMode::And,
                excluded_tags: None,
                excluded_tags_mode: LogicMode::Or,
                status: vec![
                    PublicationStatus::Ongoing,
                    PublicationStatus::Completed,
                    PublicationStatus::Hiatus,
                    PublicationStatus::Cancelled,
                ],
                original_language: None,
                excluded_original_language: None,
                available_translated_language: Some(vec![String::from("en")]),
                publication_demographic: vec![
                    Demographic::Shounen,
                    Demographic::Shoujo,
                    Demographic::Josei,
                    Demographic::Seinen,
                    Demographic::None,
                ],
                ids: None,
                content_rating: vec![
                    ContentRating::Safe,
                    ContentRating::Suggestive,
                    ContentRating::Erotica,
                ],
                created_at_since: None,
                updated_at_since: None,
                order: SortingOrder::descending(),
                includes: None,
                has_available_chapters: true,
                group: None,
            }
        }
    }
}

pub mod chapter {
    use super::*;

    #[derive(Debug, Serialize)]
    #[serde(rename_all(serialize = "camelCase"))]
    pub struct SortingOrder {
        created_at: Order,
        updated_at: Order,
        publish_at: Order,
        readable_at: Order,
        volume: Order,
        chapter: Order,
    }

    impl SortingOrder {
        pub fn ascending() -> Self {
            SortingOrder {
                created_at: Order::Ascending,
                updated_at: Order::Ascending,
                publish_at: Order::Ascending,
                readable_at: Order::Ascending,
                volume: Order::Ascending,
                chapter: Order::Ascending,
            }
        }

        pub fn descending() -> Self {
            SortingOrder {
                created_at: Order::Descending,
                updated_at: Order::Descending,
                publish_at: Order::Descending,
                readable_at: Order::Descending,
                volume: Order::Descending,
                chapter: Order::Descending,
            }
        }
    }

    impl From<dto::Order> for SortingOrder {
        fn from(value: dto::Order) -> Self {
            match value {
                dto::Order::Ascending => Self::ascending(),
                dto::Order::Descending => Self::descending(),
            }
        }
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all(serialize = "camelCase"))]
    pub struct ChapterQuery {
        pub limit: u32,
        pub offset: u32,
        pub translated_language: Option<Vec<String>>,
        pub original_language: Option<Vec<String>>,
        pub exclude_original_language: Option<Vec<String>>,
        pub content_rating: Option<Vec<ContentRating>>,
        pub excluded_groups: Option<Vec<Uuid>>,
        pub excluded_uploaders: Option<Vec<Uuid>>,
        #[serde(serialize_with = "serialize_bool")]
        pub include_future_updates: bool,
        pub created_at_since: Option<String>,
        pub updated_at_since: Option<String>,
        pub publish_at_since: Option<String>,
        pub order: SortingOrder,
        pub includes: Option<Vec<RelationshipType>>,
        #[serde(serialize_with = "serialize_bool")]
        pub include_empty_pages: bool,
        #[serde(serialize_with = "serialize_bool")]
        pub include_future_publish_at: bool,
        #[serde(serialize_with = "serialize_bool")]
        pub include_external_url: bool,
    }

    impl ChapterQuery {
        pub fn new(limit: u32, offset: u32) -> Self {
            ChapterQuery {
                limit,
                offset,
                ..Default::default()
            }
        }

        pub fn set_limit(mut self, limit: u32) -> Self {
            self.limit = limit;
            self
        }

        pub fn set_offset(mut self, offset: u32) -> Self {
            self.offset = offset;
            self
        }

        pub fn set_order(mut self, order: SortingOrder) -> Self {
            self.order = order;
            self
        }
    }

    impl Default for ChapterQuery {
        fn default() -> Self {
            ChapterQuery {
                limit: 100,
                offset: 0,
                translated_language: Some(vec![String::from("en")]),
                original_language: None,
                exclude_original_language: None,
                content_rating: None,
                excluded_groups: None,
                excluded_uploaders: None,
                include_future_updates: true,
                created_at_since: None,
                updated_at_since: None,
                publish_at_since: None,
                order: SortingOrder::ascending(),
                includes: Some(vec![
                    RelationshipType::Manga,
                    RelationshipType::ScanlationGroup,
                    RelationshipType::User,
                ]),
                include_empty_pages: false,
                include_future_publish_at: false,
                include_external_url: false,
            }
        }
    }
}

fn serialize_bool<S>(v: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if *v {
        serializer.serialize_u8(1)
    } else {
        serializer.serialize_u8(0)
    }
}

#[cfg(test)]
mod tests {
    use super::chapter::ChapterQuery;
    use super::manga::SearchQuery;

    #[test]
    fn manga_query() {
        let q = SearchQuery::new("5Toubun no hanayome");
        let result = serde_qs::to_string(&q).unwrap();

        let correct_result = String::from(
            "limit=20\
            &offset=0\
            &title=5Toubun+no+hanayome\
            &includedTagsMode=AND\
            &excludedTagsMode=OR\
            &status[0]=ongoing\
            &status[1]=completed\
            &status[2]=hiatus\
            &status[3]=cancelled\
            &availableTranslatedLanguage[0]=en\
            &publicationDemographic[0]=shounen\
            &publicationDemographic[1]=shoujo\
            &publicationDemographic[2]=josei\
            &publicationDemographic[3]=seinen\
            &publicationDemographic[4]=none\
            &contentRating[0]=safe\
            &contentRating[1]=suggestive\
            &contentRating[2]=erotica\
            &order[title]=desc\
            &order[year]=desc\
            &order[createdAt]=desc\
            &order[updatedAt]=desc\
            &order[latestUploadedChapter]=desc\
            &order[followedCount]=desc\
            &order[relevance]=desc\
            &order[rating]=desc\
            &hasAvailableChapters=1",
        );

        assert_eq!(result, correct_result)
    }

    #[test]
    fn chapter_query() {
        let q = ChapterQuery::default();
        let result = serde_qs::to_string(&q).unwrap();

        let correct_result = String::from(
            "limit=100\
            &offset=0\
            &translatedLanguage[0]=en\
            &includeFutureUpdates=1\
            &order[createdAt]=asc\
            &order[updatedAt]=asc\
            &order[publishAt]=asc\
            &order[readableAt]=asc\
            &order[volume]=asc\
            &order[chapter]=asc\
            &includes[0]=manga\
            &includes[1]=scanlation_group\
            &includes[2]=user\
            &includeEmptyPages=0\
            &includeFuturePublishAt=0\
            &includeExternalUrl=0",
        );

        assert_eq!(result, correct_result)
    }
}
