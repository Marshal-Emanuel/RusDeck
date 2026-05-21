use std::collections::VecDeque;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

use portable_pty::{CommandBuilder, MasterPty, NativePtySystem, PtySize, PtySystem};

#[derive(Clone, Copy, Default)]
pub struct Cell {
    pub c: char,
    pub fg: [u8; 3],
    pub bg: [u8; 3],
    pub bold: bool,
}

#[derive(Clone)]
pub struct TerminalBuffer {
    pub lines: VecDeque<Vec<Cell>>,
    pub width: usize,
    pub height: usize,
    cursor_col: usize,
    cursor_row: usize,
}

impl TerminalBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let lines = vec![vec![Cell::default(); width]; height];
        Self {
            lines: VecDeque::from(lines),
            width,
            height,
            cursor_col: 0,
            cursor_row: 0,
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.lines = VecDeque::from(vec![vec![Cell::default(); width]; height]);
        self.cursor_col = 0;
        self.cursor_row = 0;
    }

    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }

    pub fn cursor(&self) -> (usize, usize) {
        (self.cursor_col, self.cursor_row)
    }

    pub fn add_text(&mut self, text: &str) {
        for c in text.chars() {
            match c {
                '\n' | '\r' => {
                    self.cursor_col = 0;
                    self.cursor_row += 1;
                    if self.cursor_row >= self.height {
                        self.lines.pop_back();
                        self.lines.push_front(vec![Cell::default(); self.width]);
                        self.cursor_row = self.height - 1;
                    }
                }
                '\x08' | '\x7f' => {
                    if self.cursor_col > 0 {
                        self.cursor_col -= 1;
                        self.lines[self.cursor_row][self.cursor_col] = Cell::default();
                    }
                }
                _ => {
                    if self.cursor_col < self.width {
                        self.lines[self.cursor_row][self.cursor_col] = Cell {
                            c,
                            fg: [0, 255, 204],
                            bg: [0, 0, 0],
                            bold: false,
                        };
                        self.cursor_col += 1;
                    } else {
                        self.cursor_col = 0;
                        self.cursor_row += 1;
                        if self.cursor_row >= self.height {
                            self.lines.pop_back();
                            self.lines.push_front(vec![Cell::default(); self.width]);
                            self.cursor_row = self.height - 1;
                        }
                        self.lines[self.cursor_row][self.cursor_col] = Cell {
                            c,
                            fg: [0, 255, 204],
                            bg: [0, 0, 0],
                            bold: false,
                        };
                        self.cursor_col += 1;
                    }
                }
            }
        }
    }
}

pub struct TerminalWidget {
    buffer: Arc<Mutex<TerminalBuffer>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    _reader: thread::JoinHandle<()>,
}

impl TerminalWidget {
    pub fn new(cols: usize, rows: usize) -> Option<Self> {
        let pty_system = NativePtySystem::default();
        let pair = pty_system.openpty(PtySize {
            rows: rows as u16,
            cols: cols as u16,
            pixel_width: 0,
            pixel_height: 0,
        }).ok()?;

        let mut cmd = CommandBuilder::new("bash");
        cmd.arg("-i");
        let child = pair.slave.spawn_command(cmd).ok()?;

        let buffer = Arc::new(Mutex::new(TerminalBuffer::new(cols, rows)));
        let buffer_clone = Arc::clone(&buffer);
        let mut reader = pair.master.try_clone_reader().ok()?;
        let writer: Box<dyn Write + Send> = pair.master.take_writer().ok()?;
        let writer = Arc::new(Mutex::new(writer));

        let _reader = thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let text = String::from_utf8_lossy(&buf[..n]).to_string();
                        if let Ok(mut b) = buffer_clone.lock() {
                            b.add_text(&text);
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        drop(child);

        Some(Self {
            buffer,
            writer,
            _reader,
        })
    }

    pub fn get_buffer(&self) -> Arc<Mutex<TerminalBuffer>> {
        Arc::clone(&self.buffer)
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        if let Ok(mut buf) = self.buffer.lock() {
            buf.resize(cols, rows);
        }
    }

    pub fn write_input(&mut self, data: &[u8]) {
        if let Ok(mut w) = self.writer.lock() {
            let _ = w.write_all(data);
            let _ = w.flush();
        }
    }

    pub fn handle_key(&mut self, key: &str, ctrl: bool) {
        let bytes: &[u8] = match key {
            "Enter" => b"\r",
            "Backspace" => b"\x7f",
            "Tab" => b"\t",
            "Escape" => b"\x1b",
            "ArrowUp" => b"\x1b[A",
            "ArrowDown" => b"\x1b[B",
            "ArrowRight" => b"\x1b[C",
            "ArrowLeft" => b"\x1b[D",
            "Home" => b"\x1b[H",
            "End" => b"\x1b[F",
            "PageUp" => b"\x1b[5~",
            "PageDown" => b"\x1b[6~",
            "Delete" => b"\x1b[3~",
            "F1" => b"\x1bOP",
            "F2" => b"\x1bOQ",
            "F3" => b"\x1bOR",
            "F4" => b"\x1bOS",
            "F5" => b"\x1b[15~",
            "F6" => b"\x1b[17~",
            "F7" => b"\x1b[18~",
            "F8" => b"\x1b[19~",
            "F9" => b"\x1b[20~",
            "F10" => b"\x1b[21~",
            "F11" => b"\x1b[23~",
            "F12" => b"\x1b[24~",
            "c" if ctrl => b"\x03",
            "d" if ctrl => b"\x04",
            "z" if ctrl => b"\x1a",
            "l" if ctrl => b"\x0c",
            "u" if ctrl => b"\x15",
            "k" if ctrl => b"\x0b",
            _ => return,
        };
        self.write_input(bytes);
    }

    pub fn handle_char(&mut self, c: char) {
        let mut buf = [0u8; 4];
        let encoded = c.encode_utf8(&mut buf);
        self.write_input(encoded.as_bytes());
    }
}
