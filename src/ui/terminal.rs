use std::collections::VecDeque;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Cell {
    pub c: char,
    pub fg: [u8; 3],
    pub bg: [u8; 3],
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            fg: [0, 255, 204],
            bg: [0, 0, 0],
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

pub struct TerminalBuffer {
    pub cells: Vec<Vec<Cell>>,
    width: usize,
    height: usize,
    pub scrollback: VecDeque<String>,
    scrollback_lines: usize,
    cursor_col: usize,
    cursor_row: usize,
    saved_cursor_col: usize,
    saved_cursor_row: usize,
    current_fg: [u8; 3],
    current_bg: [u8; 3],
    current_bold: bool,
    current_italic: bool,
    current_underline: bool,
    origin_mode: bool,
    auto_wrap: bool,
    application_cursor_keys: bool,
    bracketed_paste: bool,
}

impl TerminalBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![vec![Cell::default(); width]; height];
        Self {
            cells,
            width,
            height,
            scrollback: VecDeque::new(),
            scrollback_lines: 1000,
            cursor_col: 0,
            cursor_row: 0,
            saved_cursor_col: 0,
            saved_cursor_row: 0,
            current_fg: [0, 255, 204],
            current_bg: [0, 0, 0],
            current_bold: false,
            current_italic: false,
            current_underline: false,
            origin_mode: false,
            auto_wrap: true,
            application_cursor_keys: false,
            bracketed_paste: false,
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        if width == self.width && height == self.height {
            return;
        }

        let mut new_cells = vec![vec![Cell::default(); width]; height];

        let copy_w = width.min(self.width);
        let copy_h = height.min(self.height);

        for y in 0..copy_h {
            for x in 0..copy_w {
                new_cells[y][x] = self.cells[y][x].clone();
            }
        }

        self.cells = new_cells;
        self.width = width;
        self.height = height;
        self.cursor_col = self.cursor_col.min(width - 1);
        self.cursor_row = self.cursor_row.min(height - 1);
    }

    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }
    pub fn cursor(&self) -> (usize, usize) { (self.cursor_col, self.cursor_row) }

    fn set_cell(&mut self, col: usize, row: usize, c: char) {
        if col >= self.width || row >= self.height {
            return;
        }
        self.cells[row][col] = Cell {
            c,
            fg: self.current_fg,
            bg: self.current_bg,
            bold: self.current_bold,
            italic: self.current_italic,
            underline: self.current_underline,
        };
    }

    fn set_cursor(&mut self, col: usize, row: usize) {
        self.cursor_col = col.min(self.width - 1);
        self.cursor_row = row.min(self.height - 1);
    }

    fn save_cursor(&mut self) {
        self.saved_cursor_col = self.cursor_col;
        self.saved_cursor_row = self.cursor_row;
    }

    fn restore_cursor(&mut self) {
        self.set_cursor(self.saved_cursor_col, self.saved_cursor_row);
    }

    fn clear_attrs(&mut self) {
        self.current_fg = [0, 255, 204];
        self.current_bg = [0, 0, 0];
        self.current_bold = false;
        self.current_italic = false;
        self.current_underline = false;
    }

    fn scroll_up(&mut self) {
        let line = self.get_line_as_string(0);
        if self.scrollback.len() >= self.scrollback_lines {
            self.scrollback.pop_back();
        }
        self.scrollback.push_front(line);
        self.cells.remove(0);
        self.cells.push(vec![Cell::default(); self.width]);
    }

    fn scroll_down(&mut self) {
        if !self.cells.is_empty() {
            self.cells.insert(0, vec![Cell::default(); self.width]);
            self.cells.pop();
        }
    }

    fn get_line_as_string(&self, row: usize) -> String {
        if row >= self.cells.len() {
            return String::new();
        }
        self.cells[row].iter().map(|c| c.c).collect()
    }

    fn clear_screen(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                *cell = Cell::default();
            }
        }
        self.cursor_col = 0;
        self.cursor_row = 0;
    }

    fn clear_eol(&mut self) {
        if self.cursor_row < self.cells.len() {
            for x in self.cursor_col..self.width {
                self.cells[self.cursor_row][x] = Cell::default();
            }
        }
    }

    fn newline(&mut self) {
        self.cursor_row += 1;
        if self.cursor_row >= self.height {
            self.scroll_up();
            self.cursor_row = self.height - 1;
        }
    }

    fn carriage_return(&mut self) {
        self.cursor_col = 0;
    }

    fn backspace(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }
}

pub struct PtyHandler {
    buffer: Arc<Mutex<TerminalBuffer>>,
    writer: Box<dyn Write + Send>,
    _reader: thread::JoinHandle<()>,
}

impl PtyHandler {
    pub fn new(cols: usize, rows: usize) -> Option<Self> {
        let pair = portable_pty::native_pty_system()
            .openpty(portable_pty::PtySize {
                rows: rows as u16,
                cols: cols as u16,
                pixel_width: 0,
                pixel_height: 0,
            })
            .ok()?;

        let mut cmd = portable_pty::CommandBuilder::new(
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
        );

        let child = pair.slave.spawn_command(cmd).ok()?;

        let mut master = pair.master;

        let buffer = Arc::new(Mutex::new(TerminalBuffer::new(cols, rows)));

        let buffer_clone = buffer.clone();
        let _reader = thread::spawn(move || {
            let mut parser = vte::Parser::new();
            let mut perform = vte::Perform::new();
            let mut buf = [0u8; 8192];

            let reader = master.take().unwrap();
            let mut rdr = std::io::BufReader::new(reader);

            loop {
                match rdr.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        for &b in &buf[..n] {
                            parser.advance(&mut perform, b);
                            if let Some(cmd) = perform.take_cmd() {
                                vte::Execute(&mut TerminalBackend { buffer: buffer_clone.clone() }, &cmd);
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let writer = Box::new(master.take().unwrap());

        Some(Self {
            buffer,
            writer,
            _reader,
        })
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        if let Ok(mut buf) = self.buffer.lock() {
            buf.resize(cols, rows);
        }
    }

    pub fn write_input(&mut self, data: &[u8]) {
        let _ = self.writer.write_all(data);
        let _ = self.writer.flush();
    }

    pub fn get_buffer(&self) -> Arc<Mutex<TerminalBuffer>> {
        self.buffer.clone()
    }
}

impl Drop for PtyHandler {
    fn drop(&mut self) {
        let _ = self.writer.write_all(b"exit\n");
    }
}

struct TerminalBackend {
    buffer: Arc<Mutex<TerminalBuffer>>,
}

impl vte::Perform for TerminalBackend {
    fn print(&mut self, c: char) {
        if let Ok(mut buf) = self.buffer.lock() {
            if buf.cursor_col >= buf.width {
                if buf.auto_wrap {
                    buf.newline();
                    buf.carriage_return();
                } else {
                    buf.cursor_col = buf.width - 1;
                }
            }
            buf.set_cell(buf.cursor_col, buf.cursor_row, c);
            buf.cursor_col += 1;
        }
    }

    fn execute(&mut self, c: char) {
        if let Ok(mut buf) = self.buffer.lock() {
            match c {
                '\n' => buf.newline(),
                '\r' => buf.carriage_return(),
                '\t' => {
                    let next = (buf.cursor_col / 8 + 1) * 8;
                    buf.cursor_col = next.min(buf.width);
                }
                '\x07' => {}
                '\x08' => buf.backspace(),
                _ => {}
            }
        }
    }

    fn crlf(&mut self) {
        if let Ok(mut buf) = self.buffer.lock() {
            buf.newline();
            buf.carriage_return();
        }
    }

    fn move_cursor(&mut self, col: u16, row: u16) {
        if let Ok(mut buf) = self.buffer.lock() {
            buf.set_cursor(col as usize, row as usize);
        }
    }

    fn cursor_up1(&mut self) {
        if let Ok(mut buf) = self.buffer.lock() {
            if buf.cursor_row > 0 {
                buf.cursor_row -= 1;
            }
        }
    }

    fn cursor_down1(&mut self) {
        if let Ok(mut buf) = self.buffer.lock() {
            buf.cursor_row += 1;
            if buf.cursor_row >= buf.height {
                buf.scroll_up();
                buf.cursor_row = buf.height - 1;
            }
        }
    }

    fn cursor_forward1(&mut self) {
        if let Ok(mut buf) = self.buffer.lock() {
            buf.cursor_col += 1;
            if buf.cursor_col >= buf.width {
                buf.cursor_col = buf.width - 1;
            }
        }
    }

    fn cursor_backward1(&mut self) {
        if let Ok(mut buf) = self.buffer.lock() {
            buf.backspace();
        }
    }

    fn erase_chars(&mut self, count: u16) {
        if let Ok(mut buf) = self.buffer.lock() {
            for i in 0..count as usize {
                let col = (buf.cursor_col + i).min(buf.width - 1);
                buf.set_cell(col, buf.cursor_row, ' ');
            }
        }
    }

    fn delete_chars(&mut self, count: u16) {
        if let Ok(mut buf) = self.buffer.lock() {
            for i in 0..count as usize {
                let col = buf.cursor_col + i;
                if col < buf.width - 1 {
                    buf.set_cell(col, buf.cursor_row, ' ');
                }
            }
        }
    }

    fn erase_in_display(&mut self, mode: vte::EraseMode) {
        if let Ok(mut buf) = self.buffer.lock() {
            match mode {
                vte::EraseMode::All | vte::EraseMode::Above => {
                    for y in 0..=buf.cursor_row {
                        for x in 0..buf.width {
                            buf.set_cell(x, y, ' ');
                        }
                    }
                }
                vte::EraseMode::Below => {
                    for y in buf.cursor_row..buf.height {
                        for x in 0..buf.width {
                            buf.set_cell(x, y, ' ');
                        }
                    }
                }
                vte::EraseMode::Line => buf.clear_eol(),
                _ => {}
            }
        }
    }

    fn erase_in_line(&mut self, mode: vte::EraseMode) {
        if let Ok(mut buf) = self.buffer.lock() {
            match mode {
                vte::EraseMode::Left | vte::EraseMode::All => {
                    for x in 0..=buf.cursor_col {
                        buf.set_cell(x, buf.cursor_row, ' ');
                    }
                }
                vte::EraseMode::Right => buf.clear_eol(),
                _ => {}
            }
        }
    }

    fn set_scrolling_region(&mut self, top: u16, bottom: u16) {
    }

    fn save_cursor_position(&mut self) {
        if let Ok(mut buf) = self.buffer.lock() {
            buf.save_cursor();
        }
    }

    fn restore_cursor_position(&mut self) {
        if let Ok(mut buf) = self.buffer.lock() {
            buf.restore_cursor();
        }
    }

    fn set_mode(&mut self, mode: vte::Mode) {
    }

    fn unset_mode(&mut self, mode: vte::Mode) {
    }

    fn set_attribute(&mut self, attr: vte::Attr) {
        if let Ok(mut buf) = self.buffer.lock() {
            match attr {
                vte::Attr::Reset => buf.clear_attrs(),
                vte::Attr::Bold => buf.current_bold = true,
                vte::Attr::Italic => buf.current_italic = true,
                vte::Attr::Underline => buf.current_underline = true,
                vte::Attr::Reverse => {}
                vte::Attr::FgColor(vte::Color::Idx(n)) => {
                    if let Some(c) = ansi_color(n) {
                        buf.current_fg = c;
                    }
                }
                vte::Attr::FgColor(vte::Color::Rgb(r, g, b)) => {
                    buf.current_fg = [r, g, b];
                }
                vte::Attr::BgColor(vte::Color::Idx(n)) => {
                    if let Some(c) = ansi_color(n) {
                        buf.current_bg = c;
                    }
                }
                vte::Attr::BgColor(vte::Color::Rgb(r, g, b)) => {
                    buf.current_bg = [r, g, b];
                }
                _ => {}
            }
        }
    }

    fn set_window_mode(&mut self, mode: vte::WindowMode) {}

    fn focus_in(&mut self) {}
    fn focus_out(&mut self) {}

    fn verbatim_insert(&mut self, c: char) {
        self.print(c);
    }
}

fn ansi_color(n: u8) -> Option<[u8; 3]> {
    match n {
        0 => Some([0, 0, 0]),
        1 => Some([205, 0, 0]),
        2 => Some([0, 205, 0]),
        3 => Some([205, 205, 0]),
        4 => Some([0, 0, 238]),
        5 => Some([205, 0, 205]),
        6 => Some([0, 205, 205]),
        7 => Some([229, 229, 229]),
        8 => Some([127, 127, 127]),
        9 => Some([255, 0, 0]),
        10 => Some([0, 255, 0]),
        11 => Some([255, 255, 0]),
        12 => Some([92, 92, 255]),
        13 => Some([255, 0, 255]),
        14 => Some([0, 255, 255]),
        15 => Some([255, 255, 255]),
        16..=231 => {
            let n = n - 16;
            let r = ((n / 36) * 51) as u8;
            let g = (((n / 6) % 6) * 51) as u8;
            let b = ((n % 6) * 51) as u8;
            Some([r, g, b])
        }
        232..=255 => {
            let n = n - 232;
            let v = (n * 10 + 8) as u8;
            Some([v, v, v])
        }
        _ => None,
    }
}

pub struct TerminalWidget {
    handler: PtyHandler,
}

impl TerminalWidget {
    pub fn new(cols: usize, rows: usize) -> Option<Self> {
        Some(Self {
            handler: PtyHandler::new(cols, rows)?,
        })
    }

    pub fn get_buffer(&self) -> Arc<Mutex<TerminalBuffer>> {
        self.handler.get_buffer()
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        self.handler.resize(cols, rows);
    }

    pub fn write_input(&mut self, data: &[u8]) {
        self.handler.write_input(data);
    }

    pub fn handle_key(&mut self, key: &str, mods: TerminalModifiers) {
        let bytes = match key {
            "Backspace" => b"\x7f",
            "Enter" => b"\r",
            "Tab" => b"\t",
            "Escape" => b"\x1b",
            "ArrowUp" => {
                if mods.ctrl {
                    b"\x1b[1;5A"
                } else if mods.alt {
                    b"\x1b[1;3A"
                } else {
                    b"\x1b[A"
                }
            }
            "ArrowDown" => {
                if mods.ctrl {
                    b"\x1b[1;5B"
                } else if mods.alt {
                    b"\x1b[1;3B"
                } else {
                    b"\x1b[B"
                }
            }
            "ArrowRight" => {
                if mods.ctrl {
                    b"\x1b[1;5C"
                } else if mods.alt {
                    b"\x1b[1;3C"
                } else {
                    b"\x1b[C"
                }
            }
            "ArrowLeft" => {
                if mods.ctrl {
                    b"\x1b[1;5D"
                } else if mods.alt {
                    b"\x1b[1;3D"
                } else {
                    b"\x1b[D"
                }
            }
            "Home" => b"\x1b[H",
            "End" => b"\x1b[F",
            "PageUp" => b"\x1b[5~",
            "PageDown" => b"\x1b[6~",
            "Insert" => b"\x1b[2~",
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
            _ => return,
        };
        self.handler.write_input(bytes);
    }

    pub fn handle_char(&mut self, c: char) {
        if c == '\u{7f}' {
            self.handler.write_input(b"\x7f");
        } else if c == '\r' {
            self.handler.write_input(b"\r");
        } else if c == '\t' {
            self.handler.write_input(b"\t");
        } else if c == '\x1b' {
            self.handler.write_input(b"\x1b");
        } else {
            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            self.handler.write_input(s.as_bytes());
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

    pub fn with_ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    pub fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }

    pub fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }
}