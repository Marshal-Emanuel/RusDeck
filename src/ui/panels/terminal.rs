use egui::{Rect, ScrollArea, TextEdit, Frame, Color32, FontId};
use crate::theme::Theme;
use crate::ui::terminal::TerminalWidget;

pub const CHAMFER: f32 = 10.0;

pub fn draw_terminal(ui: &mut egui::Ui, rect: Rect, term: &mut TerminalWidget, theme: &Theme) {
    ui.allocate_ui_at_rect(rect, |ui| {
        ui.vertical(|ui| {
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("TERMINAL")
                        .monospace()
                        .size(13.0)
                        .color(theme.low()),
                );
            });

            ui.add_space(6.0);

            Frame::none()
                .fill(Color32::from_rgba_unmultiplied(8, 12, 10, 255))
                .stroke(egui::Stroke::new(1.0, theme.mid()))
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .stick_to_bottom(true)
                        .max_height(rect.height() - 80.0)
                        .show(ui, |ui| {
                            for entry in &term.history {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("$ ")
                                            .monospace()
                                            .size(12.0)
                                            .color(theme.dimmed()),
                                    );
                                    ui.label(
                                        egui::RichText::new(&entry.command)
                                            .monospace()
                                            .size(12.0)
                                            .color(theme.high()),
                                    );
                                });
                                ui.label(
                                    egui::RichText::new(&entry.output)
                                        .monospace()
                                        .size(12.0)
                                        .color(theme.terminal_text),
                                );
                                ui.add_space(4.0);
                            }
                        });

                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("$ ")
                                .monospace()
                                .size(12.0)
                                .color(theme.dimmed()),
                        );
                        let response = ui.add_sized(
                            ui.available_size(),
                            TextEdit::singleline(&mut term.current_input)
                                .font(FontId::monospace(12.0))
                                .text_color(theme.terminal_text)
                                .desired_width(rect.width() - 40.0)
                                .hint_text("type command..."),
                        );

                        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            if let Some(cmd) = term.take_input() {
                                term.execute(&cmd);
                            }
                        }

                        if response.gained_focus() {
                            ui.ctx().request_repaint();
                        }
                    });
                });
        });
    });
}
