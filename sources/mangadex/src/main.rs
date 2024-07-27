use std::sync::Arc;
use std::net::TcpListener;

use dto::{Author, Chapter, Filter, Manga, MangaList};
use mangadex::enums::RelationshipType;
use mangadex::error::Error;
use mangadex::query::{chapter::ChapterQuery, manga::MangaQuery};
use mangadex::schema::{Chapter as MDChapter, Manga as MDManga};
use mangadex::Mangadex;

mod mangadex;

pub fn main() {
    let listener = TcpListener::bind("127.0.0.1:3232").unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let port = stream.local_addr().unwrap().port();


    }
}


fn chapters(
    id: &str,
    page: u32,
    filter: Filter,
    user_agent: &str,
) -> Result<Vec<Chapter>, Error> {
    let client = Arc::new(Mangadex::new(user_agent));
    let limit = 50;
    let offset = (page - 1) * 50;
    let query = ChapterQuery::new(limit, offset).set_order(filter.sort.into());

    let response = client.chapters(id, &query).unwrap();

    let mut chapters = Vec::new();
    for ch in response.data {
        chapters.push(convert_chapter(ch).unwrap());
    }

    Ok(chapters)
}

fn convert_chapter(md_chapter: MDChapter) -> Result<Chapter, Error> {
    let url = format!("https://api.mangadex.org/at-home/server/{}", md_chapter.id);
    let title = md_chapter
        .attributes
        .title
        .unwrap_or(String::from("No Title"));
    let number = md_chapter
        .attributes
        .chapter
        .and_then(|s| Some(s.clone()))
        .unwrap_or(String::from("No Chapter"));

    let language = String::from("en");

    Ok(Chapter {
        url,
        title,
        number,
        language,
    })
}

fn get_chapter_pages(client: Arc<Mangadex>, id: &str) -> Result<Vec<String>, Error> {
    let result = client.page_hash(id).unwrap();

    let mut urls = Vec::new();
    for h in result.chapter.data {
        urls.push(format!(
            "https://{}/data/{}/{}",
            result.base_url, result.chapter.hash, h
        ))
    }

    Ok(urls)
}

fn search(
    keyword: &str,
    page: u32,
    filter: Filter,
    user_agent: &str,
) -> Result<MangaList, Error> {
    let client = Arc::new(Mangadex::new(user_agent));
    let limit = 10;
    let query = &MangaQuery::new(keyword)
        .set_limit(limit)
        .set_offset((page - 1) * limit)
        .set_order(filter.sort.clone().into());

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .build()
        .unwrap();

    let response = client.search(query).unwrap(); // Fix this ass bro

    // This is all concurrency bulshit
    let mut handles = Vec::new();
    for md_manga in response.data {
        handles.push(rt.spawn(convert_manga(client.clone(), md_manga)));
    }

    let mut data = Vec::new();
    for h in handles {
        data.push(
            rt.block_on(h).unwrap().unwrap(), // Fix this bro
        );
    }

    Ok(MangaList {
        data,
        page,
        total_page: response.total / limit,
    })
}

/// This function will extract the author and artist from the manga's relation list
async fn extract_author(client: Arc<Mangadex>, md_manga: &MDManga) -> Vec<Author> {
    let mut handles = Vec::new();

    for rel in md_manga.relationships.as_ref().unwrap() {
        // Checks if the relation is artist or author
        if let RelationshipType::Artist | RelationshipType::Author = rel.rel_type {
            let details = match rel.rel_type {
                RelationshipType::Artist => String::from("Artist"),
                RelationshipType::Author => String::from("Author"),
                _ => unreachable!(),
            };

            let cl = client.clone();
            let rl = rel.clone();
            let author = tokio::spawn(async move {
                let name = cl.author(&rl.id).unwrap().data.attributes.name;

                Author { name, details }
            });
            handles.push(author);
        }
    }

    let mut authors = Vec::new();
    for handle in handles {
        authors.push(handle.await.unwrap());
    }

    authors
}

async fn extract_cover(
    client: Arc<Mangadex>,
    md_manga: &MDManga,
    query: &MangaQuery,
) -> Result<String, Error> {
    todo!()
}

async fn convert_manga(client: Arc<Mangadex>, md_manga: MDManga) -> Result<Manga, Error> {
    let authors = extract_author(client.clone(), &md_manga).await;
    let attr = &md_manga.attributes;

    let url = format!("https://api.mangadex.org/manga/{}", md_manga.id);
    let title = attr
        .title
        .get("en")
        .cloned()
        .or_else(|| attr.title.keys().next().cloned())
        .unwrap_or(String::from("Unknown Title"));

    let description = attr
        .description
        .as_ref()
        .and_then(|desc| desc.get("en"))
        .cloned()
        .unwrap_or(String::from("No description"));

    let original_language = attr.original_language.clone();
    // For now, it only supports english language
    let language = String::from("en");
    let status = md_manga.attributes.status.to_dto();

    Ok(Manga {
        url,
        title,
        authors,
        original_language,
        language,
        description,
        status,
    })
}
