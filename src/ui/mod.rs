pub mod background;
mod layout;
pub mod panels;
pub mod terminal;

use egui::{Color32, Pos2, Vec2, Rect, Align2};
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
use terminal::TerminalWidget;

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
    let mut fonts = egui::epaint::text::FontDefinitions::default();
    fonts.font_data.insert("FiraCodeNF".to_owned(), nerd_font.into());
    fonts.families.insert(
        egui::FontFamily::Monospace,
        vec!["FiraCodeNF".to_owned()],
    );
    fonts.families.insert(
        egui::FontFamily::Proportional,
        vec!["FiraCodeNF".to_owned()],
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

pub fn draw(ctx: &egui::Context, state: &AppState, theme: &Theme, bg_cache: &mut BackgroundCache, term: &mut TerminalWidget) {
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
            draw_filesystem(&painter, layout.storage, state, theme);
            draw_network(&painter, layout.network, state, theme);
            draw_processes(&painter, layout.processes, state, theme);
            draw_system_logs(&painter, layout.system_logs, state, theme);

            ui.allocate_ui_at_rect(layout.terminal, |ui| {
                draw_terminal(ui, layout.terminal, term, theme);
            });

            let tb = &layout.topbar;
            painter.line_segment(
                [Pos2::new(tb.left(), tb.bottom()), Pos2::new(tb.right(), tb.bottom())],
                egui::Stroke::new(1.0, theme.mid()),
            );
            painter.text(
                Pos2::new(tb.left() + 10.0, tb.center().y),
                Align2::LEFT_CENTER,
                "SYS_NODE_01",
                egui::FontId::new(18.0, egui::FontFamily::Monospace),
                theme.dimmed(),
            );
            painter.text(
                Pos2::new(tb.right() - 10.0, tb.center().y),
                Align2::RIGHT_CENTER,
                chrono::Local::now().format("%H:%M:%S UTC").to_string(),
                egui::FontId::new(18.0, egui::FontFamily::Monospace),
                theme.high(),
            );
        });
}
