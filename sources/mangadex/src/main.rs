use std::convert::Infallible;

use dto::{Author, Carrier, Filter, Manga, MangaList, PublicationStatus};
use mangadex::{enums::RelationshipType, error::Error, query::{self, manga::{MangaQuery, SortingOrder}}, Mangadex};

mod mangadex;

pub fn main() {
    let manga = search(
        "5Toubun no hanayome",
        0,
        &Filter { language: "en".to_string(), sort: dto::Order::Descending },
        "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0"
    )
    .unwrap();

    println!("{:#?}", manga);
}

fn search(keyword: &str, page: u32, filter: &Filter, user_agent: &str) -> Result<Carrier<MangaList>, Error> {
    let client = Mangadex::new(user_agent);
    let query = &MangaQuery::new(keyword)
        .set_offset(page * 10)
        .set_order(filter.sort.clone().into());

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .unwrap();

    let response = runtime.block_on(client.old_search(query))?;

    let mut mangas : Vec<Manga> = Vec::new();
    for m in response.data {
        let url = format!("https://api.mangadex.org/manga/{}", m.id);
        let mut attributes = m.attributes;

        let authors: Vec<Author> = m.relationships.unwrap()
            .iter()
            .filter(|r| matches!(r.rel_type, RelationshipType::Artist | RelationshipType::Author))
            .filter_map(|r| {
                let author_data = runtime.block_on(client.old_get_author(&r.id)).unwrap().data;

                let author_name = author_data.attributes.name;

                match r.rel_type {
                RelationshipType::Artist => Some(Author { name: author_name, details: String::from("Artist") }),
                RelationshipType::Author => Some(Author { name: author_name, details: String::from("Author") }),
                _ => None
            }})
            .collect();

        let manga = Manga {
            title: attributes.title.remove("en").unwrap(),
            description: attributes.description.unwrap().remove("en").unwrap_or("no description".to_string()),
            language: String::from("en"),
            status: attributes.status.to_dto(),
            original_language: attributes.original_language,
            authors,
            url
        };

        mangas.push(manga);
    }

    Ok(Carrier {
        source: "MangaDex".to_string(),
        data: MangaList {
            page,
            total_page: response.total / 10,
            data: mangas
        }
    })
}
