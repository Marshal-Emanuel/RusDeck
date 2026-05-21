use std::collections::VecDeque;
use std::process::Command;

#[derive(Clone)]
pub struct CommandEntry {
    pub command: String,
    pub output: String,
}

pub struct TerminalWidget {
    pub history: VecDeque<CommandEntry>,
    pub current_input: String,
    cursor_visible: bool,
    max_history: usize,
}

impl TerminalWidget {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(50),
            current_input: String::new(),
            cursor_visible: true,
            max_history: 50,
        }
    }

    pub fn execute(&mut self, cmd: &str) {
        let trimmed = cmd.trim();
        if trimmed.is_empty() {
            return;
        }

        let output = if trimmed == "clear" {
            self.history.clear();
            return;
        } else if trimmed == "help" {
            "Available: clear, help, top, free, df -h, ps aux, uptime, whoami, hostname, date".to_string()
        } else {
            match Command::new("bash").args(["-c", trimmed]).output() {
                Ok(out) => {
                    let mut result = String::from_utf8_lossy(&out.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                    if !stderr.is_empty() {
                        if !result.is_empty() { result.push('\n'); }
                        result.push_str(&stderr);
                    }
                    if result.is_empty() { result = "(no output)".to_string(); }
                    result
                }
                Err(e) => format!("Error: {}", e),
            }
        };

        self.history.push_back(CommandEntry {
            command: trimmed.to_string(),
            output,
        });

        if self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }

    pub fn toggle_cursor(&mut self) {
        self.cursor_visible = !self.cursor_visible;
    }

    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }
}
