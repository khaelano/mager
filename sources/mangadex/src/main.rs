use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::{env, io};

use serde_json;

use dto::carriers::{self, Request, Response, Status};
use dto::{Author, Chapter, ChapterList, Filter, Manga, MangaList};

use mangadex::enums::RelationshipType;
use mangadex::error::Error;
use mangadex::query::{chapter::ChapterQuery, manga::MangaQuery};
use mangadex::schema::{Chapter as MDChapter, Manga as MDManga};
use mangadex::Mangadex;

mod mangadex;

pub fn main() {
    let args: Vec<String> = env::args().collect();
    let port = args.get(1).unwrap();

    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).unwrap();
    // println!("listening at port {port}");

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();

        let mut length: [u8; 4] = [0; 4];
        stream.read_exact(&mut length).unwrap();

        let mut request = vec![0; u32::from_ne_bytes(length) as usize];
        stream.read_exact(&mut request).unwrap();

        let request: Request = serde_json::from_slice(&request).unwrap();
        handle_request(request, stream);
    }
}

fn handle_request(request: Request, mut stream: TcpStream) {
    let client_name = String::from("MangaDex");
    let user_agent = "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0";
    let response = match request.command {
        carriers::Command::Search {
            keyword,
            page,
            filter,
        } => {
            let mangas = search(&keyword, page, filter, user_agent).unwrap();
            let content = Response {
                status: Status::Ok,
                reason: String::from("Feeling good today aren't we?"),
                source_name: client_name.clone(),
                content: mangas,
            };
            serde_json::to_string(&content).unwrap()
        }
        carriers::Command::Chapters {
            identifier,
            page,
            filter,
        } => {
            let chapters = chapters(&identifier, page, filter, user_agent).unwrap();
            let content = Response {
                status: Status::Ok,
                reason: String::from("Feeling good today aren't we?"),
                source_name: client_name.clone(),
                content: chapters,
            };
            serde_json::to_string(&content).unwrap()
        }
        carriers::Command::Pages { identifier } => {
            let pages = get_chapter_pages(&identifier, user_agent).unwrap();
            let content = Response {
                status: Status::Ok,
                reason: String::from("Feeling good today aren't we?"),
                source_name: client_name.clone(),
                content: pages,
            };
            serde_json::to_string(&content).unwrap()
        }
        carriers::Command::Ping => {
            let content = Response {
                status: Status::Ok,
                reason: String::from("Pong, this source is active"),
                source_name: client_name.clone(),
                content: (),
            };
            serde_json::to_string(&content).unwrap()
        }
    };

    write_to_stream(&mut stream, &response).unwrap();
}

fn write_to_stream(stream: &mut TcpStream, payload: &str) -> Result<(), io::Error> {
    let size = payload.len() as u32;
    stream.write_all(&size.to_ne_bytes())?;
    stream.write_all(payload.as_bytes())?;
    stream.flush()?;

    Ok(())
}

fn chapters(id: &str, page: u32, filter: Filter, user_agent: &str) -> Result<ChapterList, Error> {
    let client = Arc::new(Mangadex::new(user_agent));
    let limit = 50;
    let offset = (page - 1) * 50;
    let query = ChapterQuery::new(limit, offset).set_order(filter.sort.into());

    let response = client.chapters(id, &query).unwrap();

    let mut chapters = Vec::new();
    for ch in response.data {
        chapters.push(convert_chapter(ch).unwrap());
    }

    Ok(ChapterList {
        page,
        total_page: (response.total + limit - 1) / limit,
        data: chapters,
    })
}

fn convert_chapter(md_chapter: MDChapter) -> Result<Chapter, Error> {
    let identifier = md_chapter.id;
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
        identifier,
        title,
        number,
        language,
    })
}

fn get_chapter_pages(id: &str, user_agent: &str) -> Result<Vec<String>, Error> {
    let client = Arc::new(Mangadex::new(user_agent));
    let result = client.page_hash(id).unwrap();

    let mut urls = Vec::new();
    for h in result.chapter.data {
        urls.push(format!(
            "{}/data/{}/{}",
            result.base_url, result.chapter.hash, h
        ))
    }

    Ok(urls)
}

fn search(keyword: &str, page: u32, filter: Filter, user_agent: &str) -> Result<MangaList, Error> {
    let client = Arc::new(Mangadex::new(user_agent));
    let limit = 20;
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
        total_page: (response.total + limit - 1) / limit,
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
                let name = cl.author(&rl.id).unwrap().attributes.name;

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

async fn convert_manga(client: Arc<Mangadex>, md_manga: MDManga) -> Result<Manga, Error> {
    let authors = extract_author(client.clone(), &md_manga).await;
    let attr = &md_manga.attributes;

    let title = attr
        .title
        .get("en")
        .cloned()
        .or_else(|| {
            let key: Vec<&String> = attr.title.keys().collect();
            attr.title.get(key.first().cloned().unwrap()).cloned()
        })
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
        identifier: md_manga.id,
        title,
        authors,
        original_language,
        language,
        description,
        status,
    })
}
