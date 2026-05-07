use std::collections::VecDeque;
use std::process::Command;
use std::time::{Duration, Instant};

pub struct LogLine {
    pub timestamp: String,
    pub message: String,
    pub age: f32,
}

pub struct LogBuffer {
    lines: VecDeque<LogLine>,
    max_cap: usize,
    last_fetch: Instant,
    fetch_interval_secs: u64,
}

impl LogBuffer {
    pub fn new(max_cap: usize) -> Self {
        Self {
            lines: VecDeque::with_capacity(max_cap),
            max_cap,
            last_fetch: Instant::now().checked_sub(Duration::from_secs(10)).unwrap_or_else(Instant::now),
            fetch_interval_secs: 5,
        }
    }

    pub fn poll(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_fetch).as_secs() < self.fetch_interval_secs {
            return;
        }
        self.last_fetch = now;

        let output = Self::fetch_journal().or_else(Self::fetch_syslog);

        if let Some(lines) = output {
            let total = lines.len();
            for (i, line) in lines.into_iter().enumerate() {
                let age = if total > 0 { i as f32 / total as f32 } else { 0.0 };

                if self.lines.len() >= self.max_cap {
                    self.lines.pop_back();
                }

                self.lines.push_front(LogLine {
                    timestamp: line.0,
                    message: line.1,
                    age,
                });
            }
        }
    }

    pub fn get_recent(&self, count: usize) -> Vec<&LogLine> {
        self.lines.iter().take(count).collect()
    }

    fn fetch_journal() -> Option<Vec<(String, String)>> {
        let output = Command::new("journalctl")
            .args(["-n", "50", "--no-pager", "-o", "short-iso", "--since=-5minutes"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        Some(Self::parse_lines(String::from_utf8_lossy(&output.stdout)))
    }

    fn fetch_syslog() -> Option<Vec<(String, String)>> {
        let output = Command::new("tail")
            .args(["-n", "50", "/var/log/syslog"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        Some(Self::parse_lines(String::from_utf8_lossy(&output.stdout)))
    }

    fn parse_lines(content: std::borrow::Cow<str>) -> Vec<(String, String)> {
        content
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                if parts.len() >= 2 {
                    Some((parts[0].to_string(), parts[1].to_string()))
                } else {
                    None
                }
            })
            .collect()
    }
}