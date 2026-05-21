use egui::{Rect, ScrollArea, Frame, Color32, Ui, RichText, Sense, Id};
use crate::theme::Theme;
use crate::ui::terminal::TerminalWidget;

pub fn draw_terminal(ui: &mut Ui, rect: Rect, term: &mut TerminalWidget, theme: &Theme) {
    term.poll_results();

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
                            RichText::new("bash")
                                .monospace()
                                .size(10.0)
                                .color(theme.dimmed()),
                        );
                    });

                    ui.add_space(8.0);

                    ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .stick_to_bottom(true)
                        .max_height(rect.height() - 50.0)
                        .show(ui, |ui| {
                            for entry in &term.history {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("❯ ")
                                            .monospace()
                                            .size(13.0)
                                            .color(Color32::from_rgb(0, 200, 160)),
                                    );
                                    ui.label(
                                        RichText::new(&entry.command)
                                            .monospace()
                                            .size(13.0)
                                            .color(theme.high()),
                                    );
                                });

                                if !entry.output.is_empty() {
                                    ui.add_space(2.0);
                                    let output_color = if entry.output.starts_with("Error:") {
                                        Color32::from_rgb(255, 100, 100)
                                    } else if entry.output == "(running...)" {
                                        theme.dimmed()
                                    } else {
                                        theme.terminal_text
                                    };
                                    ui.label(
                                        RichText::new(&entry.output)
                                            .monospace()
                                            .size(12.0)
                                            .color(output_color),
                                    );
                                    ui.add_space(6.0);
                                }
                            }

                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("❯ ")
                                        .monospace()
                                        .size(13.0)
                                        .color(Color32::from_rgb(0, 200, 160)),
                                );

                                let cursor_char = if term.cursor_visible() { "▌" } else { " " };
                                let display = format!("{}{}", term.current_input, cursor_char);

                                ui.label(
                                    RichText::new(display)
                                        .monospace()
                                        .size(13.0)
                                        .color(theme.high()),
                                );
                            });
                        });
                });
            });

        let response = ui.interact(rect, terminal_id, Sense::click());
        if response.clicked() {
            ui.memory_mut(|m| m.request_focus(terminal_id));
        }

        if focused {
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                term.execute();
                ui.ctx().request_repaint();
            }

            if ui.input(|i| i.key_pressed(egui::Key::Backspace)) {
                term.backspace();
                ui.ctx().request_repaint();
            }

            if let Some(text) = ui.input(|i| i.events.iter().find_map(|e| {
                if let egui::Event::Text(t) = e { Some(t.clone()) } else { None }
            })) {
                for c in text.chars() {
                    term.append_char(c);
                }
                ui.ctx().request_repaint();
            }
        }
    });
}
