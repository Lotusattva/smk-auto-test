use anyhow::{Context, Error, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::PathBuf,
};
use tokio::sync::mpsc;

mod cli;
mod excel;
mod parse_go_map;
mod util;

use cli::Cli;
use excel::read_excel;
use util::to_base64;

#[derive(Serialize)]
struct Payload {
    request_name: Option<String>,
    images: Vec<String>,
    params: Option<Value>,
}

async fn send_request(
    client: &Client,
    address: &str,
    url: &str,
    payload: &Payload,
) -> Result<reqwest::Response> {
    let full_url = format!("{}{}", address, url);

    let response = client.post(&full_url).json(payload).send().await?;

    Ok(response)
}

async fn process_response(
    response: Result<reqwest::Response>,
    expected: Option<String>,
    index: usize,
) -> Result<Option<String>> {
    match response {
        Ok(res) => {
            let response_body = res
                .json::<Value>()
                .await
                .context("Failed to parse response body")?;

            let response_text = response_body.as_str().map_or_else(
                || response_body.to_string().replace('"', ""),
                |s| s.to_string().replace('"', ""),
            );

            if expected.is_none_or(|expected| !response_text.contains(&expected)) {
                Ok(Some(format!(
                    "Row {}: Response: {}",
                    index + 2,
                    response_text
                )))
            } else {
                Ok(None)
            }
        }

        Err(e) => Ok(Some(format!("Row {}: Request failed: {}", index + 2, e))),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let img_folder = cli.folder.unwrap_or(PathBuf::from("./images"));

    let (tx, mut rx) = mpsc::channel::<String>(32);
    let output_path = cli.output.unwrap_or(PathBuf::from("output.txt"));
    let output_path_clone = output_path.clone();
    let writer_handle = tokio::spawn(async move {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(output_path)
            .context("Failed to open output file")?;

        let mut writer = BufWriter::new(file);

        while let Some(msg) = rx.recv().await {
            writeln!(writer, "{}", msg).context("Failed to write to output")?;
        }

        writer.flush().context("Failed to flush output")?;

        Ok::<(), Error>(())
    });

    let rows = read_excel(&cli.excel).context("Failed to read Excel file")?;

    println!("Total requests to process: {}", rows.len());

    let client = Client::new();

    let mut handles = Vec::new();

    let pb = ProgressBar::new(rows.len() as u64);
    pb.set_message("Processing Requests");
    pb.set_style(ProgressStyle::with_template(
        "[{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} ({percent}%) - ETA: {eta_precise}",
    )?
    .progress_chars("##-"));

    for (index, row) in rows.into_iter().enumerate() {
        let img_folder = img_folder.clone();
        let address = cli.address.clone();
        let client = client.clone();
        let tx = tx.clone();
        let request_pb = pb.clone();

        handles.push(tokio::spawn(async move {
            let img_path = img_folder.join(&row.image_name);

            let img_base64 =
                to_base64(&img_path).context(format!("Failed to process image: {:?}", img_path))?;

            let payload = Payload {
                request_name: row.param,
                images: vec![img_base64],
                params: row.args,
            };

            let response = send_request(&client, &address, &row.url, &payload).await;

            if let Some(mismatch) = process_response(response, row.response, index).await? {
                tx.send(mismatch)
                    .await
                    .context("Failed to send mismatch message")?;
            }

            request_pb.inc(1);

            Ok::<(), Error>(())
        }));
    }

    drop(tx);

    for handle in handles {
        handle.await??;
    }

    pb.finish();

    let _ = writer_handle.await?;

    println!(
        "Mismatched responses have been written to {}.",
        output_path_clone.display()
    );

    Ok(())
}
