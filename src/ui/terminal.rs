use std::collections::VecDeque;
use std::process::Command;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

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
    tx: Sender<(String, String)>,
    rx: Receiver<(String, String)>,
    pending_cmd: Option<String>,
}

impl TerminalWidget {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            history: VecDeque::with_capacity(50),
            current_input: String::new(),
            cursor_visible: true,
            max_history: 50,
            tx,
            rx,
            pending_cmd: None,
        }
    }

    pub fn execute(&mut self) {
        let trimmed = self.current_input.trim().to_string();
        if trimmed.is_empty() {
            return;
        }

        if trimmed == "clear" {
            self.history.clear();
            self.current_input.clear();
            return;
        }

        self.history.push_back(CommandEntry {
            command: trimmed.clone(),
            output: "(running...)".to_string(),
        });

        if self.history.len() > self.max_history {
            self.history.pop_front();
        }

        self.current_input.clear();
        self.pending_cmd = Some(trimmed.clone());

        let tx = self.tx.clone();
        thread::spawn(move || {
            let result = if trimmed == "help" {
                "Available: clear, help, top, free, df -h, ps aux, uptime, whoami, hostname, date".to_string()
            } else {
                match Command::new("bash").args(["-c", &trimmed]).output() {
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
            let _ = tx.send((trimmed, result));
        });
    }

    pub fn poll_results(&mut self) {
        while let Ok((cmd, output)) = self.rx.try_recv() {
            if let Some(entry) = self.history.iter_mut().rev().find(|e| e.command == cmd) {
                entry.output = output;
            }
        }
    }

    pub fn append_char(&mut self, c: char) {
        self.current_input.push(c);
    }

    pub fn backspace(&mut self) {
        self.current_input.pop();
    }

    pub fn toggle_cursor(&mut self) {
        self.cursor_visible = !self.cursor_visible;
    }

    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }
}
