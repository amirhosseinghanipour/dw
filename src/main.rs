mod config;
mod downloader;
mod utils;

use crate::config::DownloadConfig;
use clap::Parser;
use downloader::DownloadOptimizer;
use std::process;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "A blazingly fast download accelerator, written in Rust."
)]
struct Args {
    url: String,
    #[arg(short, long)]
    output: Option<String>,
    #[arg(short = 'c', long, default_value = "8")]
    connections: usize,
    #[arg(short = 'b', long, default_value = "1024")]
    buffer_size: usize,
    #[arg(long)]
    adaptive: bool,
    #[arg(long, default_value = "1")]
    min_chunk: usize,
}

const GITHUB_ISSUES_URL: &str = "https://github.com/amirhosseinghanipour/dw/issues";

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = DownloadConfig {
        max_connections: args.connections,
        buffer_size: args.buffer_size * 1024,
        adaptive_buffering: args.adaptive,
        min_chunk_size: args.min_chunk as u64 * 1024 * 1024,
        connection_timeout: std::time::Duration::from_secs(30),
    };

    let optimizer = match DownloadOptimizer::new(config).await {
        Ok(optimizer) => optimizer,
        Err(e) => {
            eprintln!("Failed to initialize downloader: {e}");
            eprintln!("\nIf this issue persists, please open an issue on GitHub:");
            eprintln!("{GITHUB_ISSUES_URL}");
            process::exit(1);
        }
    };

    let filename = args.output.unwrap_or_else(|| {
        utils::extract_filename(&args.url).unwrap_or_else(|| "downloaded_file".to_string())
    });

    match optimizer.download(&args.url, &filename).await {
        Ok(_) => println!("Download saved as: {filename}"),
        Err(e) => {
            eprintln!("Download failed: {e}");
            eprintln!("\nIf you believe this is a bug, please open an issue on GitHub:");
            eprintln!("{GITHUB_ISSUES_URL}");
            process::exit(1);
        }
    }
}
