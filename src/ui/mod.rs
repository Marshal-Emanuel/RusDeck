pub mod background;
pub mod clipboard;
mod layout;
pub mod panels;
pub mod terminal;

use egui::{Color32, Pos2, Vec2, Rect, Align2, RichText};
use crate::app::AppState;
use crate::theme::Theme;
use background::BackgroundCache;
use layout::Layout;
use panels::filesystem::draw_filesystem;
use panels::hardware::draw_hardware;
use panels::network::draw_network;
use panels::processes::draw_processes;
use panels::system_logs::draw_system_logs;
use panels::terminal::draw_terminal;
use terminal::TerminalTab;

pub fn setup_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = Color32::TRANSPARENT;
    visuals.window_fill = Color32::TRANSPARENT;
    visuals.window_stroke = egui::Stroke::NONE;
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(0.0, 0.0);
    style.spacing.window_margin = egui::Margin::ZERO;

    let nerd_font = egui::FontData::from_static(include_bytes!(
        "/home/marshal/.local/share/fonts/Fira Code Regular Nerd Font Complete Windows Compatible.ttf"
    ));
    let orbitron_font = egui::FontData::from_static(include_bytes!(
        "/home/marshal/.local/share/fonts/Orbitron-Bold.ttf"
    ));
    let mut fonts = egui::epaint::text::FontDefinitions::default();
    fonts.font_data.insert("FiraCodeNF".to_owned(), nerd_font.into());
    fonts.font_data.insert("Orbitron".to_owned(), orbitron_font.into());
    
    if let Some(vec) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
        vec.insert(0, "FiraCodeNF".to_owned());
    } else {
        fonts.families.insert(
            egui::FontFamily::Monospace,
            vec!["FiraCodeNF".to_owned()],
        );
    }
    
    if let Some(vec) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        vec.insert(0, "FiraCodeNF".to_owned());
    } else {
        fonts.families.insert(
            egui::FontFamily::Proportional,
            vec!["FiraCodeNF".to_owned()],
        );
    }

    fonts.families.insert(
        egui::FontFamily::Name("Orbitron".into()),
        vec!["Orbitron".to_owned()],
    );

    style.text_styles = [
        (egui::TextStyle::Small, egui::FontId::new(10.0, egui::FontFamily::Monospace)),
        (egui::TextStyle::Body, egui::FontId::new(13.0, egui::FontFamily::Monospace)),
        (egui::TextStyle::Button, egui::FontId::new(13.0, egui::FontFamily::Monospace)),
        (egui::TextStyle::Heading, egui::FontId::new(18.0, egui::FontFamily::Monospace)),
        (egui::TextStyle::Monospace, egui::FontId::new(13.0, egui::FontFamily::Monospace)),
    ]
    .into();
    ctx.set_fonts(fonts);
    ctx.set_style(style);
}

pub fn draw(ctx: &egui::Context, state: &AppState, theme: &Theme, bg_cache: &mut BackgroundCache, terminals: &mut Vec<TerminalTab>, active_terminal_idx: &mut usize, file_explorer: &mut panels::filesystem::FileExplorerState) -> Option<std::path::PathBuf> {
    let mut dir_change = None;
    egui::Area::new("root".into())
        .fixed_pos(Pos2::new(0.0, 0.0))
        .show(ctx, |ui| {
            let avail = ui.available_rect_before_wrap();
            let screen_w = avail.width().max(800.0);
            let screen_h = avail.height().max(600.0);
            let painter = ui.painter_at(avail);

            background::draw_background(&painter, Rect::from_min_size(Pos2::ZERO, egui::vec2(screen_w, screen_h)), theme, bg_cache);

            let layout = Layout::new(screen_w, screen_h);

            draw_hardware(&painter, layout.hardware, state, theme);
            
            ui.allocate_ui_at_rect(layout.storage, |ui| {
                dir_change = draw_filesystem(ui, layout.storage, file_explorer, theme);
            });
            
            draw_network(&painter, layout.network, state, theme);
            draw_processes(&painter, layout.processes, state, theme);
            draw_system_logs(&painter, layout.system_logs, state, theme);

            ui.allocate_ui_at_rect(layout.terminal, |ui| {
                draw_terminal(ui, layout.terminal, terminals, active_terminal_idx, theme);
            });

            let tb = &layout.topbar;

            // 1. Left-aligned region for Application Name and Version
            let left_rect = Rect::from_min_max(
                tb.min,
                Pos2::new(tb.center().x - 100.0, tb.max.y)
            );
            ui.allocate_ui_at_rect(left_rect, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::BOTTOM), |ui| {
                        ui.label(
                            RichText::new("RUSDECK")
                                .font(egui::FontId::new(24.0, egui::FontFamily::Name("Orbitron".into())))
                                .color(theme.high())
                        );
                        
                        ui.add_space(6.0);
                        
                        ui.label(
                            RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                                .monospace()
                                .size(12.0)
                                .color(theme.low())
                        );
                    });
                });
            });

            // 2. Mathematically centered region for Clock and Milliseconds
            let clock_width = 100.0; // Total width for HH:MM:SS.mmm at 18.0/13.0 size
            let center_rect = Rect::from_min_max(
                Pos2::new(tb.center().x - clock_width / 2.0, tb.min.y),
                Pos2::new(tb.center().x + clock_width / 2.0, tb.max.y)
            );
            ui.allocate_ui_at_rect(center_rect, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::BOTTOM), |ui| {
                    let now = chrono::Local::now();
                    let time_str = now.format("%H:%M:%S").to_string();
                    let ms_str = now.format(".%3f").to_string();
                    
                    ui.label(
                        RichText::new(time_str)
                            .monospace()
                            .size(18.0) // Reduced font size for better proportions
                            .strong()
                            .color(theme.high())
                    );
                    ui.label(
                        RichText::new(ms_str)
                            .monospace()
                            .size(13.0) // Milliseconds remains at 13.0
                            .color(theme.low())
                    );
                    
                    ui.ctx().request_repaint_after(std::time::Duration::from_millis(33));
                });
            });

            // 3. Right-aligned region for Close Button
            let right_rect = Rect::from_min_max(
                Pos2::new(tb.center().x + 100.0, tb.min.y),
                tb.max
            );
            ui.allocate_ui_at_rect(right_rect, |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(10.0);
                    
                    // Close / Exit application button [ X ]
                    let (exit_resp, exit_painter) = ui.allocate_painter(Vec2::new(32.0, 24.0), egui::Sense::click());
                    let exit_hovered = exit_resp.hovered();
                    let exit_color = if exit_hovered {
                        Color32::from_rgb(255, 70, 70) // Glowing hazard red
                    } else {
                        theme.low()
                    };
                    
                    // Draw a sleek frame for the exit button
                    let frame_stroke = if exit_hovered {
                        egui::Stroke::new(1.5, Color32::from_rgb(255, 70, 70))
                    } else {
                        egui::Stroke::new(1.0, theme.mid())
                    };
                    
                    exit_painter.rect_stroke(
                        exit_resp.rect.shrink(1.0),
                        0.0,
                        frame_stroke,
                    );
                    
                    exit_painter.text(
                        exit_resp.rect.center(),
                        Align2::CENTER_CENTER,
                        "X",
                        egui::FontId::new(13.0, egui::FontFamily::Monospace),
                        exit_color,
                    );

                    if exit_resp.clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });
        
    dir_change
}
