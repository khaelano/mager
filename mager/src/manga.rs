use std::env;
use std::fs::File;
use std::io::{self, stdin, stdout, Write};
use std::path::{Path, PathBuf};
use std::process;

use crate::utils::{connect_to_source, download_resource, read_from_stream, write_to_stream};
use dto::carriers::{Command, Request, Response, Status};
use dto::{Chapter, ChapterList, ChapterPages, Filter, Manga, MangaList, Order};

use crate::MangaOperation;

fn print_chapter_list(cl: &ChapterList) {
    println!(
        "Displaying Chapter List (page {} of {}):",
        cl.page, cl.total_page
    );

    let mut i = 1;
    println!("Index    Chapter                              Title");
    for ch in cl.data.iter() {
        println!("{:>5}    {:<33}    {}", i, ch.number, ch.title);
        i += 1;
    }
}

fn print_mangalist(ml: &MangaList) {
    println!(
        "Displaying Manga List (page {} of {}):",
        ml.page, ml.total_page
    );

    let mut i = 1;
    for m in ml.data.iter() {
        println!("{:>2}. {}", i, &m.title);
        i += 1;
    }
}

fn print_manga_info(manga: &Manga) {
    println!("Displaying details for \"{}\":", manga.title);

    let authors: Vec<String> = manga
        .authors
        .iter()
        .map(|a| format!("{} ({})", a.name, a.details))
        .collect();

    println!("Authors           : {}", authors.join(", "));

    let status = match manga.status {
        dto::PublicationStatus::Ongoing => "Ongoing",
        dto::PublicationStatus::Completed => "Completed",
        dto::PublicationStatus::Hiatus => "Hiatus",
        dto::PublicationStatus::Cancelled => "Cancelled",
        dto::PublicationStatus::Unknown => "Unknown",
    };
    println!("Status            : {}", status);
    println!("Original language : {}", manga.original_language);
    println!("Language          : {}", manga.language);
    println!();
    println!("--Description--");
    println!("{}", manga.description.trim());
}

fn fetch_chapter_list(
    identifier: &str,
    page: u32,
    filter: Filter,
) -> Result<ChapterList, io::Error> {
    let req = Request {
        command: Command::Chapters {
            identifier: identifier.to_string(),
            page,
            filter,
        },
        version: String::from("0.0.0"),
    };

    let payload = serde_json::to_string(&req)?;

    let mut connection = connect_to_source(7878)?;
    write_to_stream(&payload, &mut connection)?;

    let raw_response = read_from_stream(&mut connection)?;
    let response: Response<ChapterList> = serde_json::from_slice(&raw_response)?;

    match response.status {
        Status::Ok => Ok(response.content),
        Status::Error => Err(io::Error::new(
            io::ErrorKind::NotFound,
            "API response error",
        )),
    }
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

fn browse_chapters(port: u16, manga: &Manga, mut page: u32) {
    loop {
        let filter = Filter {
            language: String::from("en"),
            sort: Order::Descending,
        };

        let request = Request {
            command: Command::Chapters {
                identifier: manga.identifier.clone(),
                page,
                filter,
            },
            version: String::from("0.0.0"),
        };

        let mut connection = connect_to_source(port).unwrap();
        write_to_stream(&serde_json::to_string(&request).unwrap(), &mut connection).unwrap();

        let response: Response<ChapterList> =
            serde_json::from_slice(&read_from_stream(&mut connection).unwrap()).unwrap();

        let ch_list = &response.content;

        print_chapter_list(ch_list);
        println!("Choose chapter index to download or perform an action");
        print!(
            "Enter a command: [1-{}, n: next, p: prev, b: back]: ",
            ch_list.data.len()
        );
        stdout().flush().unwrap();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        let input = input.trim();
        match input.parse::<u32>() {
            Ok(i) => {
                let chapter = ch_list.data.get((i - 1) as usize).unwrap();
                download_chapter(port, &manga.title, chapter);
                break;
            }
            Err(_) => match input {
                "n" => page += 1,
                "p" => page -= 1,
                "b" => break,
                _ => panic!("Invalid operation"),
            },
        }
    }
}

fn browse_manga(port: u16, keyword: &str, mut page: u32) {
    loop {
        let filter = Filter {
            language: String::from("en"),
            sort: Order::Descending,
        };

        let request = Request {
            command: Command::Search {
                keyword: keyword.to_string(),
                page,
                filter,
            },
            version: String::from("0.0.0"),
        };

        let mut connection = connect_to_source(port).unwrap();
        write_to_stream(&serde_json::to_string(&request).unwrap(), &mut connection).unwrap();

        let response: Response<MangaList> =
            serde_json::from_slice(read_from_stream(&mut connection).unwrap().as_ref()).unwrap();

        let mn_list = response.content;

        print_mangalist(&mn_list); // Display search result
        println!("Choose manga index to see or perform an action");
        print!(
            "Enter a command [1-{}, n: next, p: prev, a: abort]: ",
            mn_list.data.len()
        );
        stdout().flush().unwrap();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        let input = input.trim();
        match input.parse::<u32>() {
            Ok(i) => {
                let manga = mn_list.data.get((i - 1) as usize).unwrap();
                print_manga_info(manga);

                browse_chapters(port, manga, 1);
                break;
            }
            Err(_) => match input {
                "n" => page += 1,
                "p" => page -= 1,
                "a" => break,
                _ => break,
            },
        }
    }
}

pub(crate) fn manga_menu_handler(source: &str, operation: &MangaOperation) {
    let home = env::var("HOME").unwrap();
    let port = 7878;

    let path = if cfg!(debug_assertions) {
        format!("{home}/Projects/mager/target/debug/{source}")
    } else {
        format!("{home}/.local/mager/sources/{source}")
    };

    let mut source_proccess = process::Command::new(path)
        .arg(&port.to_string())
        .spawn()
        .unwrap();

    // This code will attempt to connect to the source 10 times
    let mut connection = connect_to_source(port).unwrap();

    // Attempt to ping the source to ensure the source is connected
    let request = Request {
        command: Command::Ping,
        version: String::from("0.0.0"),
    };

    let request = serde_json::to_string(&request).unwrap();
    write_to_stream(request.trim(), &mut connection).unwrap();

    let raw_response = read_from_stream(&mut connection).unwrap();
    let response: Response<()> = serde_json::from_slice(&raw_response).unwrap();

    if let Status::Error = response.status {
        panic!("Error connecting to source");
    }

    match operation {
        MangaOperation::Search { keyword, page } => {
            browse_manga(port, keyword, *page);
        }
        MangaOperation::Download => todo!(),
    }
    source_proccess.kill().unwrap();
}
