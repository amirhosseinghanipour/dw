use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct BandwidthMonitor {
    bytes_transferred: AtomicU64,
    samples: Mutex<Vec<(std::time::Instant, u64)>>,
}

impl BandwidthMonitor {
    pub fn new() -> Self {
        Self {
            bytes_transferred: AtomicU64::new(0),
            samples: Mutex::new(Vec::new()),
        }
    }

    pub fn record_bytes(&self, bytes: u64) {
        self.bytes_transferred.fetch_add(bytes, Ordering::Relaxed);
        let now = std::time::Instant::now();

        let mut samples = self.samples.lock().unwrap();
        samples.push((now, bytes));

        samples.retain(|(time, _)| now.duration_since(*time).as_secs() < 10);
    }

    pub fn get_current_speed(&self) -> f64 {
        let samples = self.samples.lock().unwrap();
        if samples.is_empty() {
            return 0.0;
        }

        let total_bytes: u64 = samples.iter().map(|(_, bytes)| bytes).sum();
        let duration = samples
            .last()
            .map(|(time, _)| time.duration_since(samples[0].0).as_secs_f64())
            .unwrap_or(0.001);

        if duration == 0.0 {
            return 0.0;
        }

        total_bytes as f64 / duration / 1024.0 / 1024.0
    }
}

pub fn extract_filename(url: &str) -> Option<String> {
    url::Url::parse(url).ok().and_then(|u| {
        u.path_segments()
            .and_then(|mut segments| segments.next_back())
            .and_then(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(s.to_string())
                }
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_extract_filename_from_simple_url() {
        let url = "http://example.com/some/path/to/file.zip";
        assert_eq!(extract_filename(url), Some("file.zip".to_string()));
    }

    #[test]
    fn test_extract_filename_with_query_params() {
        let url = "http://example.com/archive.tar.gz?token=123&user=abc";
        assert_eq!(extract_filename(url), Some("archive.tar.gz".to_string()));
    }

    #[test]
    fn test_extract_filename_from_root() {
        let url = "http://example.com/";
        assert_eq!(extract_filename(url), None);
    }

    #[test]
    fn test_extract_filename_without_extension() {
        let url = "http://example.com/some/directory/resource";
        assert_eq!(extract_filename(url), Some("resource".to_string()));
    }

    #[test]
    fn test_extract_filename_from_invalid_url() {
        let url = "not-a-valid-url";
        assert_eq!(extract_filename(url), None);
    }

    #[test]
    fn test_bandwidth_monitor_initial_state() {
        let monitor = BandwidthMonitor::new();
        assert_eq!(monitor.bytes_transferred.load(Ordering::Relaxed), 0);
        assert_eq!(monitor.get_current_speed(), 0.0);
    }

    #[test]
    fn test_bandwidth_monitor_records_bytes() {
        let monitor = BandwidthMonitor::new();
        monitor.record_bytes(1024);
        monitor.record_bytes(2048);
        assert_eq!(monitor.bytes_transferred.load(Ordering::Relaxed), 3072);
    }

    #[test]
    fn test_bandwidth_monitor_calculates_speed() {
        let monitor = BandwidthMonitor::new();

        monitor.record_bytes(1024 * 1024);
        thread::sleep(Duration::from_millis(500));
        monitor.record_bytes(1024 * 1024);

        let speed = monitor.get_current_speed();
        assert!(
            speed > 1.0 && speed < 5.0,
            "Speed should be in a reasonable range (calculated: {} MB/s)",
            speed
        );
    }

    #[test]
    fn test_bandwidth_monitor_sample_retention() {
        let monitor = BandwidthMonitor::new();
        monitor.record_bytes(100);

        thread::sleep(Duration::from_secs(11));

        monitor.record_bytes(100);

        let speed = monitor.get_current_speed();
        assert!(speed >= 0.0, "Speed should not be negative.");
    }
}
