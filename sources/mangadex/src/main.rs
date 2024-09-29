use color_eyre::eyre::{eyre, Report};
use color_eyre::Result;
use futures::future;
use tokio::task::JoinHandle;

use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

use dto::carriers::{self, Request, Response, Status};
use dto::*;

use mangadex::enums::RelationshipType;
use mangadex::query::{chapter::ChapterQuery, manga::SearchQuery};
use mangadex::schema::{self, Manga as MDManga};
use mangadex::Mangadex;

mod mangadex;

#[tokio::main]
pub async fn main() -> Result<()> {
    color_eyre::install()?;
    let args: Vec<String> = env::args().collect();
    let port = args.get(1).unwrap();

    let listener = TcpListener::bind(format!("127.0.0.1:{port}"))?;

    for stream in listener.incoming() {
        let mut stream = stream?;

        let mut length: [u8; 4] = [0; 4];
        stream.read_exact(&mut length)?;

        let mut request = vec![0; u32::from_ne_bytes(length) as usize];
        stream.read_exact(&mut request)?;

        let request: Request = serde_json::from_slice(&request)?;
        handle_request(request, stream).await?;
    }
    Ok(())
}

async fn handle_request(request: Request, mut stream: TcpStream) -> Result<()> {
    let client_name = String::from("MangaDex");
    let user_agent = "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0";
    let response = match request.command {
        carriers::Command::Ping => {
            let content = Response {
                status: Status::Ok,
                reason: String::from("Pong, this source is active"),
                source_name: client_name.clone(),
                content: Some(()),
            };
            serde_json::to_string(&content)?
        }
        carriers::Command::Search {
            keyword,
            page,
            filter,
        } => {
            let content = search(&keyword, page, filter, user_agent);
            serde_json::to_string(&content)?
        }
        carriers::Command::FetchManga { manga_identifier } => {
            let content = fetch_manga(&manga_identifier, user_agent).await;
            serde_json::to_string(&content)?
        }
        carriers::Command::FetchChapterList {
            identifier,
            page,
            filter,
        } => {
            let content = fetch_chapter_list(&identifier, page, filter, user_agent);
            serde_json::to_string(&content)?
        }
        carriers::Command::FetchChapter { chapter_identifier } => {
            let content = fetch_chapter(&chapter_identifier, user_agent);
            serde_json::to_string(&content)?
        }
    };

    write_to_stream(&mut stream, &response)
}

fn search(keyword: &str, page: u32, filter: Filter, user_agent: &str) -> Response<MangaList> {
    let client = Arc::new(Mangadex::new(user_agent));
    let source_name = "MangaDex".to_string();
    let limit = 20;
    let query = &SearchQuery::new(keyword)
        .set_limit(limit)
        .set_offset((page - 1) * limit)
        .set_order(filter.sort.clone().into());

    let mglist_cont = match client.search(query) {
        Ok(mglist_cont) => mglist_cont,
        Err(report) => return create_error_response(report, &source_name),
    };

    let total_page = (mglist_cont.total + limit - 1) / limit;
    let data = mglist_cont
        .data
        .into_iter()
        .map(|mg| {
            let title = extract_title("en", &mg).unwrap_or(String::from("Unknown Title"));
            let identifier = mg.id;
            let attr = mg.attributes;
            let status = attr.status.to_dto();

            MangaListEntry {
                identifier,
                title,
                status,
            }
        })
        .collect::<Vec<MangaListEntry>>();

    Response {
        status: Status::Ok,
        reason: "All good".to_string(),
        source_name,
        content: Some(MangaList {
            page,
            total_page,
            data,
        }),
    }
}

/// This function will fetch manga details for a specified manga id
async fn fetch_manga(id: &str, user_agent: &str) -> Response<Manga> {
    let client = Arc::new(Mangadex::new(user_agent));
    let source_name = "MangaDex".to_string();

    let manga = match client.manga(id) {
        Ok(mg_cont) => mg_cont.data,
        Err(report) => return create_error_response(report, &source_name),
    };

    let authors = match extract_author(client.clone(), &manga).await {
        Ok(authors) => authors,
        Err(report) => return create_error_response(report, &source_name),
    };

    let title = extract_title("en", &manga).unwrap_or("Unknown Title".to_string());
    let identifier = manga.id;
    let attr = manga.attributes;
    let description = attr
        .description
        .as_ref()
        .and_then(|desc| desc.get("en"))
        .cloned()
        .unwrap_or(String::from("No description"));
    let original_language = attr.original_language;
    let status = attr.status.to_dto();
    // For now, it only supports english language
    let language = String::from("en");

    Response {
        status: Status::Ok,
        reason: "All good!".to_string(),
        source_name: source_name.clone(),
        content: Some(Manga {
            identifier,
            title,
            authors,
            original_language,
            language,
            description,
            status,
        }),
    }
}

fn fetch_chapter_list(
    id: &str,
    page: u32,
    filter: Filter,
    user_agent: &str,
) -> Response<ChapterList> {
    let client = Arc::new(Mangadex::new(user_agent));
    let client_name = String::from("MangaDex");
    let limit = 40;
    let offset = (page - 1) * 50;
    let query = ChapterQuery::new(limit, offset).set_order(filter.sort.into());

    let chlist_cont = match client.chapters(id, &query) {
        Err(report) => return create_error_response(report, &client_name),
        Ok(chlist_cont) => chlist_cont,
    };

    let total_page = (chlist_cont.total + limit - 1) / limit;
    let data = chlist_cont
        .data
        .into_iter()
        .map(|ch| {
            let identifier = ch.id;
            let title = ch.attributes.title.unwrap_or("No title".to_string());
            let number = ch.attributes.chapter.unwrap_or("No number".to_string());

            ChapterListEntry {
                identifier,
                title,
                number,
            }
        })
        .collect();

    Response {
        status: Status::Ok,
        reason: String::from("All good"),
        source_name: client_name,
        content: Some(ChapterList {
            page,
            total_page,
            data,
        }),
    }
}

fn fetch_chapter(id: &str, user_agent: &str) -> Response<Chapter> {
    let client = Arc::new(Mangadex::new(user_agent));
    let source_name = "MangaDex".to_string();

    let ch_container = match client.chapter(id) {
        Ok(ch_cont) => ch_cont,
        Err(report) => return create_error_response(report, &source_name),
    };

    let page_urls = match get_chapter_pages(id, user_agent) {
        Ok(pages) => pages,
        Err(report) => return create_error_response(report, &source_name),
    };

    let chapter = ch_container.data;
    let identifier = chapter.id;
    let attr = &chapter.attributes;

    // This code will find the chapter's origin manga
    let mut manga_identifier = String::from("");
    for rel in chapter.relationships.unwrap() {
        if let RelationshipType::Manga = rel.rel_type {
            manga_identifier = rel.id.clone();
        }
    }

    let title = attr.title.clone().unwrap_or(String::from("No Title"));
    let number = attr.chapter.clone().unwrap_or(String::from("No Number"));
    let language = attr.translated_language.clone();

    Response {
        status: Status::Ok,
        reason: "All good".to_string(),
        source_name,
        content: Some(Chapter {
            identifier,
            manga_identifier,
            title,
            number,
            language,
            page_urls,
        }),
    }
}

fn create_error_response<T>(report: Report, source_name: &str) -> Response<T> {
    let err_msg = report
        .downcast::<ureq::Error>()
        .ok()
        .and_then(|err| match err {
            ureq::Error::Status(code, response) => response
                .into_json::<schema::ErrorResponse>()
                .map(|resp| {
                    resp.errors
                        .first()
                        .map(|e| format!("{}: {}", code, e.title))
                })
                .unwrap_or(Some("Unknown error".to_string())),

            ureq::Error::Transport(transport) => transport.message().map(|m| m.to_string()),
        })
        .unwrap_or("Unknown error".to_string());

    Response {
        status: Status::Error,
        reason: err_msg,
        source_name: source_name.to_string(),
        content: None,
    }
}

fn get_chapter_pages(id: &str, user_agent: &str) -> Result<Vec<String>> {
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

/// This function will try to extract manga title for the preferred language. If the
/// preferred language doesn't exist, it will try to use whatever the first title is available.
/// If it doesn't exist too, it will return None.
fn extract_title(preferred_lang: &str, manga: &MDManga) -> Option<String> {
    let attr = &manga.attributes;
    attr.title.get(preferred_lang).cloned().or_else(|| {
        attr.title
            .keys()
            .next()
            .and_then(|k| attr.title.get(k).cloned())
    })
}

/// This function will extract the author and artist from the manga's relation list
async fn extract_author(client: Arc<Mangadex>, md_manga: &MDManga) -> Result<Vec<Author>> {
    let mut handles = Vec::new();

    let Some(relationships) = &md_manga.relationships else {
        return Err(eyre!("There's no relationships".to_string()));
    };

    for rel in relationships {
        // Checks if the relation is artist or author
        if let RelationshipType::Artist | RelationshipType::Author = rel.rel_type {
            let details = match rel.rel_type {
                RelationshipType::Artist => String::from("Artist"),
                RelationshipType::Author => String::from("Author"),
                _ => unreachable!(),
            };

            let cl = client.clone();
            let rl = rel.clone();
            let author: JoinHandle<Result<Author>> = tokio::spawn(async move {
                let name: String = cl
                    .author(&rl.id)?
                    .attributes
                    .name
                    .chars()
                    .filter(|c| c.is_ascii() && *c != '(' && *c != ')')
                    .collect::<String>();

                Ok(Author {
                    name: name.trim().to_string(),
                    details,
                })
            });
            handles.push(author);
        }
    }

    let mut authors: Vec<Author> = Vec::new();

    for handle in future::join_all(handles).await {
        authors.push(handle??);
    }

    Ok(authors)
}

fn write_to_stream(stream: &mut TcpStream, payload: &str) -> Result<()> {
    let size = payload.len() as u32;
    stream.write_all(&size.to_ne_bytes())?;
    stream.write_all(payload.as_bytes())?;
    stream.flush()?;

    Ok(())
}
