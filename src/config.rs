use std::time::Duration;

#[derive(Clone)]
pub struct DownloadConfig {
    pub max_connections: usize,
    pub buffer_size: usize,
    pub adaptive_buffering: bool,
    pub min_chunk_size: u64,
    pub connection_timeout: Duration,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            max_connections: 8,
            buffer_size: 1024 * 1024,
            adaptive_buffering: true,
            min_chunk_size: 1024 * 1024,
            connection_timeout: Duration::from_secs(30),
        }
    }
}
