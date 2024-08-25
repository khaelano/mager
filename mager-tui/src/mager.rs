use std::path::PathBuf;
use std::{env, process::Child};

use color_eyre::eyre::eyre;
use color_eyre::Result;
use crossterm::style::Print;
use dto::carriers::{Command, Response};
use dto::{carriers::Request, Filter, MangaList};
use dto::{Chapter, ChapterList, ChapterPages};
use reqwest::{self, ClientBuilder};
use serde_json::Value;
use tokio::fs;
use tokio::sync::mpsc::UnboundedSender;

use crate::actions::{Action, AsyncItem};
use crate::source::Source;
use crate::utils::{connect_to_source, download_resource, read_from_stream, write_to_stream};

pub(crate) async fn list_local_sources() -> Result<Vec<Source>> {
    let mut sources: Vec<Source> = Vec::new();

    let home = env::var("HOME")?;
    let dir = format!("{home}/.local/mager/sources/");

    fs::create_dir_all(&dir).await?;
    let mut registries = fs::read_dir(&dir).await?;

    while let Some(reg) = registries.next_entry().await? {
        if !reg.file_type().await?.is_file() {
            continue;
        }

        sources.push(Source {
            name: reg.file_name().clone().to_string_lossy().to_string(),
            url: None,
            is_local: true,
            process: None,
        });
    }

    Ok(sources)
}

pub(crate) async fn list_repo_sources() -> Result<Vec<Source>> {
    let mut sources: Vec<Source> = Vec::new();
    let client = ClientBuilder::new().user_agent("mager").build()?;

    let response = client
        .get("https://api.github.com/repos/khaelano/mager/contents/release/sources")
        .send()
        .await?
        .bytes()
        .await?;

    let parsed_response: Value = serde_json::from_slice(&response)?;

    for source in parsed_response.as_array().unwrap() {
        sources.push(Source {
            name: source.get("name").unwrap().as_str().unwrap().to_string(),
            url: Some(
                source
                    .get("download_url")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string(),
            ),
            is_local: false,
            process: None,
        });
    }

    Ok(sources)
}

pub(crate) async fn send_request(
    action_tx: UnboundedSender<Action>,
    request: Request,
) -> Result<()> {
    let port = 7878;
    let request_string = serde_json::to_string(&request)?;

    let mut connection = connect_to_source(port)?;
    write_to_stream(&request_string, &mut connection)?;
    let response_bytes = read_from_stream(&mut connection)?;

    match request.command {
        Command::Ping => {}
        Command::Search {
            keyword: _,
            page: _,
            filter: _,
        } => {
            let response: Response<MangaList> = serde_json::from_slice(&response_bytes)?;
            action_tx.send(Action::Process(AsyncItem::Mangas(response.content)))?;
        }
        Command::Chapters {
            identifier: _,
            page: _,
            filter: _,
        } => {
            let response: Response<ChapterList> = serde_json::from_slice(&response_bytes)?;
            action_tx.send(Action::Process(AsyncItem::Chapters(response.content)))?;
        }
        Command::Pages { identifier: _ } => {
            let response: Response<ChapterPages> = serde_json::from_slice(&response_bytes)?;
            action_tx.send(Action::Process(AsyncItem::Pages(response.content)))?;
        }
    }
    Ok(())
}

pub(crate) async fn fetch_sources(action_tx: UnboundedSender<Action>) -> Result<()> {
    let local_sources = list_local_sources().await?;

    action_tx.send(Action::Process(AsyncItem::Sources(local_sources)))?;
    Ok(())
}

fn download_chapter(port: u16, manga_title: &str, chapter: &Chapter) {
    let home = env::var("HOME").unwrap();
    let request = Request {
        command: Command::Pages {
            identifier: chapter.identifier.clone(),
        },
        version: String::from("0.0.0"),
    };

    let mut connection = connect_to_source(port).unwrap();
    write_to_stream(&serde_json::to_string(&request).unwrap(), &mut connection).unwrap();

    let response: Response<ChapterPages> =
        serde_json::from_slice(&read_from_stream(&mut connection).unwrap()).unwrap();

    let base_folder = format!("{home}/Downloads/mager/{}", response.source_name);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let mut handles = Vec::new();
    let mut counter = 1;
    for url in response.content {
        let mut path = PathBuf::from(base_folder.clone());
        path.push(format!(
            "{manga_title}/{} - {}/{counter}.png",
            chapter.number, chapter.title
        ));

        let handle = rt.spawn(download_resource(url.clone(), path.clone()));

        handles.push(handle);
        counter += 1;
    }

    for h in handles {
        rt.block_on(h).unwrap().unwrap();
    }

    println!("Download successful")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn local_sources_test() {
        let rt = tokio::runtime::Runtime::new().unwrap();

        let _result = rt.block_on(list_local_sources()).unwrap();
    }

    #[test]
    fn repo_sources_test() {
        let rt = tokio::runtime::Runtime::new().unwrap();

        let _result = rt.block_on(list_repo_sources()).unwrap();
    }
}
