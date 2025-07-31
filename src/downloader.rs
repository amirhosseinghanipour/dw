use crate::config::DownloadConfig;
use crate::utils::BandwidthMonitor;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use reqwest::header;
use std::sync::Arc;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::sync::Semaphore;

pub struct DownloadOptimizer {
    clients: Vec<Client>,
    config: DownloadConfig,
    bandwidth_monitor: Arc<BandwidthMonitor>,
    semaphore: Arc<Semaphore>,
}

impl DownloadOptimizer {
    pub async fn new(
        config: DownloadConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut clients = Vec::new();

        let mut headers = header::HeaderMap::new();
        headers.insert(header::USER_AGENT, header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"));
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static(
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
            ),
        );
        headers.insert(
            header::ACCEPT_LANGUAGE,
            header::HeaderValue::from_static("en-US,en;q=0.5"),
        );

        for _ in 0..config.max_connections {
            let client = Client::builder()
                .default_headers(headers.clone())
                .http1_only()
                .tcp_keepalive(std::time::Duration::from_secs(15))
                .timeout(config.connection_timeout)
                .build()?;
            clients.push(client);
        }

        Ok(Self {
            clients,
            config: config.clone(),
            bandwidth_monitor: Arc::new(BandwidthMonitor::new()),
            semaphore: Arc::new(Semaphore::new(config.max_connections * 2)),
        })
    }

    pub async fn download(
        &self,
        url: &str,
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let head_response = self.clients[0].head(url).send().await?;
        let total_size = head_response.content_length().unwrap_or(0);
        let accepts_ranges = head_response
            .headers()
            .get("accept-ranges")
            .and_then(|v| v.to_str().ok())
            .map(|v| v == "bytes")
            .unwrap_or(false);

        if accepts_ranges
            && total_size > self.config.min_chunk_size
            && self.config.max_connections > 1
        {
            self.parallel_download(url, filename, total_size).await
        } else {
            self.single_download(url, filename).await
        }
    }

    async fn parallel_download(
        &self,
        url: &str,
        filename: &str,
        total_size: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let file = std::fs::File::create(filename)?;
        file.set_len(total_size)?;

        let chunk_size = total_size / self.config.max_connections as u64;
        let mut handles = Vec::new();

        let pb = create_progress_bar(total_size);

        for i in 0..self.config.max_connections {
            let start = i as u64 * chunk_size;
            let end = if i == self.config.max_connections - 1 {
                total_size - 1
            } else {
                (i + 1) as u64 * chunk_size - 1
            };

            let client = self.clients[i % self.clients.len()].clone();
            let url = url.to_string();
            let filename = filename.to_string();
            let semaphore = self.semaphore.clone();
            let monitor = self.bandwidth_monitor.clone();
            let pb = pb.clone();

            handles.push(tokio::spawn(async move {
                let _permit = semaphore
                    .acquire()
                    .await
                    .map_err(|_| "Semaphore error".to_string())?;
                download_chunk(&client, &url, &filename, start, end, monitor, pb).await
            }));
        }

        for handle in handles {
            handle.await??;
        }

        pb.finish_with_message("Download complete");
        Ok(())
    }

    async fn single_download(
        &self,
        url: &str,
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let response = self.clients[0].get(url).send().await?;
        if !response.status().is_success() {
            return Err(format!("HTTP Error: {}", response.status()).into());
        }

        let total_size = response.content_length().unwrap_or(0);
        let pb = create_progress_bar(total_size);

        let mut file = tokio::fs::File::create(filename).await?;
        let mut stream = response.bytes_stream();
        let monitor = self.bandwidth_monitor.clone();

        let mut downloaded: u64 = 0;
        let mut buffer = vec![0u8; self.config.buffer_size];

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;

            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);

            monitor.record_bytes(chunk.len() as u64);

            if self.config.adaptive_buffering && downloaded % (5 * 1024 * 1024) == 0 {
                let speed = monitor.get_current_speed();
                if speed > 50.0 && buffer.len() < 4 * 1024 * 1024 {
                    buffer.resize(buffer.len() * 2, 0);
                } else if speed < 5.0 && buffer.len() > 64 * 1024 {
                    buffer.resize(buffer.len() / 2, 0);
                }
            }
        }

        pb.finish_with_message("Download complete");
        Ok(())
    }
}

async fn download_chunk(
    client: &Client,
    url: &str,
    filename: &str,
    start: u64,
    end: u64,
    monitor: Arc<BandwidthMonitor>,
    pb: ProgressBar,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let range = format!("bytes={start}-{end}");
    let response = client
        .get(url)
        .header(reqwest::header::RANGE, range)
        .send()
        .await?;

    if response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
        return Err("Server doesn't support range requests".into());
    }

    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .open(filename)
        .await?;

    file.seek(std::io::SeekFrom::Start(start)).await?;

    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;

        let written = chunk.len() as u64;
        pb.inc(written);

        monitor.record_bytes(written);
    }

    Ok(())
}

fn create_progress_bar(total_size: u64) -> ProgressBar {
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} \
                 ({bytes_per_sec}, {eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
    );
    pb
}
