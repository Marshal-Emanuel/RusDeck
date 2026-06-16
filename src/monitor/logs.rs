use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

#[derive(Clone)]
pub struct LogLine {
    pub timestamp: String,
    pub message: String,
}

pub struct LogBuffer {
    lines: VecDeque<LogLine>,
    max_cap: usize,
    rx: Receiver<LogLine>,
}

impl LogBuffer {
    pub fn new(max_cap: usize) -> Self {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            // Try journalctl -f first; fall back to tail -f on a syslog file
            let mut child = Command::new("journalctl")
                .args(["--no-pager", "-f", "-n", "200", "-o", "short-iso", "--quiet"])
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn();

            if child.is_err() || !child.as_ref().map(|_| true).unwrap_or(false) {
                // journalctl unavailable — fall back to tail -f on syslog
                let syslog_candidates = [
                    "/var/log/syslog",
                    "/var/log/messages",
                    "/var/log/user.log",
                ];
                if let Some(path) = syslog_candidates.iter().find(|p| std::path::Path::new(p).exists()) {
                    child = Command::new("tail")
                        .args(["-f", "-n", "200", path])
                        .stdout(Stdio::piped())
                        .stderr(Stdio::null())
                        .spawn();
                }
            }

            let mut child = match child {
                Ok(c) => c,
                Err(_) => return, // Neither source available — give up silently
            };

            let stdout = match child.stdout.take() {
                Some(s) => s,
                None => return,
            };

            let reader = BufReader::new(stdout);
            for raw_line in reader.lines() {
                match raw_line {
                    Ok(line) if !line.is_empty() => {
                        let parsed = Self::parse_line(&line);
                        // If the channel receiver is gone (app exited), stop streaming
                        if tx.send(parsed).is_err() {
                            break;
                        }
                    }
                    _ => {}
                }
            }

            // Clean up the child process when the stream ends
            let _ = child.wait();
        });

        Self {
            lines: VecDeque::with_capacity(max_cap),
            max_cap,
            rx,
        }
    }

    /// Drain all available lines from the stream channel and append them to the buffer.
    /// This is non-blocking — if no new lines arrived since last call it returns immediately.
    pub fn poll(&mut self) {
        loop {
            match self.rx.try_recv() {
                Ok(log_line) => {
                    self.lines.push_back(log_line);
                    if self.lines.len() > self.max_cap {
                        self.lines.pop_front();
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
    }

    fn parse_line(line: &str) -> LogLine {
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() >= 2 {
            LogLine {
                timestamp: parts[0].to_string(),
                message: parts[1].to_string(),
            }
        } else {
            LogLine {
                timestamp: String::new(),
                message: line.to_string(),
            }
        }
    }

    pub fn get_recent(&self, count: usize) -> Vec<&LogLine> {
        // Show the most recently received lines at the bottom — natural stream order
        let skip = self.lines.len().saturating_sub(count);
        self.lines.iter().skip(skip).collect()
    }
}
