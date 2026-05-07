use std::collections::VecDeque;
use std::fs;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::PathBuf;

pub struct LogLine {
    pub timestamp: String,
    pub message: String,
    pub age: f32,
}

pub struct LogBuffer {
    lines: VecDeque<LogLine>,
    max_cap: usize,
    file_path: PathBuf,
    last_pos: u64,
    last_len: u64,
}

impl LogBuffer {
    pub fn new(max_cap: usize) -> Self {
        let path = Self::find_syslog_path().unwrap_or_else(|| PathBuf::from("/var/log/syslog"));
        Self {
            lines: VecDeque::with_capacity(max_cap),
            max_cap,
            file_path: path,
            last_pos: 0,
            last_len: 0,
        }
    }

    fn find_syslog_path() -> Option<PathBuf> {
        let candidates = [
            "/var/log/syslog",
            "/var/log/messages",
            "/var/log/user.log",
            "/run/log/syslog",
        ];
        for path in &candidates {
            if fs::metadata(path).is_ok() {
                return Some(PathBuf::from(*path));
            }
        }
        None
    }

    pub fn poll(&mut self) {
        let result = self.read_new_lines();
        if let Some(new_lines) = result {
            for line in new_lines {
                if self.lines.len() >= self.max_cap {
                    self.lines.pop_back();
                }
                self.lines.push_front(line);
            }
        }
    }

fn read_new_lines(&mut self) -> Option<Vec<LogLine>> {
        let metadata = match fs::metadata(&self.file_path) {
            Ok(m) => m,
            Err(_) => return None,
        };

        let file_len = metadata.len();

        if file_len < self.last_len {
            self.last_pos = 0;
            self.last_len = file_len;
        }

        if file_len == self.last_pos {
            return None;
        }

        let file = match fs::OpenOptions::new().read(true).open(&self.file_path) {
            Ok(f) => f,
            Err(_) => return None,
        };

        let mut reader = BufReader::new(file);
        if self.last_pos > 0 {
            if reader.seek(SeekFrom::Start(self.last_pos)).is_err() {
                return None;
            }
        }

        let mut buf = String::new();
        let bytes_read = reader.read_to_string(&mut buf).ok()?;

        let new_pos = self.last_pos + bytes_read as u64;
        self.last_pos = new_pos;
        self.last_len = file_len;

        if buf.is_empty() {
            return None;
        }

        let lines: Vec<LogLine> = buf
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| {
                let (ts, msg) = Self::parse_line(line);
                LogLine { timestamp: ts, message: msg, age: 0.0 }
            })
            .collect();

        if lines.is_empty() {
            return None;
        }

        let total = lines.len();
        let mut result = Vec::with_capacity(total);
        for (i, mut line) in lines.into_iter().enumerate() {
            line.age = i as f32 / total as f32;
            result.push(line);
        }

        Some(result)
    }

    fn parse_line(line: &str) -> (String, String) {
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() >= 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            (String::new(), line.to_string())
        }
    }

    pub fn get_recent(&self, count: usize) -> Vec<&LogLine> {
        self.lines.iter().take(count).collect()
    }
}