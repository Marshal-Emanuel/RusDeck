use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};

#[derive(Clone)]
pub struct LogLine {
    pub timestamp: String,
    pub message: String,
}

pub struct LogBuffer {
    lines: VecDeque<LogLine>,
    max_cap: usize,
    child: Option<Child>,
    reader: Option<BufReader<std::process::ChildStdout>>,
}

impl LogBuffer {
    pub fn new(max_cap: usize) -> Self {
        Self {
            lines: VecDeque::with_capacity(max_cap),
            max_cap,
            child: None,
            reader: None,
        }
    }

    pub fn poll(&mut self) {
        if self.reader.is_none() {
            self.start_journalctl();
        }

        if let Some(reader) = &mut self.reader {
            let mut buf = String::new();
            loop {
                buf.clear();
                match reader.read_line(&mut buf) {
                    Ok(0) => break,
                    Ok(_) => {
                        let line = buf.trim_end();
                        if line.is_empty() || Self::is_noise(line) {
                            continue;
                        }
                        let parts: Vec<&str> = line.splitn(2, ' ').collect();
                        let (ts, msg) = if parts.len() >= 2 {
                            (parts[0].to_string(), parts[1].to_string())
                        } else {
                            (String::new(), line.to_string())
                        };
                        if self.lines.len() >= self.max_cap {
                            self.lines.pop_back();
                        }
                        self.lines.push_front(LogLine { timestamp: ts, message: msg });
                    }
                    Err(_) => break,
                }
            }
        } else if let Some(new_lines) = Self::read_syslog_file() {
            self.lines.clear();
            for line in new_lines.into_iter().rev() {
                if self.lines.len() >= self.max_cap {
                    break;
                }
                self.lines.push_front(line);
            }
        }
    }

    fn start_journalctl(&mut self) {
        let child = Command::new("journalctl")
            .args(["-f", "-o", "short-iso", "--quiet"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn();

        let mut child = match child {
            Ok(c) => c,
            Err(_) => return,
        };

        let stdout = match child.stdout.take() {
            Some(s) => s,
            None => return,
        };

        self.reader = Some(BufReader::new(stdout));
        self.child = Some(child);
    }

    fn read_syslog_file() -> Option<Vec<LogLine>> {
        let candidates = [
            "/var/log/syslog",
            "/var/log/messages",
            "/var/log/user.log",
        ];
        let path = candidates.iter().find(|p| std::path::Path::new(p).exists())?;
        let content = std::fs::read_to_string(path).ok()?;
        let log_lines: Vec<LogLine> = content.lines()
            .filter(|l| !l.is_empty())
            .filter(|l| !Self::is_noise(l))
            .map(|line| {
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                let (ts, msg) = if parts.len() >= 2 {
                    (parts[0].to_string(), parts[1].to_string())
                } else {
                    (String::new(), line.to_string())
                };
                LogLine { timestamp: ts, message: msg }
            })
            .collect();
        let last = if log_lines.len() > 50 { &log_lines[log_lines.len() - 50..] } else { &log_lines };
        Some(last.to_vec())
    }

    fn is_noise(line: &str) -> bool {
        let patterns = [
            "Can't update stage views actor",
            "client bug: event processing lagging behind",
            "meta_window_set_geom",
        ];
        patterns.iter().any(|p| line.contains(p))
    }

    pub fn get_recent(&self, count: usize) -> Vec<&LogLine> {
        self.lines.iter().take(count).collect()
    }
}
