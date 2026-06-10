use egui::{Rect, Color32, Ui, RichText, Sense, Id, Pos2, ScrollArea, Vec2};
use crate::theme::Theme;
use crate::ui::terminal::{TerminalWidget, TerminalTab};

#[derive(Clone, Copy, Default)]
struct SelectionState {
    start: Option<(usize, usize)>,
    end: Option<(usize, usize)>,
}

pub fn draw_terminal(ui: &mut Ui, rect: Rect, terminals: &mut Vec<TerminalTab>, active_tab_idx: &mut usize, theme: &Theme) {
    let terminal_id = Id::new("terminal_panel");
    let focused = ui.memory(|m| m.has_focus(terminal_id));

    // Restore background color to original [6, 10, 8]
    let fill_color = Color32::from_rgba_unmultiplied(6, 10, 8, 255);

    // 1. Draw cyberpunk HUD border on rect
    let painter = ui.painter_at(rect);
    
    // Use almost the full rect, only shrinking by 4px to avoid touching neighbor borders
    let bounds = rect.shrink(4.0);
    let c = 28.0; // Chamfer size
    
    let step_h = 24.0;
    let step_x = bounds.max.x - 220.0;

    // Background Polygon with top-right, bottom-left, top-left chamfers, and a lifted stepped bottom-right edge
    let bg_points = vec![
        Pos2::new(bounds.min.x + 6.0, bounds.min.y),
        Pos2::new(bounds.max.x - c, bounds.min.y),
        Pos2::new(bounds.max.x, bounds.min.y + c),
        Pos2::new(bounds.max.x, bounds.max.y - step_h),
        Pos2::new(step_x, bounds.max.y - step_h),
        Pos2::new(step_x - step_h, bounds.max.y),
        Pos2::new(bounds.min.x + c, bounds.max.y),
        Pos2::new(bounds.min.x, bounds.max.y - c),
        Pos2::new(bounds.min.x, bounds.min.y + 6.0),
    ];
    painter.add(egui::Shape::convex_polygon(bg_points.clone(), fill_color, egui::Stroke::NONE));

    // Continuous stepped outer border (open at top-left corner for the tech node)
    let outer_stroke = egui::Stroke::new(1.0, theme.low());
    painter.add(egui::Shape::line(bg_points, outer_stroke));

    let thick_stroke = egui::Stroke::new(4.0, theme.accent);
    let glow_stroke = egui::Stroke::new(12.0, theme.accent.linear_multiply(0.15)); // Shading glow

    // Heavy Top-Right Accent Bracket
    let top_right_bracket = vec![
        Pos2::new(bounds.max.x - c - 30.0, bounds.min.y),
        Pos2::new(bounds.max.x - c, bounds.min.y),
        Pos2::new(bounds.max.x, bounds.min.y + c),
        Pos2::new(bounds.max.x, bounds.min.y + c + 30.0),
    ];
    painter.add(egui::Shape::line(top_right_bracket.clone(), glow_stroke));
    painter.add(egui::Shape::line(top_right_bracket, thick_stroke));

    // Segmented blocks nesting in the notch created by the step-up
    let h_height = 14.0; // Height of the decoration
    let mut current_x = step_x + 20.0;
    let chunk_widths = vec![100.0, 12.0, 30.0]; // Long block, gap, short block, gap, medium block
    
    for &w in &chunk_widths {
        if current_x + w + h_height > bounds.max.x - 10.0 {
            break; // Avoid overflowing past the right edge
        }
        
        let chunk_poly = vec![
            Pos2::new(current_x, bounds.max.y),
            Pos2::new(current_x + w, bounds.max.y),
            Pos2::new(current_x + w + h_height, bounds.max.y - h_height),
            Pos2::new(current_x + h_height, bounds.max.y - h_height),
        ];
        painter.add(egui::Shape::convex_polygon(chunk_poly, theme.accent, egui::Stroke::NONE));
        
        current_x += w + 8.0; // Gap between chunks
    }

    // Heavy Bottom-Left Chamfer Bracket
    let bottom_left_bracket = vec![
        Pos2::new(bounds.min.x + c + 30.0, bounds.max.y),
        Pos2::new(bounds.min.x + c, bounds.max.y),
        Pos2::new(bounds.min.x, bounds.max.y - c),
        Pos2::new(bounds.min.x, bounds.max.y - c - 30.0),
    ];
    painter.add(egui::Shape::line(bottom_left_bracket.clone(), glow_stroke));
    painter.add(egui::Shape::line(bottom_left_bracket, thick_stroke));

    // Top-Left Tech Node acting as the corner edge (High-tech reticle/crosshair design)
    let node_pos = Pos2::new(bounds.min.x + 6.0, bounds.min.y + 6.0);
    painter.circle_stroke(node_pos, 6.0, egui::Stroke::new(1.5, theme.accent)); // Outer edge ring
    painter.circle_filled(node_pos, 1.5, theme.high()); // Inner core dot
    
    // Crosshair ticks
    painter.add(egui::Shape::line(vec![
        Pos2::new(node_pos.x - 4.5, node_pos.y),
        Pos2::new(node_pos.x - 2.5, node_pos.y),
    ], egui::Stroke::new(1.0, theme.accent)));
    painter.add(egui::Shape::line(vec![
        Pos2::new(node_pos.x + 2.5, node_pos.y),
        Pos2::new(node_pos.x + 4.5, node_pos.y),
    ], egui::Stroke::new(1.0, theme.accent)));
    painter.add(egui::Shape::line(vec![
        node_pos - Vec2::new(0.0, 4.5),
        node_pos - Vec2::new(0.0, 2.5),
    ], egui::Stroke::new(1.0, theme.accent)));
    painter.add(egui::Shape::line(vec![
        node_pos + Vec2::new(0.0, 2.5),
        node_pos + Vec2::new(0.0, 4.5),
    ], egui::Stroke::new(1.0, theme.accent)));
    
    // Circuit routing line extending inwards from the node (like it was originally)
    let circ_stroke = egui::Stroke::new(1.5, theme.high());
    painter.add(egui::Shape::line(vec![
        Pos2::new(bounds.min.x + 12.0, bounds.min.y + 6.0), // Starts at right edge of node
        Pos2::new(bounds.min.x + 24.0, bounds.min.y + 6.0),
        Pos2::new(bounds.min.x + 30.0, bounds.min.y + 12.0),
        Pos2::new(bounds.min.x + 100.0, bounds.min.y + 12.0),
    ], circ_stroke));

    // Left Edge Data Track (kept inside bounds)
    for i in 0..8 {
        let y = bounds.min.y + 60.0 + (i as f32 * 8.0);
        painter.rect_filled(
            Rect::from_min_size(Pos2::new(bounds.min.x + 4.0, y), Vec2::new(4.0, 5.0)),
            0.0,
            theme.dimmed(),
        );
    }

    // Right Edge Crosshair (aligned correctly)
    let mid_y = bounds.min.y + bounds.height() * 0.6;
    painter.add(egui::Shape::line(vec![
        Pos2::new(bounds.max.x - 8.0, mid_y),
        Pos2::new(bounds.max.x + 2.0, mid_y),
    ], circ_stroke));
    painter.add(egui::Shape::line(vec![
        Pos2::new(bounds.max.x - 3.0, mid_y - 5.0),
        Pos2::new(bounds.max.x - 3.0, mid_y + 5.0),
    ], circ_stroke));

    // Inner structural outline (matching the step-up on the right side)
    let inner_bounds = bounds.shrink(6.0);
    let inner_stroke = egui::Stroke::new(1.0, theme.faint());
    let inner_step_x = step_x - 6.0;
    
    painter.add(egui::Shape::line(vec![
        Pos2::new(inner_bounds.min.x + 12.0, inner_bounds.min.y),
        Pos2::new(inner_bounds.max.x - (c - 6.0), inner_bounds.min.y),
        Pos2::new(inner_bounds.max.x, inner_bounds.min.y + (c - 6.0)),
        Pos2::new(inner_bounds.max.x, inner_bounds.max.y - step_h),
        Pos2::new(inner_step_x, inner_bounds.max.y - step_h),
        Pos2::new(inner_step_x - step_h, inner_bounds.max.y),
        Pos2::new(inner_bounds.min.x + (c - 6.0), inner_bounds.max.y),
        Pos2::new(inner_bounds.min.x, inner_bounds.max.y - (c - 6.0)),
        Pos2::new(inner_bounds.min.x, inner_bounds.min.y + 12.0),
    ], inner_stroke));

    // 2. Draw terminal contents safely inside the inner frame
    ui.allocate_ui_at_rect(inner_bounds.shrink2(Vec2::new(16.0, 16.0)), |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("⌘ TERMINALS:")
                        .monospace()
                        .size(11.0)
                        .color(theme.low()),
                );
                
                let mut tab_to_close = None;
                for (idx, tab) in terminals.iter().enumerate() {
                    ui.add_space(6.0);
                    let is_active = idx == *active_tab_idx;
                    
                    let tab_text = RichText::new(format!(" {}", tab.title))
                        .monospace()
                        .size(11.0)
                        .color(if is_active { theme.accent } else { theme.dimmed() });
                    let resp = ui.selectable_label(is_active, tab_text);
                    if resp.clicked() {
                        *active_tab_idx = idx;
                    }

                    if terminals.len() > 1 {
                        let close_text = RichText::new("x")
                            .monospace()
                            .size(11.0)
                            .color(if is_active { theme.accent } else { theme.dimmed() });
                        let close_resp = ui.selectable_label(false, close_text);
                        if close_resp.clicked() {
                            tab_to_close = Some(idx);
                        }
                    }
                }

                ui.add_space(10.0);
                let plus_resp = ui.selectable_label(
                    false,
                    RichText::new("[ + ]")
                        .monospace()
                        .size(11.0)
                        .color(theme.high()),
                );
                if plus_resp.clicked() {
                    println!("DEBUG: Plus button clicked! Spawning new terminal...");
                    match TerminalWidget::new(200, 50, ui.ctx().clone()) {
                        Some(new_term) => {
                            println!("DEBUG: Spawning new terminal succeeded!");
                            let tab_count = terminals.len();
                            terminals.push(TerminalTab {
                                title: format!("Term {}", tab_count + 1),
                                widget: new_term,
                            });
                            *active_tab_idx = tab_count;
                        }
                        None => {
                            println!("DEBUG: Spawning new terminal failed!");
                        }
                    }
                }

                if let Some(close_idx) = tab_to_close {
                    terminals.remove(close_idx);
                    if *active_tab_idx >= terminals.len() {
                        *active_tab_idx = terminals.len().saturating_sub(1);
                    }
                }
            });

            ui.separator();

            if terminals.is_empty() {
                return;
            }
            if *active_tab_idx >= terminals.len() {
                *active_tab_idx = terminals.len() - 1;
            }

            let term = &mut terminals[*active_tab_idx].widget;

            let cell_h = 17.0;
            let font_id = egui::FontId::new(13.0, egui::FontFamily::Monospace);
            let actual_char_w = ui.fonts(|f| f.glyph_width(&font_id, 'M'));
            let available_size = ui.available_size();
            let pad_x = 16.0; // padding + scrollbar
            let pad_y = 10.0; // padding
            let cols = (((available_size.x - pad_x) / actual_char_w).floor() as usize).max(40);
            let rows = (((available_size.y - pad_y) / cell_h).floor() as usize).max(10);

            term.resize(cols, rows);

            let buffer = term.get_buffer();
            let (total_rows, content_w, buf_width, history_len, cursor_row, cursor_col) = {
                if let Ok(buf_guard) = buffer.lock() {
                    let total_rows = buf_guard.history.len() + buf_guard.lines.len();
                    let content_w = buf_guard.width() as f32 * actual_char_w;
                    let (cursor_col, cursor_row) = buf_guard.cursor();
                    (total_rows, content_w, buf_guard.width(), buf_guard.history.len(), cursor_row, cursor_col)
                } else {
                    (0, 0.0, 80, 0, 0, 0)
                }
            };

            if total_rows > 0 {
                let font_size = 13.0;

                ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .auto_shrink([false; 2])
                    .show_rows(ui, cell_h, total_rows, |ui, row_range| {
                        let selection_id = Id::new("terminal_selection");
                        let mut selection: SelectionState = ui.memory(|m| m.data.get_temp(selection_id).unwrap_or_default());

                        let visible_rows = row_range.end - row_range.start;
                        let draw_h = visible_rows as f32 * cell_h;
                        let desired_size = Vec2::new(content_w.min(ui.available_width()), draw_h);
                        let (response, painter) = ui.allocate_painter(desired_size, Sense::click_and_drag());

                        let origin = response.rect.min;
                        let clip_rect = response.rect;

                        // Handle selection input
                        if response.drag_started() {
                            if let Some(pos) = ui.input(|i| i.pointer.press_origin()) {
                                let rel_x = (pos.x - origin.x).max(0.0);
                                let rel_y = (pos.y - origin.y).max(0.0);
                                let col = ((rel_x / actual_char_w) as usize).min(buf_width - 1);
                                let row = row_range.start + ((rel_y / cell_h) as usize).min(visible_rows - 1);
                                selection.start = Some((row, col));
                                selection.end = Some((row, col));
                            }
                        } else if response.dragged() {
                            if let Some(pos) = ui.input(|i| i.pointer.latest_pos()) {
                                let rel_x = (pos.x - origin.x).max(0.0);
                                let rel_y = (pos.y - origin.y).max(0.0);
                                let col = ((rel_x / actual_char_w) as usize).min(buf_width - 1);
                                let row = row_range.start + ((rel_y / cell_h) as usize).min(visible_rows - 1);
                                selection.end = Some((row, col));
                            }
                        } else if response.clicked() || ui.input(|i| i.pointer.any_pressed()) {
                            selection.start = None;
                            selection.end = None;
                        }

                        painter.rect_filled(
                            Rect::from_min_size(origin, desired_size),
                            0.0,
                            Color32::from_rgba_unmultiplied(6, 10, 8, 255),
                        );

                        // Lock briefly to clone only the visible rows
                        let visible_cells = {
                            let mut cells = Vec::with_capacity(visible_rows);
                            if let Ok(buf_guard) = buffer.lock() {
                                for virtual_row in row_range.clone() {
                                    let line_cells = if virtual_row < history_len {
                                        buf_guard.history[virtual_row].clone()
                                    } else {
                                        buf_guard.lines[virtual_row - history_len].clone()
                                    };
                                    cells.push(line_cells);
                                }
                            } else {
                                for _ in row_range.clone() {
                                    cells.push(vec![crate::ui::terminal::Cell::default(); buf_width]);
                                }
                            }
                            cells
                        };

                        for virtual_row in row_range.clone() {
                            let local_row = virtual_row - row_range.start;
                            let y = origin.y + local_row as f32 * cell_h;

                            let line_cells = &visible_cells[local_row];

                            // Render selection highlight background
                            for col_idx in 0..buf_width {
                                if is_cell_selected(virtual_row, col_idx, selection.start, selection.end) {
                                    let x = origin.x + col_idx as f32 * actual_char_w;
                                    painter.rect_filled(
                                        Rect::from_min_size(
                                            Pos2::new(x, y),
                                            Vec2::new(actual_char_w, cell_h),
                                        ),
                                        0.0,
                                        Color32::from_rgba_unmultiplied(100, 149, 237, 80),
                                    );
                                }
                            }

                            let mut line_end = 0;
                            for col_idx in (0..buf_width).rev() {
                                if line_cells[col_idx].c != ' ' {
                                    line_end = col_idx + 1;
                                    break;
                                }
                            }

                            // Detect right-prompt situation: cursor row, cursor far right, text at left
                            let cursor_virtual_row = history_len + cursor_row;
                            let is_cursor_row = cursor_virtual_row >= row_range.start && cursor_virtual_row < row_range.end;
                            let local_cursor_row = if is_cursor_row { cursor_virtual_row - row_range.start } else { 0 };
                            let is_right_prompt = is_cursor_row && cursor_col > line_end + 10 && line_end > 0 && line_end < buf_width / 2;

                            // For right prompt, render text right-aligned
                            let render_start_col = if is_right_prompt { buf_width - line_end } else { 0 };
                            let effective_cursor_col = if is_right_prompt { buf_width } else { cursor_col.min(line_end) };

                            if line_end > 0 {
                                let font_id = egui::FontId::new(font_size, egui::FontFamily::Monospace);

                                for col_idx in 0..line_end {
                                    let cell = line_cells[col_idx];
                                    let fg = if cell.bold {
                                        Color32::from_rgb(
                                            (cell.fg[0] as u32 + 40).min(255) as u8,
                                            (cell.fg[1] as u32 + 40).min(255) as u8,
                                            (cell.fg[2] as u32 + 40).min(255) as u8,
                                        )
                                    } else {
                                        Color32::from_rgb(cell.fg[0], cell.fg[1], cell.fg[2])
                                    };

                                    let render_col = render_start_col + col_idx;
                                    let cx = origin.x + render_col as f32 * actual_char_w;
                                    painter.text(
                                        Pos2::new(cx, y),
                                        egui::Align2::LEFT_TOP,
                                        cell.c.to_string(),
                                        font_id.clone(),
                                        fg,
                                    );
                                }
                            }

                            // Draw cursor
                            if is_cursor_row {
                                let cursor_x = origin.x + effective_cursor_col as f32 * actual_char_w;
                                let cursor_y = origin.y + local_cursor_row as f32 * cell_h;

                                if cursor_x >= clip_rect.min.x && cursor_x <= clip_rect.max.x
                                    && cursor_y >= clip_rect.min.y && cursor_y <= clip_rect.max.y
                                {
                                    painter.rect_filled(
                                        Rect::from_min_size(
                                            Pos2::new(cursor_x, cursor_y),
                                            Vec2::new(actual_char_w * 0.55, cell_h),
                                        ),
                                        0.0,
                                        Color32::from_rgb(0, 200, 160),
                                );
                            }
                        }
                    }

                        ui.memory_mut(|m| m.data.insert_temp(selection_id, selection));
                    });
            }
        });

        if ui.input(|i| i.pointer.any_click()) && ui.rect_contains_pointer(rect) {
            ui.memory_mut(|m| m.request_focus(terminal_id));
        }

        if terminals.is_empty() {
            return;
        }
        if *active_tab_idx >= terminals.len() {
            *active_tab_idx = terminals.len() - 1;
        }
        let term = &mut terminals[*active_tab_idx].widget;

        if focused {
            let mut needs_repaint = false;
            let selection_id = Id::new("terminal_selection");
            let selection: SelectionState = ui.memory(|m| m.data.get_temp(selection_id).unwrap_or_default());
            let has_selection = selection.start.is_some() && selection.end.is_some();

            // Process events sequentially (Copy, Paste, Keyboard Input, Control keys)
            let events = ui.input(|i| i.events.clone());
            let mut has_keyboard_input = false;
            let mut trigger_copy = false;
            let mut ctrl_c_pressed = false;

            for event in &events {
                match event {
                    egui::Event::Key { key, pressed: true, modifiers, .. } => {
                        has_keyboard_input = true;
                        if modifiers.ctrl && *key == egui::Key::C {
                            ctrl_c_pressed = true;
                        } else if modifiers.ctrl && *key == egui::Key::V && !modifiers.shift {
                            if let Some(text) = crate::ui::clipboard::paste_from_clipboard() {
                                for c in text.chars() {
                                    term.handle_char(c);
                                }
                                needs_repaint = true;
                                ui.memory_mut(|m| m.data.insert_temp(selection_id, SelectionState::default()));
                            }
                        } else {
                            if handle_keyboard_event(term, *key, *modifiers) {
                                needs_repaint = true;
                            }
                        }
                    }
                    egui::Event::Text(text) => {
                        has_keyboard_input = true;
                        for c in text.chars() {
                            if c.is_control() {
                                continue;
                            }
                            term.handle_char(c);
                            needs_repaint = true;
                        }
                    }
                    egui::Event::Paste(text) => {
                        for c in text.chars() {
                            term.handle_char(c);
                        }
                        needs_repaint = true;
                        ui.memory_mut(|m| m.data.insert_temp(selection_id, SelectionState::default()));
                    }
                    egui::Event::Copy => {
                        ctrl_c_pressed = true;
                    }
                    _ => {}
                }
            }

            if ctrl_c_pressed {
                if has_selection {
                    trigger_copy = true;
                } else {
                    let _ = handle_keyboard_event(term, egui::Key::C, egui::Modifiers { ctrl: true, ..Default::default() });
                    needs_repaint = true;
                }
            }

            if trigger_copy {
                let buffer = term.get_buffer();
                if let Ok(buf) = buffer.lock() {
                    let text = if let (Some(s), Some(e)) = (selection.start, selection.end) {
                        get_selected_text(&buf, s, e)
                    } else {
                        // Fallback: Copy the entire buffer screen
                        let mut t = String::new();
                        for row in 0..(buf.history.len() + buf.lines.len()) {
                            let line_cells = if row < buf.history.len() {
                                &buf.history[row]
                            } else {
                                &buf.lines[row - buf.history.len()]
                            };
                            let line: String = line_cells.iter().map(|c| c.c).collect();
                            t.push_str(line.trim_end());
                            t.push('\n');
                        }
                        t
                    };
                    ui.ctx().copy_text(text.clone());
                    crate::ui::clipboard::copy_to_clipboard(&text);
                }
                needs_repaint = true;
            }

            // Clear selection if keyboard input happened and we didn't copy
            if has_keyboard_input && !trigger_copy {
                ui.memory_mut(|m| m.data.insert_temp(selection_id, SelectionState::default()));
            }

            if needs_repaint {
                ui.ctx().request_repaint();
            }
        }
    });
}

fn is_cell_selected(
    row: usize,
    col: usize,
    start: Option<(usize, usize)>,
    end: Option<(usize, usize)>,
) -> bool {
    if let (Some(s), Some(e)) = (start, end) {
        let (r1, c1) = if s <= e { s } else { e };
        let (r2, c2) = if s <= e { e } else { s };

        if row < r1 || row > r2 {
            return false;
        }
        if r1 == r2 {
            row == r1 && col >= c1 && col <= c2
        } else if row == r1 {
            col >= c1
        } else if row == r2 {
            col <= c2
        } else {
            true
        }
    } else {
        false
    }
}

fn get_selected_text(
    buf: &crate::ui::terminal::TerminalBuffer,
    start: (usize, usize),
    end: (usize, usize),
) -> String {
    let mut text = String::new();
    let (s, e) = if start <= end { (start, end) } else { (end, start) };
    let (r1, c1) = s;
    let (r2, c2) = e;

    for row in r1..=r2 {
        let line_cells = if row < buf.history.len() {
            &buf.history[row]
        } else {
            &buf.lines[row - buf.history.len()]
        };
        let col_start = if row == r1 { c1 } else { 0 };
        let col_end = if row == r2 { c2.min(line_cells.len() - 1) } else { line_cells.len() - 1 };

        let mut line = String::new();
        for col in col_start..=col_end {
            line.push(line_cells[col].c);
        }

        if row == r2 {
            text.push_str(line.trim_end());
        } else {
            text.push_str(line.trim_end());
            text.push('\n');
        }
    }
    text
}

fn handle_keyboard_event(term: &mut crate::ui::terminal::TerminalWidget, key: egui::Key, modifiers: egui::Modifiers) -> bool {
    let mut bytes = Vec::new();

    let mut modifier_code = 1;
    if modifiers.shift { modifier_code += 1; }
    if modifiers.alt { modifier_code += 2; }
    if modifiers.ctrl { modifier_code += 4; }

    let letter_char = match key {
        egui::Key::A => Some('a'),
        egui::Key::B => Some('b'),
        egui::Key::C => Some('c'),
        egui::Key::D => Some('d'),
        egui::Key::E => Some('e'),
        egui::Key::F => Some('f'),
        egui::Key::G => Some('g'),
        egui::Key::H => Some('h'),
        egui::Key::I => Some('i'),
        egui::Key::J => Some('j'),
        egui::Key::K => Some('k'),
        egui::Key::L => Some('l'),
        egui::Key::M => Some('m'),
        egui::Key::N => Some('n'),
        egui::Key::O => Some('o'),
        egui::Key::P => Some('p'),
        egui::Key::Q => Some('q'),
        egui::Key::R => Some('r'),
        egui::Key::S => Some('s'),
        egui::Key::T => Some('t'),
        egui::Key::U => Some('u'),
        egui::Key::V => Some('v'),
        egui::Key::W => Some('w'),
        egui::Key::X => Some('x'),
        egui::Key::Y => Some('y'),
        egui::Key::Z => Some('z'),
        _ => None,
    };

    if modifiers.ctrl && !modifiers.alt {
        if let Some(c) = letter_char {
            bytes.push(c as u8 - b'a' + 1);
        } else {
            match key {
                egui::Key::ArrowUp => bytes.extend_from_slice(b"\x1b[1;5A"),
                egui::Key::ArrowDown => bytes.extend_from_slice(b"\x1b[1;5B"),
                egui::Key::ArrowRight => bytes.extend_from_slice(b"\x1b[1;5C"),
                egui::Key::ArrowLeft => bytes.extend_from_slice(b"\x1b[1;5D"),
                egui::Key::Home => bytes.extend_from_slice(b"\x1b[1;5H"),
                egui::Key::End => bytes.extend_from_slice(b"\x1b[1;5F"),
                egui::Key::Delete => bytes.extend_from_slice(b"\x1b[3;5~"),
                egui::Key::PageUp => bytes.extend_from_slice(b"\x1b[5;5~"),
                egui::Key::PageDown => bytes.extend_from_slice(b"\x1b[6;5~"),
                egui::Key::Backspace => bytes.push(0x08),
                egui::Key::Tab => bytes.extend_from_slice(b"\x1b[Z"),
                _ => return false,
            }
        }
    } else if modifiers.alt && !modifiers.ctrl {
        if let Some(c) = letter_char {
            bytes.push(0x1b);
            bytes.push(c as u8);
        } else {
            match key {
                egui::Key::ArrowUp => bytes.extend_from_slice(b"\x1b[1;3A"),
                egui::Key::ArrowDown => bytes.extend_from_slice(b"\x1b[1;3B"),
                egui::Key::ArrowRight => bytes.extend_from_slice(b"\x1b[1;3C"),
                egui::Key::ArrowLeft => bytes.extend_from_slice(b"\x1b[1;3D"),
                egui::Key::Home => bytes.extend_from_slice(b"\x1b[1;3H"),
                egui::Key::End => bytes.extend_from_slice(b"\x1b[1;3F"),
                egui::Key::Delete => bytes.extend_from_slice(b"\x1b[3;3~"),
                egui::Key::PageUp => bytes.extend_from_slice(b"\x1b[5;3~"),
                egui::Key::PageDown => bytes.extend_from_slice(b"\x1b[6;3~"),
                egui::Key::Escape => bytes.extend_from_slice(b"\x1b\x1b"),
                egui::Key::Backspace => bytes.extend_from_slice(b"\x1b\x7f"),
                egui::Key::Enter => bytes.extend_from_slice(b"\x1b\r"),
                _ => return false,
            }
        }
    } else if modifiers.ctrl && modifiers.alt {
        if let Some(c) = letter_char {
            bytes.push(0x1b);
            bytes.push(c as u8 - b'a' + 1);
        } else {
            match key {
                egui::Key::ArrowUp => bytes.extend_from_slice(b"\x1b[1;7A"),
                egui::Key::ArrowDown => bytes.extend_from_slice(b"\x1b[1;7B"),
                egui::Key::ArrowRight => bytes.extend_from_slice(b"\x1b[1;7C"),
                egui::Key::ArrowLeft => bytes.extend_from_slice(b"\x1b[1;7D"),
                _ => return false,
            }
        }
    } else {
        match key {
            egui::Key::ArrowUp => {
                if modifier_code > 1 {
                    bytes.extend(format!("\x1b[1;{}A", modifier_code).into_bytes());
                } else {
                    bytes.extend_from_slice(b"\x1b[A");
                }
            }
            egui::Key::ArrowDown => {
                if modifier_code > 1 {
                    bytes.extend(format!("\x1b[1;{}B", modifier_code).into_bytes());
                } else {
                    bytes.extend_from_slice(b"\x1b[B");
                }
            }
            egui::Key::ArrowRight => {
                if modifier_code > 1 {
                    bytes.extend(format!("\x1b[1;{}C", modifier_code).into_bytes());
                } else {
                    bytes.extend_from_slice(b"\x1b[C");
                }
            }
            egui::Key::ArrowLeft => {
                if modifier_code > 1 {
                    bytes.extend(format!("\x1b[1;{}D", modifier_code).into_bytes());
                } else {
                    bytes.extend_from_slice(b"\x1b[D");
                }
            }
            egui::Key::Home => {
                if modifier_code > 1 {
                    bytes.extend(format!("\x1b[1;{}H", modifier_code).into_bytes());
                } else {
                    bytes.extend_from_slice(b"\x1b[H");
                }
            }
            egui::Key::End => {
                if modifier_code > 1 {
                    bytes.extend(format!("\x1b[1;{}F", modifier_code).into_bytes());
                } else {
                    bytes.extend_from_slice(b"\x1b[F");
                }
            }
            egui::Key::PageUp => {
                if modifier_code > 1 {
                    bytes.extend(format!("\x1b[5;{}~", modifier_code).into_bytes());
                } else {
                    bytes.extend_from_slice(b"\x1b[5~");
                }
            }
            egui::Key::PageDown => {
                if modifier_code > 1 {
                    bytes.extend(format!("\x1b[6;{}~", modifier_code).into_bytes());
                } else {
                    bytes.extend_from_slice(b"\x1b[6~");
                }
            }
            egui::Key::Insert => {
                if modifier_code > 1 {
                    bytes.extend(format!("\x1b[2;{}~", modifier_code).into_bytes());
                } else {
                    bytes.extend_from_slice(b"\x1b[2~");
                }
            }
            egui::Key::Delete => {
                if modifier_code > 1 {
                    bytes.extend(format!("\x1b[3;{}~", modifier_code).into_bytes());
                } else {
                    bytes.extend_from_slice(b"\x1b[3~");
                }
            }
            egui::Key::Escape => bytes.push(0x1b),
            egui::Key::Tab => {
                if modifiers.shift {
                    bytes.extend_from_slice(b"\x1b[Z");
                } else {
                    bytes.push(0x09);
                }
            }
            egui::Key::Enter => bytes.push(b'\r'),
            egui::Key::Backspace => bytes.push(0x7f),
            
            egui::Key::F1 => {
                if modifier_code > 1 {
                    bytes.extend(format!("\x1b[1;{}P", modifier_code).into_bytes());
                } else {
                    bytes.extend_from_slice(b"\x1bOP");
                }
            }
            egui::Key::F2 => {
                if modifier_code > 1 {
                    bytes.extend(format!("\x1b[1;{}Q", modifier_code).into_bytes());
                } else {
                    bytes.extend_from_slice(b"\x1bOQ");
                }
            }
            egui::Key::F3 => {
                if modifier_code > 1 {
                    bytes.extend(format!("\x1b[1;{}R", modifier_code).into_bytes());
                } else {
                    bytes.extend_from_slice(b"\x1bOR");
                }
            }
            egui::Key::F4 => {
                if modifier_code > 1 {
                    bytes.extend(format!("\x1b[1;{}S", modifier_code).into_bytes());
                } else {
                    bytes.extend_from_slice(b"\x1bOS");
                }
            }
            egui::Key::F5 => {
                if modifier_code > 1 {
                    bytes.extend(format!("\x1b[15;{}~", modifier_code).into_bytes());
                } else {
                    bytes.extend_from_slice(b"\x1b[15~");
                }
            }
            egui::Key::F6 => {
                if modifier_code > 1 {
                    bytes.extend(format!("\x1b[17;{}~", modifier_code).into_bytes());
                } else {
                    bytes.extend_from_slice(b"\x1b[17~");
                }
            }
            egui::Key::F7 => {
                if modifier_code > 1 {
                    bytes.extend(format!("\x1b[18;{}~", modifier_code).into_bytes());
                } else {
                    bytes.extend_from_slice(b"\x1b[18~");
                }
            }
            egui::Key::F8 => {
                if modifier_code > 1 {
                    bytes.extend(format!("\x1b[19;{}~", modifier_code).into_bytes());
                } else {
                    bytes.extend_from_slice(b"\x1b[19~");
                }
            }
            egui::Key::F9 => {
                if modifier_code > 1 {
                    bytes.extend(format!("\x1b[20;{}~", modifier_code).into_bytes());
                } else {
                    bytes.extend_from_slice(b"\x1b[20~");
                }
            }
            egui::Key::F10 => {
                if modifier_code > 1 {
                    bytes.extend(format!("\x1b[21;{}~", modifier_code).into_bytes());
                } else {
                    bytes.extend_from_slice(b"\x1b[21~");
                }
            }
            egui::Key::F11 => {
                if modifier_code > 1 {
                    bytes.extend(format!("\x1b[23;{}~", modifier_code).into_bytes());
                } else {
                    bytes.extend_from_slice(b"\x1b[23~");
                }
            }
            egui::Key::F12 => {
                if modifier_code > 1 {
                    bytes.extend(format!("\x1b[24;{}~", modifier_code).into_bytes());
                } else {
                    bytes.extend_from_slice(b"\x1b[24~");
                }
            }
            
            _ => return false,
        }
    }

    if !bytes.is_empty() {
        term.write_input(&bytes);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_cell_selected() {
        let start = Some((1, 2));
        let end = Some((3, 4));

        assert!(!is_cell_selected(0, 0, start, end));
        assert!(!is_cell_selected(0, 5, start, end));

        assert!(!is_cell_selected(1, 1, start, end));
        assert!(is_cell_selected(1, 2, start, end));
        assert!(is_cell_selected(1, 9, start, end));

        assert!(is_cell_selected(2, 0, start, end));
        assert!(is_cell_selected(2, 9, start, end));

        assert!(is_cell_selected(3, 0, start, end));
        assert!(is_cell_selected(3, 4, start, end));
        assert!(!is_cell_selected(3, 5, start, end));

        assert!(!is_cell_selected(4, 0, start, end));
    }
}
