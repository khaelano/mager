use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Child, Command};
use std::time::Duration;
use std::{env, thread};

use dto::carriers::{self, Request, Response, Status};
use tokio::fs::{create_dir_all, File};

use color_eyre::eyre::{eyre, Result};
use reqwest::{self, ClientBuilder};
use tokio::io::AsyncWriteExt;

use crate::source::Source;

pub(crate) async fn download_resource(
    url: String,
    file_name: impl AsRef<Path>,
) -> Result<(), io::Error> {
    let client = ClientBuilder::new()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0")
        .build()
        .unwrap();

    let response = client.get(url).send().await.unwrap();
    let bytes = response.bytes().await.unwrap();

    if let Some(parent) = file_name.as_ref().parent() {
        create_dir_all(parent).await.unwrap()
    }

    let mut file = File::create(&file_name).await.unwrap();
    file.write_all(&bytes).await.unwrap();

    Ok(())
}

pub(crate) fn write_to_stream(request: &str, connection: &mut TcpStream) -> Result<(), io::Error> {
    let size = request.len() as u32;
    connection.write_all(&size.to_ne_bytes())?;
    connection.flush()?;
    connection.write_all(request.as_bytes())?;
    connection.flush()?;

    Ok(())
}

pub(crate) fn read_from_stream(connection: &mut TcpStream) -> Result<Vec<u8>, io::Error> {
    let mut len = [0; 4];
    connection.read_exact(&mut len)?;

    let mut buffer = vec![0; u32::from_ne_bytes(len) as usize];
    connection.read_exact(&mut buffer)?;

    Ok(buffer)
}

pub(crate) fn connect_to_source(port: u16) -> Result<TcpStream, io::Error> {
    let mut counter = 0;
    let mut error: io::Error;

    loop {
        let stream = TcpStream::connect(format!("127.0.0.1:{port}"));
        match stream {
            Ok(s) => return Ok(s),
            Err(e) => error = e,
        }

        if counter > 20 {
            return Err(error);
        }
        counter += 1;
        thread::sleep(Duration::from_millis(50));
    }
}

pub(crate) fn ping(port: u16) -> Result<()> {
    let mut connection = connect_to_source(port)?;

    let request = Request {
        command: carriers::Command::Ping,
        version: "0.0.0".to_string(),
    };

    let request = serde_json::to_string(&request)?;

    write_to_stream(&request, &mut connection)?;

    let raw_response = read_from_stream(&mut connection)?;
    let response: Response<()> = serde_json::from_slice(&raw_response)?;

    if let Status::Error = response.status {
        return Err(eyre!("Not Ok :("));
    }

    Ok(())
}
