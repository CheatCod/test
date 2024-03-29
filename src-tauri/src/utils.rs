use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use reqwest::Client;
use tokio::io::AsyncWriteExt;

pub async fn download_file(
    url: &str,
    path: &Path,
    name_override: Option<&str>,
    overwrite_old: bool,
) -> PathBuf {
    let client = Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .expect("Failed to download file");
    response
        .error_for_status_ref()
        .expect("Failed to download file");
    tokio::fs::create_dir_all(path)
        .await
        .expect("Failed to create directory");

    let file_name;
    if let Some(name) = name_override {
        file_name = name.to_string();
    } else {
        file_name = response
            .headers()
            .get("Content-Disposition")
            .map_or_else(
                || "unknown".to_string(),
                |h| {
                    h.to_str()
                        .map_or_else(|_| "unknown".to_string(), |s| s.to_string())
                },
            )
            // parse filename's value from the header, remove the ""
            .split(';')
            .nth(1)
            .unwrap_or("unknown")
            .split('=')
            .nth(1)
            .unwrap_or("unknown")
            .replace('\"', "");
    }
    if !overwrite_old && path.join(&file_name).exists() {
        return path.join(&file_name);
    }
    tokio::fs::remove_file(path.join(&file_name)).await.ok();

    let mut downloaded_file = tokio::fs::File::create(path.join(&file_name))
        .await
        .expect("Failed to create file");
    let mut stream = response.bytes_stream();
    while let Some(item) = stream.next().await {
        let chunk = item.expect("Error while downloading file");
        downloaded_file
            .write_all(&chunk)
            .await
            .expect("Failed to write to file");
    }
    path.join(&file_name)
}
