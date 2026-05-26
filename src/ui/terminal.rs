use std::collections::VecDeque;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

use portable_pty::{Child, CommandBuilder, NativePtySystem, PtySize, PtySystem};

#[derive(Clone, Copy)]
pub struct Cell {
    pub c: char,
    pub fg: [u8; 3],
    pub bg: [u8; 3],
    pub bold: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            fg: [207, 221, 225],
            bg: [0, 0, 0],
            bold: false,
        }
    }
}

#[derive(Clone)]
pub struct TerminalBuffer {
    pub lines: VecDeque<Vec<Cell>>,
    pub width: usize,
    pub height: usize,
    cursor_col: usize,
    cursor_row: usize,
    fg: [u8; 3],
    bg: [u8; 3],
    bold: bool,
}

impl TerminalBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            lines: VecDeque::from(vec![vec![Cell::default(); width]; height]),
            width,
            height,
            cursor_col: 0,
            cursor_row: 0,
            fg: [207, 221, 225],
            bg: [0, 0, 0],
            bold: false,
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
    pub fn cursor(&self) -> (usize, usize) { (self.cursor_col, self.cursor_row) }

    fn put_char(&mut self, c: char) {
        if self.cursor_col >= self.width {
            self.cursor_col = 0;
            self.cursor_row += 1;
            if self.cursor_row >= self.height {
                self.lines.pop_front();
                self.lines.push_back(vec![Cell::default(); self.width]);
                self.cursor_row = self.height - 1;
            }
        }
        if self.cursor_row < self.height && self.cursor_col < self.width {
            self.lines[self.cursor_row][self.cursor_col] = Cell {
                c,
                fg: self.fg,
                bg: self.bg,
                bold: self.bold,
            };
            self.cursor_col += 1;
        }
    }

    fn newline(&mut self) {
        self.cursor_col = 0;
        self.cursor_row += 1;
        if self.cursor_row >= self.height {
            self.lines.pop_front();
            self.lines.push_back(vec![Cell::default(); self.width]);
            self.cursor_row = self.height - 1;
        }
    }

    fn carriage_return(&mut self) {
        self.cursor_col = 0;
    }

    fn backspace(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
            self.lines[self.cursor_row][self.cursor_col] = Cell::default();
        }
    }

    fn erase_line(&mut self) {
        if self.cursor_row < self.height {
            for x in self.cursor_col..self.width {
                self.lines[self.cursor_row][x] = Cell::default();
            }
        }
    }

    fn erase_screen(&mut self) {
        for row in &mut self.lines {
            for cell in row {
                *cell = Cell::default();
            }
        }
        self.cursor_col = 0;
        self.cursor_row = 0;
    }

    fn scroll_up(&mut self) {
        self.lines.pop_front();
        self.lines.push_back(vec![Cell::default(); self.width]);
    }

    fn set_fg(&mut self, r: u8, g: u8, b: u8) {
        self.fg = [r, g, b];
    }

    fn set_bg(&mut self, r: u8, g: u8, b: u8) {
        self.bg = [r, g, b];
    }

    fn reset_attrs(&mut self) {
        self.fg = [207, 221, 225];
        self.bg = [0, 0, 0];
        self.bold = false;
    }
}

pub struct TerminalWidget {
    buffer: Arc<Mutex<TerminalBuffer>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    _child: Box<dyn Child + Send>,
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

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "fish".to_string());
        let cmd = CommandBuilder::new(&shell);
        let child = pair.slave.spawn_command(cmd).ok()?;

        let buffer = Arc::new(Mutex::new(TerminalBuffer::new(cols, rows)));
        let buffer_clone = Arc::clone(&buffer);
        let mut reader = pair.master.try_clone_reader().ok()?;
        let writer: Box<dyn Write + Send> = pair.master.take_writer().ok()?;
        let writer = Arc::new(Mutex::new(writer));

        let _reader = thread::spawn(move || {
            let mut buf = [0u8; 4096];
            let mut parser = AnsiParser::new();
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if let Ok(mut b) = buffer_clone.lock() {
                            parser.feed(&mut b, &buf[..n]);
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Some(Self {
            buffer,
            writer,
            _child: child,
            _reader,
        })
    }

    pub fn get_buffer(&self) -> &Arc<Mutex<TerminalBuffer>> {
        &self.buffer
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

struct AnsiParser {
    state: ParserState,
    params: Vec<i64>,
    current_param: String,
    has_intermediate: bool,
}

#[derive(Clone, Copy)]
enum ParserState {
    Normal,
    Escape,
    CsiEntry,
    OscEntry,
}

impl AnsiParser {
    fn new() -> Self {
        Self {
            state: ParserState::Normal,
            params: Vec::new(),
            current_param: String::new(),
            has_intermediate: false,
        }
    }

    fn feed(&mut self, buf: &mut TerminalBuffer, data: &[u8]) {
        for &b in data {
            match self.state {
                ParserState::Normal => {
                    match b {
                        0x1b => self.state = ParserState::Escape,
                        0x0a => buf.newline(),
                        0x0d => buf.carriage_return(),
                        0x08 | 0x7f => buf.backspace(),
                        0x0c => {
                            buf.erase_screen();
                        }
                        _ if b >= 0x20 => buf.put_char(b as char),
                        _ => {}
                    }
                }
                ParserState::Escape => {
                    match b {
                        b'[' => {
                            self.state = ParserState::CsiEntry;
                            self.params.clear();
                            self.current_param.clear();
                            self.has_intermediate = false;
                        }
                        b']' => {
                            self.state = ParserState::OscEntry;
                            self.current_param.clear();
                        }
                        _ => {
                            self.state = ParserState::Normal;
                        }
                    }
                }
                ParserState::CsiEntry => {
                    if b >= b'0' && b <= b'9' {
                        self.current_param.push(b as char);
                    } else if b == b';' {
                        self.push_param();
                    } else if b >= 0x20 && b <= 0x2f {
                        self.has_intermediate = true;
                    } else if b == b'?' || b == b'>' || b == b'!' || b == b'$' {
                        // Private mode prefix — skip it
                    } else {
                        self.push_param();
                        self.execute_csi(buf, b);
                        self.state = ParserState::Normal;
                    }
                }
                ParserState::OscEntry => {
                    if b == 0x07 || b == 0x1b {
                        // End of OSC sequence — ignore
                        self.state = ParserState::Normal;
                    } else {
                        self.current_param.push(b as char);
                    }
                }
            }
        }
    }

    fn push_param(&mut self) {
        if let Ok(n) = self.current_param.parse::<i64>() {
            self.params.push(n);
        } else if !self.current_param.is_empty() {
            self.params.push(0);
        }
        self.current_param.clear();
    }

    fn execute_csi(&mut self, buf: &mut TerminalBuffer, final_byte: u8) {
        if self.has_intermediate {
            return;
        }

        match final_byte {
            b'm' => self.set_sgr(buf),
            b'A' => {
                let n = self.params.first().copied().unwrap_or(1) as usize;
                buf.cursor_row = buf.cursor_row.saturating_sub(n);
            }
            b'B' => {
                let n = self.params.first().copied().unwrap_or(1) as usize;
                buf.cursor_row = (buf.cursor_row + n).min(buf.height - 1);
            }
            b'C' => {
                let n = self.params.first().copied().unwrap_or(1) as usize;
                buf.cursor_col = (buf.cursor_col + n).min(buf.width - 1);
            }
            b'D' => {
                let n = self.params.first().copied().unwrap_or(1) as usize;
                buf.cursor_col = buf.cursor_col.saturating_sub(n);
            }
            b'E' => {
                let n = self.params.first().copied().unwrap_or(1) as usize;
                buf.cursor_row = (buf.cursor_row + n).min(buf.height - 1);
                buf.cursor_col = 0;
            }
            b'F' => {
                let n = self.params.first().copied().unwrap_or(1) as usize;
                buf.cursor_row = buf.cursor_row.saturating_sub(n);
                buf.cursor_col = 0;
            }
            b'G' => {
                let n = self.params.first().copied().unwrap_or(1) as usize;
                buf.cursor_col = (n - 1).min(buf.width - 1);
            }
            b'H' | b'f' => {
                let row = self.params.first().copied().unwrap_or(1) as usize;
                let col = self.params.get(1).copied().unwrap_or(1) as usize;
                buf.cursor_row = (row - 1).min(buf.height - 1);
                buf.cursor_col = (col - 1).min(buf.width - 1);
            }
            b'J' => {
                match self.params.first().copied().unwrap_or(0) {
                    0 => {
                        for x in buf.cursor_col..buf.width {
                            buf.lines[buf.cursor_row][x] = Cell::default();
                        }
                        for row in (buf.cursor_row + 1)..buf.height {
                            for x in 0..buf.width {
                                buf.lines[row][x] = Cell::default();
                            }
                        }
                    }
                    1 => {
                        for x in 0..=buf.cursor_col {
                            buf.lines[buf.cursor_row][x] = Cell::default();
                        }
                        for row in 0..buf.cursor_row {
                            for x in 0..buf.width {
                                buf.lines[row][x] = Cell::default();
                            }
                        }
                    }
                    2 => buf.erase_screen(),
                    3 => buf.erase_screen(),
                    _ => {}
                }
            }
            b'K' => {
                match self.params.first().copied().unwrap_or(0) {
                    0 => {
                        for x in buf.cursor_col..buf.width {
                            buf.lines[buf.cursor_row][x] = Cell::default();
                        }
                    }
                    1 => {
                        for x in 0..=buf.cursor_col {
                            buf.lines[buf.cursor_row][x] = Cell::default();
                        }
                    }
                    2 => {
                        for x in 0..buf.width {
                            buf.lines[buf.cursor_row][x] = Cell::default();
                        }
                    }
                    _ => {}
                }
            }
            b'L' => {
                let n = self.params.first().copied().unwrap_or(1) as usize;
                for _ in 0..n {
                    if buf.cursor_row < buf.height {
                        buf.lines.insert(buf.cursor_row, vec![Cell::default(); buf.width]);
                        buf.lines.pop_back();
                    }
                }
            }
            b'M' => {
                let n = self.params.first().copied().unwrap_or(1) as usize;
                for _ in 0..n {
                    if buf.cursor_row < buf.height {
                        buf.lines.remove(buf.cursor_row);
                        buf.lines.push_back(vec![Cell::default(); buf.width]);
                    }
                }
            }
            b'P' => {
                let n = self.params.first().copied().unwrap_or(1) as usize;
                for x in (buf.cursor_col + n)..buf.width {
                    buf.lines[buf.cursor_row][x - n] = buf.lines[buf.cursor_row][x];
                }
                for x in (buf.width - n)..buf.width {
                    buf.lines[buf.cursor_row][x] = Cell::default();
                }
            }
            b'@' => {
                let n = self.params.first().copied().unwrap_or(1) as usize;
                for x in (buf.cursor_col..buf.width - n).rev() {
                    buf.lines[buf.cursor_row][x + n] = buf.lines[buf.cursor_row][x];
                }
                for x in buf.cursor_col..(buf.cursor_col + n).min(buf.width) {
                    buf.lines[buf.cursor_row][x] = Cell::default();
                }
            }
            b'h' | b'l' | b'r' => {
                // Set/reset mode, reverse — ignore
            }
            b'n' => {
                // Device status report — ignore
            }
            _ => {}
        }
    }

    fn set_sgr(&mut self, buf: &mut TerminalBuffer) {
        if self.params.is_empty() {
            buf.reset_attrs();
            return;
        }

        let mut i = 0;
        while i < self.params.len() {
            match self.params[i] {
                0 => buf.reset_attrs(),
                1 => buf.bold = true,
                2 => buf.bold = false,
                3 => buf.bold = true,
                22 => buf.bold = false,
                30..=37 => {
                    let c = ansi_color(self.params[i] as u8 - 30);
                    buf.set_fg(c[0], c[1], c[2]);
                }
                38 => {
                    if i + 2 < self.params.len() && self.params[i + 1] == 5 {
                        let c = extended_color(self.params[i + 2] as u8);
                        buf.set_fg(c[0], c[1], c[2]);
                        i += 2;
                    } else if i + 4 < self.params.len() && self.params[i + 1] == 2 {
                        let r = self.params[i + 2] as u8;
                        let g = self.params[i + 3] as u8;
                        let b = self.params[i + 4] as u8;
                        buf.set_fg(r, g, b);
                        i += 4;
                    }
                }
                39 => buf.set_fg(207, 221, 225),
                40..=47 => {
                    let c = ansi_color(self.params[i] as u8 - 40);
                    buf.set_bg(c[0], c[1], c[2]);
                }
                48 => {
                    if i + 2 < self.params.len() && self.params[i + 1] == 5 {
                        let c = extended_color(self.params[i + 2] as u8);
                        buf.set_bg(c[0], c[1], c[2]);
                        i += 2;
                    } else if i + 4 < self.params.len() && self.params[i + 1] == 2 {
                        let r = self.params[i + 2] as u8;
                        let g = self.params[i + 3] as u8;
                        let b = self.params[i + 4] as u8;
                        buf.set_bg(r, g, b);
                        i += 4;
                    }
                }
                49 => buf.set_bg(0, 0, 0),
                90..=97 => {
                    let c = bright_color(self.params[i] as u8 - 90);
                    buf.set_fg(c[0], c[1], c[2]);
                }
                100..=107 => {
                    let c = bright_color(self.params[i] as u8 - 100);
                    buf.set_bg(c[0], c[1], c[2]);
                }
                _ => {}
            }
            i += 1;
        }
    }
}

fn ansi_color(n: u8) -> [u8; 3] {
    match n {
        0 => [0, 0, 0],
        1 => [205, 0, 0],
        2 => [0, 205, 0],
        3 => [205, 205, 0],
        4 => [0, 0, 238],
        5 => [205, 0, 205],
        6 => [0, 205, 205],
        7 => [229, 229, 229],
        _ => [229, 229, 229],
    }
}

fn bright_color(n: u8) -> [u8; 3] {
    match n {
        0 => [127, 127, 127],
        1 => [255, 85, 85],
        2 => [85, 255, 85],
        3 => [255, 255, 85],
        4 => [85, 85, 255],
        5 => [255, 85, 255],
        6 => [85, 255, 255],
        7 => [255, 255, 255],
        _ => [255, 255, 255],
    }
}

fn extended_color(n: u8) -> [u8; 3] {
    match n {
        0..=7 => ansi_color(n),
        8..=15 => bright_color(n - 8),
        16..=231 => {
            let n = n - 16;
            let r = ((n / 36) * 51) as u8;
            let g = (((n / 6) % 6) * 51) as u8;
            let b = ((n % 6) * 51) as u8;
            [r, g, b]
        }
        232..=255 => {
            let v = ((n - 232) * 10 + 8) as u8;
            [v, v, v]
        }
        _ => [255, 255, 255],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrolling_direction() {
        let mut buf = TerminalBuffer::new(10, 3);
        // Write text to fill the buffer
        // Row 0
        buf.put_char('a');
        buf.newline();
        // Row 1
        buf.put_char('b');
        buf.newline();
        // Row 2
        buf.put_char('c');
        
        // At this point, the buffer should contain:
        // Row 0: 'a'
        // Row 1: 'b'
        // Row 2: 'c'
        assert_eq!(buf.lines[0][0].c, 'a');
        assert_eq!(buf.lines[1][0].c, 'b');
        assert_eq!(buf.lines[2][0].c, 'c');
        
        // Trigger scrolling by adding another newline and char
        buf.newline();
        buf.put_char('d');
        
        // Now, it should have scrolled UP:
        // Row 0 should be 'b'
        // Row 1 should be 'c'
        // Row 2 should be 'd'
        assert_eq!(buf.lines[0][0].c, 'b');
        assert_eq!(buf.lines[1][0].c, 'c');
        assert_eq!(buf.lines[2][0].c, 'd');
    }
}
