use egui::{Rect, Frame, Color32, Ui, RichText, Sense, Id, Align2, Pos2, ScrollArea, Vec2};
use crate::theme::Theme;
use crate::ui::terminal::TerminalWidget;
use crate::ui::clipboard;

#[derive(Clone, Copy, Default)]
struct Selection {
    start: Option<usize>,
    end: Option<usize>,
}

impl Selection {
    fn is_empty(&self) -> bool {
        self.start.is_none() && self.end.is_none()
    }

    fn rows(&self) -> Option<(usize, usize)> {
        match (self.start, self.end) {
            (Some(s), Some(e)) if s <= e => Some((s, e)),
            (Some(s), Some(e)) => Some((e, s)),
            (Some(s), None) => Some((s, s)),
            (None, Some(e)) => Some((e, e)),
            _ => None,
        }
    }
}

pub struct TerminalPanel {
    selection: Selection,
    show_copied_toast: bool,
    toast_timer: f32,
}

impl Default for TerminalPanel {
    fn default() -> Self {
        Self {
            selection: Selection::default(),
            show_copied_toast: false,
            toast_timer: 0.0,
        }
    }
}

impl TerminalPanel {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_selected_text(&self, buf: &crate::ui::terminal::TerminalBuffer) -> String {
        match self.selection.rows() {
            Some((start, end)) => {
                (start..=end)
                    .filter_map(|i| buf.lines.get(i))
                    .map(|row| {
                        row.iter()
                            .map(|cell| cell.c)
                            .collect::<String>()
                            .trim_end()
                            .to_string()
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            None => {
                buf.lines
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|cell| cell.c)
                            .collect::<String>()
                            .trim_end()
                            .to_string()
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
    }

    fn handle_copy(&mut self, term: &TerminalWidget) {
        let buffer = term.get_buffer();
        if let Ok(buf) = buffer.lock() {
            let text = self.get_selected_text(&buf);
            clipboard::copy_to_clipboard(&text);
            self.show_copied_toast = true;
            self.toast_timer = 1.5;
            self.selection = Selection::default();
        }
    }

    fn clear_selection(&mut self) {
        self.selection = Selection::default();
    }

    fn update_toast(&mut self, dt: f32) {
        if self.show_copied_toast {
            self.toast_timer -= dt;
            if self.toast_timer <= 0.0 {
                self.show_copied_toast = false;
            }
        }
    }
}

pub fn draw_terminal(
    ui: &mut Ui,
    rect: Rect,
    term: &mut TerminalWidget,
    theme: &Theme,
    panel: &mut TerminalPanel,
) {
    let terminal_id = Id::new("terminal_panel");
    let focused = ui.memory(|m| m.has_focus(terminal_id));

    panel.update_toast(ui.input(|i| i.unstable_dt));

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

                        let available_h = ui.available_height();
                        let visible_rows = ((available_h / cell_h) as usize).min(buf_guard.height());
                        let visible_h = visible_rows as f32 * cell_h;
                        let start_row = buf_guard.height() - visible_rows;

                        let desired_size = Vec2::new(content_w, visible_h);
                        let (response, painter) = ui.allocate_painter(desired_size, Sense::hover().union(Sense::drag()));

                        let origin = response.rect.min;

                        painter.rect_filled(
                            Rect::from_min_size(origin, desired_size),
                            0.0,
                            Color32::from_rgba_unmultiplied(6, 10, 8, 255),
                        );

                        for i in 0..visible_rows {
                            let row_idx = start_row + i;
                            let y = origin.y + i as f32 * cell_h;

                            let is_selected = panel
                                .selection
                                .rows()
                                .map(|(s, e)| row_idx >= s && row_idx <= e)
                                .unwrap_or(false);

                            if is_selected {
                                painter.rect_filled(
                                    Rect::from_min_size(
                                        Pos2::new(origin.x, y),
                                        Vec2::new(desired_size.x, cell_h),
                                    ),
                                    0.0,
                                    Color32::from_rgba_unmultiplied(0, 100, 80, 80),
                                );
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
                                    let fg = if is_selected {
                                        Color32::from_rgb(0, 255, 200)
                                    } else if cell.bold {
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
                        let rel_cursor_row = cursor_row.saturating_sub(start_row);
                        let cursor_x = origin.x + cursor_col as f32 * cell_w;
                        let cursor_y = origin.y + rel_cursor_row as f32 * cell_h;

                        if rel_cursor_row < visible_rows {
                            painter.text(
                                Pos2::new(cursor_x, cursor_y),
                                Align2::LEFT_TOP,
                                "▌",
                                egui::FontId::new(font_size, egui::FontFamily::Monospace),
                                Color32::from_rgb(0, 200, 160),
                            );
                        }
                    }
                });
            });

        let response = ui.interact(rect, terminal_id, Sense::click().union(Sense::drag()));

        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
        }

        if response.drag_started() {
            let mouse_pos = ui.input(|i| i.pointer.interact_pos()).unwrap_or(Pos2::ZERO);
            let rel_y = mouse_pos.y - response.rect.min.y;
            let cell_h = 17.0;
            let row = (rel_y / cell_h) as usize;
            panel.selection = Selection {
                start: Some(row),
                end: Some(row),
            };
            ui.ctx().request_repaint();
        }

        if response.dragged() {
            let mouse_pos = ui.input(|i| i.pointer.interact_pos()).unwrap_or(Pos2::ZERO);
            let rel_y = mouse_pos.y - response.rect.min.y;
            let cell_h = 17.0;
            let row = (rel_y / cell_h) as usize;
            panel.selection.end = Some(row);
            ui.ctx().request_repaint();
        }

        if response.drag_stopped() {
            if panel.selection.is_empty() {
                panel.selection = Selection::default();
            }
            ui.ctx().request_repaint();
        }

        if response.clicked() {
            if !panel.selection.is_empty() {
                panel.selection = Selection::default();
            }
            ui.memory_mut(|m| m.request_focus(terminal_id));
        }

        if focused {
            let mut needs_repaint = false;
            let modifiers = ui.input(|i| i.modifiers);

            if modifiers.ctrl && modifiers.shift && ui.input(|i| i.key_pressed(egui::Key::C)) {
                let buffer = term.get_buffer();
                if let Ok(buf) = buffer.lock() {
                    let text = panel.get_selected_text(&buf);
                    clipboard::copy_to_clipboard(&text);
                }
                panel.selection = Selection::default();
                ui.ctx().request_repaint();
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
                    panel.selection = Selection::default();
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
                    if let egui::Event::Text(text) = event {
                        for c in text.chars() {
                            if c.is_control() {
                                continue;
                            }
                            term.handle_char(c);
                            needs_repaint = true;
                        }
                    }
                }
            }

            if panel.show_copied_toast {
                ui.allocate_ui_at_rect(
                    Rect::from_min_size(
                        Pos2::new(rect.max.x - 80.0, rect.min.y + 30.0),
                        Vec2::new(70.0, 24.0),
                    ),
                    |ui| {
                        ui.label(
                            RichText::new("Copied!")
                                .monospace()
                                .size(11.0)
                                .color(Color32::from_rgb(0, 255, 200)),
                        );
                    },
                );
            }

            if needs_repaint {
                ui.ctx().request_repaint();
            }
        }
    });
}