use egui::{Rect, Frame, Color32, Ui, RichText, Sense, Id, FontId, Align2, Pos2};
use crate::theme::Theme;
use crate::ui::terminal::TerminalWidget;

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
                    ui.add_space(2.0);

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

                    ui.add_space(8.0);

                    let buffer = term.get_buffer();
                    if let Ok(buf_guard) = buffer.lock() {
                        let cell_w = 8.0;
                        let cell_h = 16.0;
                        let painter = ui.painter();

                        for row_idx in 0..buf_guard.height() {
                            let y = row_idx as f32 * cell_h;
                            if y + cell_h > rect.height() - 50.0 {
                                break;
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
                                let x = col_idx as f32 * cell_w;
                                if x + cell_w > rect.width() - 24.0 {
                                    break;
                                }

                                if cell.c != ' ' {
                                    let fg = if cell.bold {
                                        Color32::from_rgb(
                                            (cell.fg[0] as u32 + 60).min(255) as u8,
                                            (cell.fg[1] as u32 + 60).min(255) as u8,
                                            (cell.fg[2] as u32 + 60).min(255) as u8,
                                        )
                                    } else {
                                        Color32::from_rgb(cell.fg[0], cell.fg[1], cell.fg[2])
                                    };

                                    painter.text(
                                        Pos2::new(x, y),
                                        Align2::LEFT_TOP,
                                        cell.c.to_string(),
                                        FontId::monospace(12.0),
                                        fg,
                                    );
                                }
                            }
                        }

                        let (cursor_col, cursor_row) = buf_guard.cursor();
                        let cursor_x = cursor_col as f32 * cell_w;
                        let cursor_y = cursor_row as f32 * cell_h;

                        if cursor_x < rect.width() - 24.0 && cursor_y < rect.height() - 50.0 {
                            painter.text(
                                Pos2::new(cursor_x, cursor_y),
                                Align2::LEFT_TOP,
                                "▌",
                                FontId::monospace(12.0),
                                Color32::from_rgb(0, 200, 160),
                            );
                        }
                    }
                });
            });

        let response = ui.interact(rect, terminal_id, Sense::click());
        if response.clicked() {
            ui.memory_mut(|m| m.request_focus(terminal_id));
        }

        if focused {
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                term.handle_key("Enter", false);
                ui.ctx().request_repaint();
            }

            if ui.input(|i| i.key_pressed(egui::Key::Backspace)) {
                term.handle_key("Backspace", false);
                ui.ctx().request_repaint();
            }

            if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                term.handle_key("ArrowUp", false);
                ui.ctx().request_repaint();
            }

            if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                term.handle_key("ArrowDown", false);
                ui.ctx().request_repaint();
            }

            if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                term.handle_key("ArrowLeft", false);
                ui.ctx().request_repaint();
            }

            if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                term.handle_key("ArrowRight", false);
                ui.ctx().request_repaint();
            }

            if let Some(text) = ui.input(|i| i.events.iter().find_map(|e| {
                if let egui::Event::Text(t) = e { Some(t.clone()) } else { None }
            })) {
                for c in text.chars() {
                    term.handle_char(c);
                }
                ui.ctx().request_repaint();
            }
        }
    });
}
