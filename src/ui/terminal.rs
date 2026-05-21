use std::collections::VecDeque;
use std::process::Command;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Default)]
pub struct Cell {
    pub c: char,
    pub fg: [u8; 3],
    pub bg: [u8; 3],
    pub bold: bool,
}

#[derive(Clone)]
pub struct TerminalBuffer {
    lines: VecDeque<Vec<Cell>>,
    scrollback: VecDeque<String>,
    scrollback_lines: usize,
    width: usize,
    height: usize,
}

impl TerminalBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let lines = vec![vec![Cell::default(); width]; height];
        Self {
            lines: VecDeque::from(lines),
            scrollback: VecDeque::new(),
            scrollback_lines: 500,
            width,
            height,
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.lines = VecDeque::from(vec![vec![Cell::default(); width]; height]);
    }

    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }

    pub fn get_cell(&self, col: usize, row: usize) -> Cell {
        if row < self.lines.len() {
            if let Some(line) = self.lines.get(row) {
                if col < line.len() {
                    return line[col];
                }
            }
        }
        Cell::default()
    }

    pub fn get_all_cells(&self) -> Vec<Vec<Cell>> {
        self.lines.iter().map(|l| l.clone()).collect()
    }

    pub fn cursor(&self) -> (usize, usize) {
        (0, 0)
    }

    pub fn add_line(&mut self, text: &str) {
        let mut row: Vec<Cell> = Vec::with_capacity(self.width);
        for c in text.chars() {
            row.push(Cell {
                c,
                fg: [0, 255, 204],
                bg: [0, 0, 0],
                bold: false,
            });
            if row.len() >= self.width {
                row.pop();
            }
        }
        while row.len() < self.width {
            row.push(Cell::default());
        }

        if self.lines.len() >= self.height {
            let removed = self.lines.pop_back().unwrap();
            let line_str: String = removed.iter().map(|c| c.c).collect();
            if self.scrollback.len() >= self.scrollback_lines {
                self.scrollback.pop_back();
            }
            self.scrollback.push_front(line_str);
            self.lines.push_front(row);
        } else {
            self.lines.push_front(row);
        }
    }
}

pub struct PtyHandler;

impl PtyHandler {
    pub fn new(_cols: usize, _rows: usize) -> Option<Self> {
        Some(Self)
    }

    pub fn write_input(&mut self, _data: &[u8]) {}
}

pub struct TerminalWidget {
    buffer: TerminalBuffer,
    handler: PtyHandler,
}

impl TerminalWidget {
    pub fn new(cols: usize, rows: usize) -> Option<Self> {
        Some(Self {
            buffer: TerminalBuffer::new(cols, rows),
            handler: PtyHandler::new(cols, rows)?,
        })
    }

    pub fn get_buffer(&self) -> Arc<Mutex<TerminalBuffer>> {
        Arc::new(Mutex::new(self.buffer.clone()))
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        self.buffer.resize(cols, rows);
    }

    pub fn write_input(&mut self, _data: &[u8]) {
        self.handler.write_input(_data);
    }

    pub fn handle_key(&mut self, key: &str, _mods: TerminalModifiers) {
        if key == "Enter" {
            self.buffer.add_line("$ ");
        }
    }

    pub fn handle_char(&mut self, c: char) {
        self.buffer.add_line(&c.to_string());
    }

    pub fn execute_command(&mut self, cmd: &str) {
        let output = Command::new("bash")
            .args(["-c", cmd])
            .output();

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);

            for line in stdout.lines() {
                self.buffer.add_line(line);
            }
            for line in stderr.lines() {
                self.buffer.add_line(&format!("err: {}", line));
            }
        } else {
            self.buffer.add_line("Failed to execute command");
        }
    }
}

impl Clone for TerminalWidget {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            handler: PtyHandler::new(self.buffer.width, self.buffer.height).unwrap(),
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct TerminalModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl TerminalModifiers {
    pub fn new() -> Self {
        Self::default()
    }
}