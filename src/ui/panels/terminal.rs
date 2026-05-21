use egui::{Rect, ScrollArea, TextEdit, Frame, Color32, FontId, Ui, RichText};
use crate::theme::Theme;
use crate::ui::terminal::TerminalWidget;

pub fn draw_terminal(ui: &mut Ui, rect: Rect, term: &mut TerminalWidget, theme: &Theme) {
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
                        .max_height(rect.height() - 70.0)
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
                                    ui.label(
                                        RichText::new(&entry.output)
                                            .monospace()
                                            .size(12.0)
                                            .color(theme.terminal_text),
                                    );
                                    ui.add_space(6.0);
                                }
                            }
                        });

                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("❯ ")
                                .monospace()
                                .size(13.0)
                                .color(Color32::from_rgb(0, 200, 160)),
                        );

                        let _cursor_char = if term.cursor_visible() { "▌" } else { " " };

                        let response = ui.add(
                            TextEdit::singleline(&mut term.current_input)
                                .font(FontId::monospace(13.0))
                                .text_color(theme.terminal_text)
                                .desired_width(rect.width() - 60.0)
                                .hint_text("type command...")
                                .frame(false),
                        );

                        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            if !term.current_input.is_empty() {
                                let cmd = term.current_input.clone();
                                term.current_input.clear();
                                term.execute(&cmd);
                            }
                        }

                        if response.has_focus() {
                            ui.ctx().request_repaint();
                        }
                    });
                });
            });
    });
}
