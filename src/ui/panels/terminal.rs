use egui::{Rect, Frame, Color32, Ui, RichText, Sense, Id, FontId, Align2, Pos2, ScrollArea, Vec2};
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
                            .auto_shrink(false)
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                let desired_size = Vec2::new(content_w.min(ui.available_width()), content_h);
                                let (response, painter) = ui.allocate_painter(desired_size, Sense::hover());

                                let clip_rect = response.rect;
                                let origin = clip_rect.min;

                                painter.rect_filled(
                                    Rect::from_min_size(origin, desired_size),
                                    0.0,
                                    Color32::from_rgba_unmultiplied(6, 10, 8, 255),
                                );

                                for row_idx in 0..buf_guard.height() {
                                    let y = origin.y + row_idx as f32 * cell_h;
                                    if y + cell_h < clip_rect.min.y || y > clip_rect.max.y {
                                        continue;
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
                                        if x + cell_w < clip_rect.min.x || x > clip_rect.max.x {
                                            continue;
                                        }

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

            if modifiers.ctrl && modifiers.shift && ui.input(|i| i.key_pressed(egui::Key::C)) {
                let buffer = term.get_buffer();
                if let Ok(buf) = buffer.lock() {
                    let mut text = String::new();
                    for row in &buf.lines {
                        let line: String = row.iter().map(|c| c.c).collect();
                        text.push_str(line.trim_end());
                        text.push('\n');
                    }
                    use std::io::Write;
                    if let Ok(mut child) = std::process::Command::new("xsel")
                        .arg("--clipboard")
                        .arg("--input")
                        .stdin(std::process::Stdio::piped())
                        .spawn()
                    {
                        if let Some(mut stdin) = child.stdin.take() {
                            let _ = stdin.write_all(text.as_bytes());
                        }
                        let _ = child.wait();
                    } else if let Ok(mut child) = std::process::Command::new("xclip")
                        .arg("-selection")
                        .arg("clipboard")
                        .arg("-i")
                        .stdin(std::process::Stdio::piped())
                        .spawn()
                    {
                        if let Some(mut stdin) = child.stdin.take() {
                            let _ = stdin.write_all(text.as_bytes());
                        }
                        let _ = child.wait();
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

            if needs_repaint {
                ui.ctx().request_repaint();
            }
        }
    });
}
