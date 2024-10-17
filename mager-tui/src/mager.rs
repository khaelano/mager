use std::env;

use color_eyre::eyre::eyre;
use color_eyre::Result;
use dto::carriers::{Command, Response, Status};
use dto::{carriers::Request, MangaList};

use tokio::fs;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tracing::instrument;

use crate::actions::Action;
use crate::source::Source;
use crate::utils::*;

use dto::*;

/// Function to list sources that are available in the local machine.
/// By default, the path is located in $HOME/.local/mager/sources/
pub async fn list_local_sources() -> Result<Vec<Source>> {
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

/// Function to list sources that can be downloaded from the online repository.
/// WARNING: This function is not finished! (i still don't know how to properly implements it)
pub async fn list_repo_sources() -> Result<Vec<Source>> {
    todo!();
}

/// Fetch all available sources, either local sources, or the one in the online repository.
/// WARNING: This function is not finished! Use with caution
pub async fn fetch_sources(action_tx: UnboundedSender<Action>) -> Result<()> {
    let local_sources = list_local_sources().await?;

    action_tx.send(Action::DisplaySourceList(local_sources))?;
    Ok(())
}

/// Connects to port and verify source. If the current listener responds back, verification
/// is successful.
pub fn ping_source(port: u16) -> Result<()> {
    let request = Request {
        command: Command::Ping,
        version: String::from("0.0.0"),
    };
    let request_string = serde_json::to_string(&request)?;

    let mut connection = connect_to_source(port)?;
    write_to_stream(&request_string, &mut connection)?;
    let raw_response = read_from_stream(&mut connection)?;
    let response: Response<()> = serde_json::from_slice(&raw_response)?;

    match response.status {
        Status::Ok => Ok(()),
        Status::Error => Err(eyre!("Error verifying source.")),
    }
}

/// Sends manga search request to active source and return its response. Please take note
/// that all error from the server are not handled by this function.
pub async fn search_manga(
    port: u16,
    search_keyword: &str,
    page: u32,
    filter: &Filter,
) -> Result<Response<MangaList>> {
    let request = Request {
        command: Command::Search {
            keyword: search_keyword.to_string(),
            page,
            filter: filter.clone(),
        },
        version: String::from("0.0.0"),
    };
    let req_string = serde_json::to_string(&request)?;

    // Connect to source and validates it
    let Ok(()) = ping_source(port) else {
        return Err(eyre!("Error verifying source"));
    };

    let mut connection = connect_to_source(port)?;
    write_to_stream(&req_string, &mut connection)?;
    let raw_response = read_from_stream(&mut connection)?;
    let response: Response<MangaList> = serde_json::from_slice(&raw_response)?;

    Ok(response)
}

/// Sends chapter list request for a specified manga to active ource and return its response.
/// Please take note that all error from the server are not handled by this function.
pub async fn fetch_chapters(
    port: u16,
    manga_identifier: &str,
    page: u32,
    filter: &Filter,
) -> Result<Response<ChapterList>> {
    let request = Request {
        command: Command::FetchChapterList {
            identifier: manga_identifier.to_string(),
            page,
            filter: filter.clone(),
        },
        version: String::from("0.0.0"),
    };
    let req_string = serde_json::to_string(&request)?;

    // Connect to source and validates it
    let Ok(()) = ping_source(port) else {
        return Err(eyre!("Error verifying source"));
    };

    let mut connection = connect_to_source(port)?;
    write_to_stream(&req_string, &mut connection)?;
    let raw_response = read_from_stream(&mut connection)?;
    let response: Response<ChapterList> = serde_json::from_slice(&raw_response)?;

    Ok(response)
}

/// Sends a manga details request for a specified manga to active source and return its
/// response. Please take note that all error from the server are not handled by this function.
pub async fn fetch_manga(port: u16, manga_identifier: &str) -> Result<Response<Manga>> {
    let request = Request {
        command: Command::FetchManga {
            manga_identifier: manga_identifier.to_string(),
        },
        version: String::from("0.0.0"),
    };

    let req_string = serde_json::to_string(&request)?;

    // Connect to source and validates it
    if ping_source(port).is_err() {
        return Err(eyre!("Error verifying source"));
    }

    let mut connection = connect_to_source(port)?;
    write_to_stream(&req_string, &mut connection)?;
    let raw_response = read_from_stream(&mut connection)?;
    let response: Response<Manga> = serde_json::from_slice(&raw_response)?;

    Ok(response)
}

/// Sends a chapter details request for a specified manga to active source and return its
/// response. Please take note that all error from the server are not handled by this function.
pub async fn fetch_chapter(port: u16, chapter_identifier: &str) -> Result<Response<Chapter>> {
    let request = Request {
        command: Command::FetchChapter {
            chapter_identifier: chapter_identifier.to_string(),
        },
        version: String::from("0.0.0"),
    };

    let req_string = serde_json::to_string(&request)?;

    // Connect to source and validates it
    if ping_source(port).is_err() {
        return Err(eyre!("Error verifying source"));
    }

    let mut connection = connect_to_source(port)?;
    write_to_stream(&req_string, &mut connection)?;
    let raw_response = read_from_stream(&mut connection)?;
    let response: Response<Chapter> = serde_json::from_slice(&raw_response)?;

    Ok(response)
}

#[instrument]
pub async fn download_chapter(port: u16, chapter_id: &str) -> Result<Vec<UnboundedReceiver<f32>>> {
    let home = env::var("HOME")?;

    let ch_response = fetch_chapter(port, &chapter_id).await?;
    let chapter = ch_response.content.unwrap();

    let mng_response = fetch_manga(port, &chapter.manga_identifier).await?;
    let manga = mng_response.content.unwrap();

    let base_folder = format!(
        "{home}/Downloads/mager/{}/#{} - {}/",
        manga.title, chapter.number, chapter.title
    );

    let mut progress_rxs: Vec<UnboundedReceiver<f32>> = Vec::new();
    for (i, url) in chapter.page_urls.iter().enumerate() {
        progress_rxs
            .push(download_resource(url.to_string(), format!("{}/{}", base_folder, i + 1)).await?);
    }
    Ok(progress_rxs)
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
