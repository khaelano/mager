use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::thread;
use std::time::Duration;

use color_eyre::eyre::{eyre, Result};
use futures::StreamExt;
use reqwest::ClientBuilder;
use tokio::fs::{create_dir_all, File};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::{self, UnboundedReceiver};

use dto::carriers;
use dto::carriers::{Request, Response, Status};
use tracing::{event, info, warn, Level};

pub(crate) async fn download_resource(
    url: String,
    file_name: impl AsRef<Path>,
) -> Result<UnboundedReceiver<f32>> {
    info!(
        "Downloading file to {}",
        file_name.as_ref().to_str().unwrap()
    );
    let (progress_tx, progress_rx) = mpsc::unbounded_channel::<f32>();
    let client = ClientBuilder::new()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0")
        .build()?;

    let response = client.get(&url).send().await?;

    let mut downloaded = 0 as f32;
    let total_size = response
        .content_length()
        .ok_or(eyre!("Failed to get length"))? as f32;

    if let Some(parent) = file_name.as_ref().parent() {
        create_dir_all(parent).await?
    }
    let mut file = File::create(&file_name).await?;

    let mut stream = response.bytes_stream();
    let mut attempt_counter = 1;
    while let Some(chunk) = stream.next().await {
        let Ok(chunk) = chunk else {
            // this code will try to reestablish the connection if interrupted
            warn!("Network interrrupted, retrying");
            attempt_counter += 1;

            if attempt_counter >= 20 {
                event!(Level::ERROR, "Maximum retries reached");
                return Err(eyre!("Maximum retries reached"));
            }

            let range_header = format!("bytes={}", downloaded as u64);
            let response = client
                .get(&url)
                .header("Range", range_header)
                .send()
                .await?;
            stream = response.bytes_stream();
            continue;
        };
        attempt_counter = 1;
        downloaded += chunk.len() as f32;

        file.write(&chunk).await?;

        let progress = downloaded / total_size;
        event!(Level::DEBUG, "downloading progress: {progress}");
        progress_tx.send(progress)?;
    }
    info!("Download complete");

    Ok(progress_rx)
}

pub(crate) fn write_to_stream(request: &str, connection: &mut TcpStream) -> Result<()> {
    let size = request.len() as u32;
    connection.write_all(&size.to_ne_bytes())?;
    connection.write_all(request.as_bytes())?;
    connection.flush()?;

    Ok(())
}

pub(crate) fn read_from_stream(connection: &mut TcpStream) -> Result<Vec<u8>> {
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
        event!(Level::DEBUG, "Connecting to port {port}");
        counter += 1;
        let stream = TcpStream::connect(format!("127.0.0.1:{port}"));
        match stream {
            Ok(s) => {
                event!(Level::DEBUG, "Connection to port {port} established!");
                return Ok(s);
            }
            Err(e) => {
                event!(Level::DEBUG, "Failed to connect to , trying again...");
                error = e
            }
        }

        if counter > 20 {
            event!(
                Level::ERROR,
                "Failed to connect to {port}. Reached attempt limit"
            );
            return Err(error);
        }
        thread::sleep(Duration::from_millis(50));
    }
}

pub(crate) fn ping(port: u16) -> Result<()> {
    info!("Trying to ping port {port}");
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
        event!(Level::ERROR, "Response received, ping failed");
        return Err(eyre!("Not Ok :("));
    }
    info!("Response received, ping successful");

    Ok(())
}
