use egui::{Rect, Frame, Color32, Ui, RichText, Sense, Id, FontId, Align2, Pos2, ScrollArea, Vec2};
use crate::theme::Theme;
use crate::ui::terminal::TerminalWidget;

#[derive(Clone, Copy, Default)]
struct SelectionState {
    start: Option<(usize, usize)>,
    end: Option<(usize, usize)>,
}

pub fn draw_terminal(ui: &mut Ui, rect: Rect, term: &mut TerminalWidget, theme: &Theme) {
    let terminal_id = Id::new("terminal_panel");
    let focused = ui.memory(|m| m.has_focus(terminal_id));

    ui.allocate_ui_at_rect(rect, |ui| {
        Frame::none()
            .fill(Color32::from_rgba_unmultiplied(6, 10, 8, 255))
            .stroke(egui::Stroke::new(1.0, theme.mid()))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("⌘ TERMINAL")
                                .monospace()
                                .size(11.0)
                                .color(theme.low()),
                        );
                        ui.separator();
                        ui.label(
                            RichText::new("fish")
                                .monospace()
                                .size(10.0)
                                .color(theme.dimmed()),
                        );
                    });

                    ui.separator();

                    let buffer = term.get_buffer();
                    if let Ok(buf_guard) = buffer.lock() {
                        let font_size = 13.0;
                        let cell_w = 8.5;
                        let cell_h = 17.0;
                        let content_h = buf_guard.height() as f32 * cell_h;
                        let content_w = buf_guard.width() as f32 * cell_w;

                        ScrollArea::vertical()
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                let selection_id = Id::new("terminal_selection");
                                let mut selection: SelectionState = ui.memory(|m| m.data.get_temp(selection_id).unwrap_or_default());

                                let avail = ui.available_size();
                                let max_h = avail.y * 3.0;
                                let h = content_h.min(max_h);
                                let desired_size = Vec2::new(content_w.min(ui.available_width()), h);
                                let (response, painter) = ui.allocate_painter(desired_size, Sense::click_and_drag());

                                let origin = response.rect.min;
                                let clip_rect = response.rect;

                                // Handle selection input
                                if response.drag_started() {
                                    if let Some(pos) = ui.input(|i| i.pointer.press_origin()) {
                                        let rel_x = (pos.x - origin.x).max(0.0);
                                        let rel_y = (pos.y - origin.y).max(0.0);
                                        let col = ((rel_x / cell_w) as usize).min(buf_guard.width() - 1);
                                        let row = ((rel_y / cell_h) as usize).min(buf_guard.height() - 1);
                                        selection.start = Some((row, col));
                                        selection.end = Some((row, col));
                                    }
                                } else if response.dragged() {
                                    if let Some(pos) = ui.input(|i| i.pointer.latest_pos()) {
                                        let rel_x = (pos.x - origin.x).max(0.0);
                                        let rel_y = (pos.y - origin.y).max(0.0);
                                        let col = ((rel_x / cell_w) as usize).min(buf_guard.width() - 1);
                                        let row = ((rel_y / cell_h) as usize).min(buf_guard.height() - 1);
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

                                for row_idx in 0..buf_guard.height() {
                                    let y = origin.y + row_idx as f32 * cell_h;

                                    // Render selection highlight background
                                    for col_idx in 0..buf_guard.width() {
                                        if is_cell_selected(row_idx, col_idx, selection.start, selection.end) {
                                            let x = origin.x + col_idx as f32 * cell_w;
                                            painter.rect_filled(
                                                Rect::from_min_size(
                                                    Pos2::new(x, y),
                                                    Vec2::new(cell_w, cell_h),
                                                ),
                                                0.0,
                                                Color32::from_rgba_unmultiplied(100, 149, 237, 80),
                                            );
                                        }
                                    }

                                    let mut line_end = 0;
                                    for col_idx in (0..buf_guard.width()).rev() {
                                        if buf_guard.lines[row_idx][col_idx].c != ' ' {
                                            line_end = col_idx + 1;
                                            break;
                                        }
                                    }

                                    for col_idx in 0..line_end {
                                        let cell = buf_guard.lines[row_idx][col_idx];
                                        let x = origin.x + col_idx as f32 * cell_w;

                                        if cell.c != ' ' {
                                            let fg = if cell.bold {
                                                Color32::from_rgb(
                                                    (cell.fg[0] as u32 + 40).min(255) as u8,
                                                    (cell.fg[1] as u32 + 40).min(255) as u8,
                                                    (cell.fg[2] as u32 + 40).min(255) as u8,
                                                )
                                            } else {
                                                Color32::from_rgb(cell.fg[0], cell.fg[1], cell.fg[2])
                                            };

                                            painter.text(
                                                Pos2::new(x, y),
                                                Align2::LEFT_TOP,
                                                cell.c.to_string(),
                                                egui::FontId::new(font_size, egui::FontFamily::Monospace),
                                                fg,
                                            );
                                        }
                                    }
                                }

                                let (cursor_col, cursor_row) = buf_guard.cursor();
                                let cursor_x = origin.x + cursor_col as f32 * cell_w;
                                let cursor_y = origin.y + cursor_row as f32 * cell_h;

                                if cursor_x >= clip_rect.min.x && cursor_x <= clip_rect.max.x
                                    && cursor_y >= clip_rect.min.y && cursor_y <= clip_rect.max.y
                                {
                                    painter.text(
                                        Pos2::new(cursor_x, cursor_y),
                                        Align2::LEFT_TOP,
                                        "▌",
                                        egui::FontId::new(font_size, egui::FontFamily::Monospace),
                                        Color32::from_rgb(0, 200, 160),
                                    );
                                }

                                ui.memory_mut(|m| m.data.insert_temp(selection_id, selection));
                            });
                    }
                });
            });

        let response = ui.interact(rect, terminal_id, Sense::click());
        if response.clicked() {
            ui.memory_mut(|m| m.request_focus(terminal_id));
        }

        if focused {
            let mut needs_repaint = false;
            let modifiers = ui.input(|i| i.modifiers);

            // Clear selection on typing/key presses (excluding copy/paste)
            let has_keyboard_input = ui.input(|i| {
                i.events.iter().any(|e| match e {
                    egui::Event::Key { pressed: true, .. } | egui::Event::Text(_) => true,
                    _ => false,
                })
            });
            if has_keyboard_input {
                let selection_id = Id::new("terminal_selection");
                let is_copy = modifiers.ctrl && modifiers.shift && ui.input(|i| i.key_pressed(egui::Key::C));
                if !is_copy {
                    ui.memory_mut(|m| m.data.insert_temp(selection_id, SelectionState::default()));
                }
            }

            if modifiers.ctrl && modifiers.shift && ui.input(|i| i.key_pressed(egui::Key::C)) {
                let selection_id = Id::new("terminal_selection");
                let selection: SelectionState = ui.memory(|m| m.data.get_temp(selection_id).unwrap_or_default());
                let buffer = term.get_buffer();
                if let Ok(buf) = buffer.lock() {
                    let text = if let (Some(s), Some(e)) = (selection.start, selection.end) {
                        get_selected_text(&buf, s, e)
                    } else {
                        // Fallback: Copy the entire buffer screen
                        let mut t = String::new();
                        for row in &buf.lines {
                            let line: String = row.iter().map(|c| c.c).collect();
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

            if modifiers.ctrl && modifiers.shift && ui.input(|i| i.key_pressed(egui::Key::V)) {
                if let Some(text) = crate::ui::clipboard::paste_from_clipboard() {
                    for c in text.chars() {
                        term.handle_char(c);
                    }
                }
                needs_repaint = true;
            }

            if !modifiers.shift || !modifiers.ctrl {
                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    term.handle_key("Enter", false);
                    needs_repaint = true;
                }

                if ui.input(|i| i.key_pressed(egui::Key::Backspace)) {
                    term.handle_key("Backspace", false);
                    needs_repaint = true;
                }

                if ui.input(|i| i.key_pressed(egui::Key::Tab)) {
                    term.handle_key("Tab", false);
                    needs_repaint = true;
                }

                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                    term.handle_key("ArrowUp", false);
                    needs_repaint = true;
                }

                if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    term.handle_key("ArrowDown", false);
                    needs_repaint = true;
                }

                if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                    term.handle_key("ArrowLeft", false);
                    needs_repaint = true;
                }

                if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                    term.handle_key("ArrowRight", false);
                    needs_repaint = true;
                }

                if ui.input(|i| i.key_pressed(egui::Key::Home)) {
                    term.handle_key("Home", false);
                    needs_repaint = true;
                }

                if ui.input(|i| i.key_pressed(egui::Key::End)) {
                    term.handle_key("End", false);
                    needs_repaint = true;
                }

                if ui.input(|i| i.key_pressed(egui::Key::Delete)) {
                    term.handle_key("Delete", false);
                    needs_repaint = true;
                }

                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    term.handle_key("Escape", false);
                    needs_repaint = true;
                }

                if modifiers.ctrl {
                    if ui.input(|i| i.key_pressed(egui::Key::C)) {
                        term.handle_key("c", true);
                        needs_repaint = true;
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::D)) {
                        term.handle_key("d", true);
                        needs_repaint = true;
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::Z)) {
                        term.handle_key("z", true);
                        needs_repaint = true;
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::L)) {
                        term.handle_key("l", true);
                        needs_repaint = true;
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::U)) {
                        term.handle_key("u", true);
                        needs_repaint = true;
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::K)) {
                        term.handle_key("k", true);
                        needs_repaint = true;
                    }
                }

                for event in ui.input(|i| i.events.clone()) {
                    match event {
                        egui::Event::Text(text) => {
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
                                needs_repaint = true;
                            }
                        }
                        _ => {}
                    }
                }
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
        let line_cells = &buf.lines[row];
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
