use std::collections::VecDeque;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

use portable_pty::{Child, CommandBuilder, NativePtySystem, PtySize, PtySystem, MasterPty};

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
            fg: [248, 248, 242],
            bg: [6, 10, 8],
            bold: false,
        }
    }
}

#[derive(Clone)]
pub struct TerminalBuffer {
    pub history: VecDeque<Vec<Cell>>,
    pub lines: VecDeque<Vec<Cell>>,
    pub width: usize,
    pub height: usize,
    cursor_col: usize,
    cursor_row: usize,
    saved_cursor_col: usize,
    saved_cursor_row: usize,
    fg: [u8; 3],
    bg: [u8; 3],
    bold: bool,
}

impl TerminalBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            history: VecDeque::new(),
            lines: VecDeque::from(vec![vec![Cell::default(); width]; height]),
            width,
            height,
            cursor_col: 0,
            cursor_row: 0,
            saved_cursor_col: 0,
            saved_cursor_row: 0,
            fg: [248, 248, 242],
            bg: [6, 10, 8],
            bold: false,
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        if self.width == width && self.height == height {
            return;
        }

        // Adjust all history lines to the new width
        for line in &mut self.history {
            line.resize(width, Cell::default());
        }

        // Adjust active lines to the new width
        for line in &mut self.lines {
            line.resize(width, Cell::default());
        }

        // Adjust height of active screen
        if self.lines.len() < height {
            while self.lines.len() < height {
                self.lines.push_back(vec![Cell::default(); width]);
            }
        } else if self.lines.len() > height {
            while self.lines.len() > height {
                if let Some(line) = self.lines.pop_front() {
                    self.push_history(line);
                }
            }
        }

        self.width = width;
        self.height = height;

        self.cursor_col = self.cursor_col.min(width - 1);
        self.cursor_row = self.cursor_row.min(height - 1);
        self.saved_cursor_col = self.saved_cursor_col.min(width - 1);
        self.saved_cursor_row = self.saved_cursor_row.min(height - 1);
    }

    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }
    pub fn cursor(&self) -> (usize, usize) { (self.cursor_col, self.cursor_row) }

    fn push_history(&mut self, line: Vec<Cell>) {
        self.history.push_back(line);
        if self.history.len() > 2000 {
            self.history.pop_front();
        }
    }

    fn put_char(&mut self, c: char) {
        if self.cursor_col >= self.width {
            self.cursor_col = 0;
            self.cursor_row += 1;
            if self.cursor_row >= self.height {
                if let Some(discarded) = self.lines.pop_front() {
                    self.push_history(discarded);
                }
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
            if let Some(discarded) = self.lines.pop_front() {
                self.push_history(discarded);
            }
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

    fn save_cursor(&mut self) {
        self.saved_cursor_col = self.cursor_col;
        self.saved_cursor_row = self.cursor_row;
    }

    fn restore_cursor(&mut self) {
        self.cursor_col = self.saved_cursor_col.min(self.width - 1);
        self.cursor_row = self.saved_cursor_row.min(self.height - 1);
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
        if let Some(discarded) = self.lines.pop_front() {
            self.push_history(discarded);
        }
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

pub struct TerminalTab {
    pub title: String,
    pub widget: TerminalWidget,
}

pub struct TerminalWidget {
    buffer: Arc<Mutex<TerminalBuffer>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    _child: Box<dyn Child + Send>,
    _reader: thread::JoinHandle<()>,
    master: Box<dyn MasterPty + Send>,
    request_cols: usize,
    request_rows: usize,
    committed_cols: usize,
    committed_rows: usize,
    resize_stable: u8,
}

impl TerminalWidget {
    pub fn new(cols: usize, rows: usize, ctx: egui::Context) -> Option<Self> {
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
        let writer_clone = Arc::clone(&writer);

        let _reader = thread::spawn(move || {
            let mut buf = [0u8; 4096];
            let mut parser = AnsiParser::new();
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if let Ok(mut b) = buffer_clone.lock() {
                            parser.feed_bytes(&mut b, &buf[..n], &writer_clone);
                        }
                        ctx.request_repaint();
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
            master: pair.master,
            request_cols: cols,
            request_rows: rows,
            committed_cols: cols,
            committed_rows: rows,
            resize_stable: 3,
        })
    }

    pub fn get_buffer(&self) -> &Arc<Mutex<TerminalBuffer>> {
        &self.buffer
    }

    pub fn process_id(&self) -> Option<u32> {
        self._child.process_id()
    }

    pub fn is_alive(&mut self) -> bool {
        match self._child.try_wait() {
            Ok(None) => true,
            _ => false,
        }
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        // Track request stability across frames to avoid transient sizes
        // (e.g. during minimize/restore) from shifting buffer content.
        if cols == self.request_cols && rows == self.request_rows {
            self.resize_stable = self.resize_stable.saturating_add(1);
        } else {
            self.request_cols = cols;
            self.request_rows = rows;
            self.resize_stable = 0;
        }

        // Commit only when the request has been stable for 3+ frames.
        if self.resize_stable >= 3 {
            let final_cols = cols;
            let final_rows = if rows.abs_diff(self.committed_rows) <= 1 && cols == self.committed_cols {
                // 1-row tolerance: WM margins can shift available height by < 1 row
                // between maximized/restored. Skip the resize so content doesn't jitter.
                self.committed_rows
            } else {
                rows
            };

            if final_cols != self.committed_cols || final_rows != self.committed_rows {
                self.committed_cols = final_cols;
                self.committed_rows = final_rows;
                if let Ok(mut buf) = self.buffer.lock() {
                    buf.resize(final_cols, final_rows);
                }
                let _ = self.master.resize(PtySize {
                    rows: final_rows as u16,
                    cols: final_cols as u16,
                    pixel_width: 0,
                    pixel_height: 0,
                });
            }
        }
    }

    pub fn write_input(&mut self, data: &[u8]) {
        if let Ok(mut w) = self.writer.lock() {
            let _ = w.write_all(data);
            let _ = w.flush();
        }
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
    leftover: Vec<u8>,
}

#[derive(Clone, Copy, PartialEq)]
enum ParserState {
    Normal,
    Escape,
    CsiEntry,
    OscEntry,
    DcsEntry,
}

impl AnsiParser {
    fn new() -> Self {
        Self {
            state: ParserState::Normal,
            params: Vec::new(),
            current_param: String::new(),
            has_intermediate: false,
            leftover: Vec::new(),
        }
    }

    fn feed_bytes(&mut self, buf: &mut TerminalBuffer, data: &[u8], writer: &Arc<Mutex<Box<dyn Write + Send>>>) {
        self.leftover.extend_from_slice(data);

        let mut idx = 0;
        while idx < self.leftover.len() {
            let res = match std::str::from_utf8(&self.leftover[idx..]) {
                Ok(s) => Ok(s.to_string()),
                Err(e) => {
                    let valid_len = e.valid_up_to();
                    let valid_str = if valid_len > 0 {
                        Some(std::str::from_utf8(&self.leftover[idx..idx + valid_len]).unwrap().to_string())
                    } else {
                        None
                    };
                    Err((valid_str, e.error_len()))
                }
            };

            match res {
                Ok(s_str) => {
                    self.feed(buf, &s_str, writer);
                    self.leftover.clear();
                    return;
                }
                Err((valid_str, error_len)) => {
                    if let Some(s_str) = valid_str {
                        let len = s_str.len();
                        self.feed(buf, &s_str, writer);
                        idx += len;
                    }
                    if let Some(err_len) = error_len {
                        idx += err_len;
                    } else {
                        // Incomplete UTF-8 sequence at the end of buffer, keep for next read.
                        self.leftover.drain(..idx);
                        return;
                    }
                }
            }
        }
        self.leftover.clear();
    }

    fn feed(&mut self, buf: &mut TerminalBuffer, data: &str, writer: &Arc<Mutex<Box<dyn Write + Send>>>) {
        for c in data.chars() {
            let val = c as u32;
            match self.state {
                ParserState::Normal => {
                    match c {
                        '\x1b' => self.state = ParserState::Escape,
                        '\n' => buf.newline(),
                        '\r' => buf.carriage_return(),
                        '\x08' | '\x7f' => buf.backspace(),
                        '\x0c' => {
                            buf.erase_screen();
                        }
                        _ if val >= 0x20 => buf.put_char(c),
                        _ => {}
                    }
                }
                ParserState::Escape => {
                    match c {
                        '[' => {
                            self.state = ParserState::CsiEntry;
                            self.params.clear();
                            self.current_param.clear();
                            self.has_intermediate = false;
                        }
                        ']' => {
                            self.state = ParserState::OscEntry;
                            self.current_param.clear();
                        }
                        'P' | '_' | '^' | 'X' => {
                            // Device Control String, Application Program Command, Privacy Message, Start of String
                            self.state = ParserState::DcsEntry;
                        }
                        '\\' => {
                            // String Terminator (ST)
                            self.state = ParserState::Normal;
                        }
                        's' => {
                            // DECSC - Save cursor position
                            buf.save_cursor();
                            self.state = ParserState::Normal;
                        }
                        'u' => {
                            // DECRC - Restore cursor position
                            buf.restore_cursor();
                            self.state = ParserState::Normal;
                        }
                        '\r' => {
                            self.state = ParserState::Normal;
                            buf.carriage_return();
                        }
                        '\n' => {
                            self.state = ParserState::Normal;
                            buf.newline();
                        }
                        _ => {
                            self.state = ParserState::Normal;
                        }
                    }
                }
                ParserState::CsiEntry => {
                    if c >= '0' && c <= '9' {
                        self.current_param.push(c);
                    } else if c == ';' {
                        self.push_param();
                    } else if val >= 0x20 && val <= 0x2f {
                        self.has_intermediate = true;
                    } else if c == '?' || c == '>' || c == '!' || c == '$' {
                        // Private mode prefix — skip it but mark as intermediate
                        self.has_intermediate = true;
                    } else if c == '\r' || c == '\n' || c == '\x07' {
                        // Abort CSI sequence on control characters, process them normally
                        self.state = ParserState::Normal;
                        match c {
                            '\r' => buf.carriage_return(),
                            '\n' => buf.newline(),
                            _ => {}
                        }
                    } else {
                        self.push_param();
                        self.execute_csi(buf, c as u8, writer);
                        self.state = ParserState::Normal;
                    }
                }
                ParserState::OscEntry => {
                    if c == '\x07' {
                        self.state = ParserState::Normal;
                    } else if c == '\x1b' {
                        self.state = ParserState::Escape;
                    } else {
                        self.current_param.push(c);
                    }
                }
                ParserState::DcsEntry => {
                    if c == '\x07' {
                        self.state = ParserState::Normal;
                    } else if c == '\x1b' {
                        self.state = ParserState::Escape;
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

    fn execute_csi(&mut self, buf: &mut TerminalBuffer, final_byte: u8, writer: &Arc<Mutex<Box<dyn Write + Send>>>) {
        if self.has_intermediate {
            if final_byte == b'c' {
                // Secondary DA reply
                if let Ok(mut w) = writer.lock() {
                    let _ = w.write_all(b"\x1b[>1;95;0c");
                    let _ = w.flush();
                }
            }
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
                buf.cursor_row = (row.saturating_sub(1)).min(buf.height - 1);
                buf.cursor_col = (col.saturating_sub(1)).min(buf.width - 1);
            }
            b'c' => {
                // Primary Device Attributes reply: VT100 with Advanced Video Option
                if let Ok(mut w) = writer.lock() {
                    let _ = w.write_all(b"\x1b[?1;2c");
                    let _ = w.flush();
                }
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
                // Device status report — respond with cursor position
                let param = self.params.first().copied().unwrap_or(0);
                if param == 6 {
                    // DSR: report cursor position as \x1b[{row};{col}R (1-based)
                    let row = buf.cursor_row + 1;
                    let col = buf.cursor_col + 1;
                    let resp = format!("\x1b[{};{}R", row, col);
                    if let Ok(mut w) = writer.lock() {
                        let _ = w.write_all(resp.as_bytes());
                        let _ = w.flush();
                    }
                }
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
                39 => buf.set_fg(248, 248, 242),
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
                49 => buf.set_bg(6, 10, 8),
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
        0 => [6, 10, 8],          // Dark background / black
        1 => [255, 85, 85],       // Dracula red
        2 => [80, 250, 123],      // Dracula green
        3 => [241, 250, 140],     // Dracula yellow
        4 => [0, 171, 255],       // Tron blue (sensible, vibrant electric blue!)
        5 => [255, 121, 198],     // Dracula magenta
        6 => [139, 233, 253],     // Dracula cyan
        7 => [248, 248, 242],     // Dracula white/gray
        _ => [248, 248, 242],
    }
}

fn bright_color(n: u8) -> [u8; 3] {
    match n {
        0 => [98, 114, 164],      // Dracula comment gray
        1 => [255, 110, 110],     // Bright red
        2 => [90, 255, 135],      // Bright green
        3 => [255, 255, 150],     // Bright yellow
        4 => [51, 190, 255],      // Bright electric blue
        5 => [255, 140, 210],     // Bright magenta
        6 => [160, 240, 255],     // Bright cyan
        7 => [255, 255, 255],     // Pure white
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
        
        // The discarded first line ('a') should now be in the history buffer
        assert_eq!(buf.history.len(), 1);
        assert_eq!(buf.history[0][0].c, 'a');
    }
}
