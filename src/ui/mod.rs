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
    let gear_clicked = egui::Area::new("root".into())
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

            // 3. Right-aligned region for Close Button only
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
                });
            });

            // ── Bottom-right floating gear ──
            let gear_pad = 16.0;
            let gear_size = 36.0;
            let gear_pos = Pos2::new(screen_w - gear_pad - gear_size, screen_h - gear_pad - gear_size);
            let gear_rect = Rect::from_min_size(gear_pos, Vec2::splat(gear_size));
            let gear_id = egui::Id::new("gear_btn");
            let gear_resp = ui.interact(gear_rect, gear_id, egui::Sense::click());
            let gear_hovered = gear_resp.hovered() || *show_theme_panel;
            let gear_color = if gear_hovered { theme.accent } else { theme.accent.linear_multiply(0.5) };

            let gp = ui.painter_at(gear_rect);
            gp.rect_stroke(gear_rect.shrink(1.0), Rounding::same(5.0), Stroke::new(1.5, if gear_hovered { theme.accent } else { theme.mid() }));
            gp.text(gear_rect.center(), Align2::CENTER_CENTER, "\u{2699}", egui::FontId::new(18.0, egui::FontFamily::Monospace), gear_color);

            let mut gear_clicked = false;
            if gear_resp.clicked() {
                *show_theme_panel = !*show_theme_panel;
                gear_clicked = true;
            }

            gear_clicked
        }).inner;

    // ── Settings modal (outside root Area for proper screen-center positioning) ──
    if *show_theme_panel {
        let screen = ctx.screen_rect();
        let modal_w = 380.0;
        let item_h = 32.0;
        let header_h = 40.0;
        let content_pad = 14.0;
        let inner_h = ALL_THEMES.len() as f32 * item_h;
        let modal_h = header_h + content_pad * 2.0 + inner_h;

        let modal_pos = Pos2::new(
            (screen.width() - modal_w) / 2.0,
            (screen.height() - modal_h) / 2.0,
        );

        let settings_id = egui::Id::new("settings_modal");
        let settings_layer = egui::LayerId::new(egui::Order::Foreground, settings_id);
        let painter = egui::Painter::new(ctx.clone(), settings_layer, screen);

        // Full-screen overlay
        painter.rect_filled(screen, Rounding::ZERO, Color32::from_black_alpha(140));

        // Panel rect in screen coords
        let panel_rect = Rect::from_min_size(modal_pos, egui::vec2(modal_w, modal_h));
        let clip = 12.0_f32;

        // Chamfered polygon (asymmetric: top-right + bottom-left, with step at bottom-right)
        let step_h = 24.0;
        let step_x = panel_rect.right() - 120.0;
        let chamfer_points = vec![
            Pos2::new(panel_rect.left(), panel_rect.top()),
            Pos2::new(panel_rect.right() - clip, panel_rect.top()),
            Pos2::new(panel_rect.right(), panel_rect.top() + clip),
            Pos2::new(panel_rect.right(), panel_rect.bottom() - step_h),
            Pos2::new(step_x, panel_rect.bottom() - step_h),
            Pos2::new(step_x - step_h, panel_rect.bottom()),
            Pos2::new(panel_rect.left() + clip, panel_rect.bottom()),
            Pos2::new(panel_rect.left(), panel_rect.bottom() - clip),
        ];
        let fill = Color32::from_rgba_unmultiplied(6, 10, 8, 255);
        painter.add(egui::Shape::convex_polygon(chamfer_points.clone(), fill, egui::Stroke::NONE));
        painter.add(egui::Shape::closed_line(chamfer_points, egui::Stroke::new(1.0, theme.low())));

        // Neon accent brackets at chamfer corners
        let thick_stroke = egui::Stroke::new(2.5, theme.accent);
        // Top-Right Bracket
        painter.add(egui::Shape::line(vec![
            Pos2::new(panel_rect.right() - clip - 12.0, panel_rect.top()),
            Pos2::new(panel_rect.right() - clip, panel_rect.top()),
            Pos2::new(panel_rect.right(), panel_rect.top() + clip),
            Pos2::new(panel_rect.right(), panel_rect.top() + clip + 12.0),
        ], thick_stroke));
        // Bottom-Left Bracket
        painter.add(egui::Shape::line(vec![
            Pos2::new(panel_rect.left() + clip + 12.0, panel_rect.bottom()),
            Pos2::new(panel_rect.left() + clip, panel_rect.bottom()),
            Pos2::new(panel_rect.left(), panel_rect.bottom() - clip),
            Pos2::new(panel_rect.left(), panel_rect.bottom() - clip - 12.0),
        ], thick_stroke));

        // Nested chunk segments in the step notch
        let h_height = 12.0;
        let chunk_widths = vec![60.0, 8.0, 20.0];
        let mut current_x = step_x + 15.0;
        for &w in &chunk_widths {
            if current_x + w + h_height > panel_rect.right() - 10.0 { break; }
            let chunk_poly = vec![
                Pos2::new(current_x, panel_rect.bottom()),
                Pos2::new(current_x + w, panel_rect.bottom()),
                Pos2::new(current_x + w + h_height, panel_rect.bottom() - h_height),
                Pos2::new(current_x + h_height, panel_rect.bottom() - h_height),
            ];
            painter.add(egui::Shape::convex_polygon(chunk_poly, theme.accent, egui::Stroke::NONE));
            current_x += w + 6.0;
        }

        // Dot indicator
        painter.rect_filled(
            Rect::from_min_size(Pos2::new(panel_rect.left() + clip + 4.0, panel_rect.top() + 15.0), egui::vec2(4.0, 4.0)),
            0.0,
            theme.accent,
        );

        // Label
        painter.text(
            Pos2::new(panel_rect.left() + clip + 14.0, panel_rect.top() + 10.0),
            Align2::LEFT_TOP,
            "SETTINGS",
            egui::FontId::new(14.0, egui::FontFamily::Monospace),
            theme.high(),
        );

        // Close button
        let close_btn_size = 22.0;
        let close_btn_rect = Rect::from_min_size(
            Pos2::new(panel_rect.right() - content_pad - close_btn_size, panel_rect.top() + 8.0),
            Vec2::splat(close_btn_size),
        );
        let pointer_pos = ctx.input(|i| i.pointer.latest_pos()).unwrap_or(Pos2::ZERO);
        let close_hov = close_btn_rect.contains(pointer_pos);
        let close_clicked = close_hov && ctx.input(|i| i.pointer.any_click());
        painter.rect_stroke(close_btn_rect, egui::Rounding::same(3.0), egui::Stroke::new(1.0, if close_hov { Color32::from_rgb(255, 80, 80) } else { theme.mid() }));
        painter.text(close_btn_rect.center(), Align2::CENTER_CENTER, "X", egui::FontId::new(13.0, egui::FontFamily::Monospace), if close_hov { Color32::from_rgb(255, 80, 80) } else { theme.low() });
        if close_clicked {
            *show_theme_panel = false;
        }

        // Theme selection list
        let list_top = panel_rect.top() + header_h;
        for (i, variant) in ALL_THEMES.iter().enumerate() {
            let is_active = *variant == *theme_variant;
            let preview_color = variant.preview();
            let variant_name = variant.name();

            let item_y = list_top + i as f32 * item_h;
            let item_rect = Rect::from_min_size(
                Pos2::new(panel_rect.left() + content_pad, item_y),
                egui::vec2(modal_w - content_pad * 2.0, item_h),
            );

            let item_hovered = item_rect.contains(pointer_pos);
            let item_clicked = item_hovered && ctx.input(|i| i.pointer.any_click());

            // Hover background
            let is_hovered = item_hovered;
            let v_bg = if is_hovered { theme.faint() } else if is_active { theme.background.linear_multiply(1.3) } else { Color32::TRANSPARENT };
            if v_bg != Color32::TRANSPARENT {
                painter.rect_filled(item_rect, Rounding::same(4.0), v_bg);
            }

            // Active indicator bar
            if is_active {
                painter.rect_filled(
                    Rect::from_min_size(Pos2::new(item_rect.left(), item_rect.top() + 2.0), egui::vec2(3.0, item_rect.height() - 4.0)),
                    Rounding::same(1.5),
                    theme.accent,
                );
            }

            // Color parallelogram (slanted to match chamfer style)
            let dot_cx = item_rect.left() + 16.0;
            let dot_cy = item_rect.center().y;
            let dw = 5.0;
            let dh = 6.0;
            let skew = 2.0;
            let dot_pts = vec![
                Pos2::new(dot_cx - dw + skew, dot_cy - dh),
                Pos2::new(dot_cx + dw + skew, dot_cy - dh),
                Pos2::new(dot_cx + dw - skew, dot_cy + dh),
                Pos2::new(dot_cx - dw - skew, dot_cy + dh),
            ];
            painter.add(egui::Shape::convex_polygon(dot_pts, preview_color, egui::Stroke::NONE));

            // Name
            let v_name_color = if is_active { theme.accent } else { theme.high() };
            painter.text(
                Pos2::new(item_rect.left() + 30.0, item_rect.center().y),
                Align2::LEFT_CENTER,
                variant_name,
                egui::FontId::new(13.0, egui::FontFamily::Monospace),
                v_name_color,
            );

            if is_active {
                painter.text(
                    Pos2::new(item_rect.right() - 8.0, item_rect.center().y),
                    Align2::RIGHT_CENTER,
                    "\u{2713}",
                    egui::FontId::new(12.0, egui::FontFamily::Monospace),
                    theme.accent,
                );
            }

            if item_clicked {
                if *theme_variant != *variant {
                    *theme_variant = *variant;
                    *theme = Theme::from_variant(*variant);
                    *bg_cache = BackgroundCache::new();
                    crate::theme::save_theme(*variant);
                }
            }
        }

        // Click outside modal to close
        let bg_clicked = ctx.input(|i| i.pointer.any_click());
        let clicking_outside = bg_clicked && !panel_rect.contains(pointer_pos);
        if clicking_outside && !gear_clicked {
            *show_theme_panel = false;
        }
    }

    dir_change
}
