pub mod background;
pub mod clipboard;
mod layout;
pub mod panels;
pub mod terminal;

use egui::{Color32, Pos2, Vec2, Rect, Align2, RichText, Rounding, Stroke};
use crate::app::AppState;
use crate::theme::{Theme, ThemeVariant, ALL_THEMES};
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

pub fn draw(ctx: &egui::Context, state: &AppState, theme: &mut Theme, theme_variant: &mut ThemeVariant, show_theme_panel: &mut bool, bg_cache: &mut BackgroundCache, terminals: &mut Vec<TerminalTab>, active_terminal_idx: &mut usize, file_explorer: &mut panels::filesystem::FileExplorerState) -> Option<std::path::PathBuf> {
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

            // 3. Right-aligned region for Settings and Close Button
            let right_rect = Rect::from_min_max(
                Pos2::new(tb.center().x + 100.0, tb.min.y),
                tb.max
            );
            let mut gear_rect = Rect::NOTHING;
            let mut gear_clicked = false;
            ui.allocate_ui_at_rect(right_rect, |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(10.0);

                    // Close / Exit application button [ X ]
                    let (exit_resp, exit_painter) = ui.allocate_painter(Vec2::new(32.0, 24.0), egui::Sense::click());
                    let exit_hovered = exit_resp.hovered();
                    let exit_color = if exit_hovered {
                        Color32::from_rgb(255, 70, 70)
                    } else {
                        theme.low()
                    };

                    let frame_stroke = if exit_hovered {
                        Stroke::new(1.5, Color32::from_rgb(255, 70, 70))
                    } else {
                        Stroke::new(1.0, theme.mid())
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

                    ui.add_space(6.0);

                    // Settings gear button
                    let gear_size = 32.0;
                    let (gear_resp, gear_painter) = ui.allocate_painter(Vec2::new(gear_size, 24.0), egui::Sense::click());
                    let gear_hovered = gear_resp.hovered();
                    let gear_color = if *show_theme_panel {
                        theme.accent
                    } else if gear_hovered {
                        theme.accent
                    } else {
                        theme.accent.linear_multiply(0.6)
                    };

                    gear_painter.rect_stroke(
                        gear_resp.rect.shrink(1.0),
                        0.0,
                        Stroke::new(1.0, if gear_hovered || *show_theme_panel { theme.accent } else { theme.mid() }),
                    );

                    gear_painter.text(
                        gear_resp.rect.center(),
                        Align2::CENTER_CENTER,
                        "\u{2699}",
                        egui::FontId::new(15.0, egui::FontFamily::Monospace),
                        gear_color,
                    );

                    if gear_resp.clicked() {
                        *show_theme_panel = !*show_theme_panel;
                        gear_clicked = true;
                    }

                    gear_rect = gear_resp.rect;
                });
            });

            // Theme picker dropdown
            if *show_theme_panel {
                let dropdown_w = 220.0;
                let item_h = 30.0;
                let dd_h = ALL_THEMES.len() as f32 * item_h + 12.0;
                let dd_left = gear_rect.left() - dropdown_w + gear_rect.width();
                let dd_top = gear_rect.bottom() + 4.0;

                let dd_rect = Rect::from_min_max(
                    Pos2::new(dd_left, dd_top),
                    Pos2::new(dd_left + dropdown_w, dd_top + dd_h),
                );

                let dd_area_id = egui::Id::new("theme_dropdown");
                let dd_res = egui::Area::new(dd_area_id)
                    .fixed_pos(dd_rect.min)
                    .movable(false)
                    .show(ctx, |ui| {
                        let panel_rect = Rect::from_min_size(Pos2::ZERO, egui::vec2(dropdown_w, dd_h));
                        let panel_painter = ui.painter_at(panel_rect);

                        panel_painter.rect_filled(panel_rect, Rounding::same(4.0), theme.background);
                        panel_painter.rect_stroke(panel_rect, Rounding::same(4.0), Stroke::new(1.0, theme.mid()));

                        ui.allocate_ui_at_rect(panel_rect.shrink2(egui::vec2(8.0, 6.0)), |ui| {
                            ui.vertical(|ui| {
                                for variant in ALL_THEMES {
                                    let is_active = variant == theme_variant;
                                    let preview_color = variant.preview();
                                    let variant_name = variant.name();

                                    let item_resp = ui.allocate_rect(
                                        Rect::from_min_size(
                                            ui.cursor().min,
                                            egui::vec2(dropdown_w - 16.0, item_h),
                                        ),
                                        egui::Sense::click(),
                                    );
                                    let item_rect = item_resp.rect;

                                    let item_bg = if item_rect.contains(ui.input(|i| i.pointer.latest_pos().unwrap_or(Pos2::ZERO))) {
                                        theme.faint()
                                    } else {
                                        Color32::TRANSPARENT
                                    };
                                    if item_bg != Color32::TRANSPARENT {
                                        let item_painter = ui.painter_at(item_rect);
                                        item_painter.rect_filled(item_rect, Rounding::same(3.0), item_bg);
                                    }

                                    let dot_size = 8.0;
                                    let dot_pos = Pos2::new(item_rect.left() + 6.0, item_rect.center().y);
                                    let dot_stroke = if is_active {
                                        Stroke::new(2.0, theme.high())
                                    } else {
                                        Stroke::NONE
                                    };
                                    if dot_stroke.width > 0.0 {
                                        ui.painter_at(item_rect).circle(dot_pos, dot_size / 2.0 + 1.5, Color32::TRANSPARENT, dot_stroke);
                                    }
                                    ui.painter_at(item_rect).circle_filled(dot_pos, dot_size / 2.0, preview_color);

                                    let name_color = if is_active { theme.accent } else { theme.high() };
                                    let label_pos = Pos2::new(item_rect.left() + 22.0, item_rect.center().y);
                                    ui.painter_at(item_rect).text(
                                        label_pos,
                                        Align2::LEFT_CENTER,
                                        variant_name,
                                        egui::FontId::new(13.0, egui::FontFamily::Monospace),
                                        name_color,
                                    );

                                    if is_active {
                                        ui.painter_at(item_rect).text(
                                            Pos2::new(item_rect.right() - 6.0, item_rect.center().y),
                                            Align2::RIGHT_CENTER,
                                            "\u{2713}",
                                            egui::FontId::new(12.0, egui::FontFamily::Monospace),
                                            theme.low(),
                                        );
                                    }

                                    if item_rect.contains(ui.input(|i| i.pointer.latest_pos().unwrap_or(Pos2::ZERO)))
                                        && ui.input(|i| i.pointer.any_click())
                                    {
                                        if *theme_variant != *variant {
                                            *theme_variant = *variant;
                                            *theme = Theme::from_variant(*variant);
                                            *bg_cache = BackgroundCache::new();
                                        }
                                    }

                                    ui.add_space(0.0);
                                }
                            });
                        });
                    });

                // Close dropdown if click outside (but not on the gear button itself)
                if dd_res.response.clicked_elsewhere() && !gear_clicked {
                    *show_theme_panel = false;
                }
            }
        });
        
    dir_change
}
