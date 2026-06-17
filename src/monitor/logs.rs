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
            // Try spawning with stdbuf -oL to force line-buffering on journalctl.
            // This prevents systemd from block-buffering stdout when piped.
            let mut child = Command::new("stdbuf")
                .args(["-oL", "journalctl", "--no-pager", "-f", "-n", "200", "-o", "short-iso", "--quiet"])
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn();

            // If stdbuf is not available, spawn journalctl directly.
            let mut child = match child {
                Ok(c) => Ok(c),
                Err(_) => Command::new("journalctl")
                    .args(["--no-pager", "-f", "-n", "200", "-o", "short-iso", "--quiet"])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn(),
            };

            // If journalctl is not available (or errors on spawn), fall back to tail -f
            let mut child = match child {
                Ok(c) => Ok(c),
                Err(_) => {
                    let syslog_candidates = [
                        "/var/log/syslog",
                        "/var/log/messages",
                        "/var/log/user.log",
                    ];
                    if let Some(path) = syslog_candidates.iter().find(|p| std::path::Path::new(p).exists()) {
                        Command::new("tail")
                            .args(["-f", "-n", "200", path])
                            .stdout(Stdio::piped())
                            .stderr(Stdio::null())
                            .spawn()
                    } else {
                        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No log source available"))
                    }
                }
            };

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
                        // Smooth out the stream to create a futuristic scrolling effect!
                        // This causes the initial backlog (and any bursts) to stream line-by-line visually.
                        std::thread::sleep(std::time::Duration::from_millis(30));
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
