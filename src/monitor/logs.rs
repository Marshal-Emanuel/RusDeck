use std::collections::VecDeque;
use std::process::Command;

#[derive(Clone)]
pub struct LogLine {
    pub timestamp: String,
    pub message: String,
}

pub struct LogBuffer {
    lines: VecDeque<LogLine>,
    max_cap: usize,
}

impl LogBuffer {
    pub fn new(max_cap: usize) -> Self {
        Self {
            lines: VecDeque::with_capacity(max_cap),
            max_cap,
        }
    }

    pub fn poll(&mut self) {
        let Some(new_lines) = Self::read_journalctl()
            .or_else(|| Self::read_syslog_file())
        else { return };

        self.lines.clear();
        for line in new_lines.into_iter().rev() {
            if self.lines.len() >= self.max_cap {
                break;
            }
            self.lines.push_front(line);
        }
    }

    fn read_journalctl() -> Option<Vec<LogLine>> {
        let output = Command::new("journalctl")
            .args(["--no-pager", "-n", "50", "-o", "short-iso", "--quiet"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8(output.stdout).ok()?;
        Some(Self::parse_lines(&text))
    }

    fn read_syslog_file() -> Option<Vec<LogLine>> {
        let candidates = [
            "/var/log/syslog",
            "/var/log/messages",
            "/var/log/user.log",
        ];
        let path = candidates.iter().find(|p| std::path::Path::new(p).exists())?;
        let content = std::fs::read_to_string(path).ok()?;
        let lines: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();
        let last = if lines.len() > 50 { &lines[lines.len() - 50..] } else { &lines };
        Some(Self::parse_lines(&last.join("\n")))
    }

    fn parse_lines(text: &str) -> Vec<LogLine> {
        text.lines()
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
            .collect()
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
