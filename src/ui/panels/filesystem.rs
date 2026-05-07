use egui::{Pos2, Align2, FontId, Rect, Stroke, Painter};
use crate::app::AppState;
use crate::theme::Theme;
use super::draw_panel_frame;

const BAR_HEIGHT: f32 = 5.0;

pub fn draw_filesystem(painter: &Painter, rect: Rect, state: &AppState, theme: &Theme) {
    draw_panel_frame(painter, rect, "FILESYSTEM", theme);

    let chamfer = 10.0;
    let label_x = rect.left() + chamfer + 4.0;
    let value_x = rect.right() - chamfer - 4.0;
    let bar_max_width = rect.width() - 20.0;
    let bar_x = label_x;

    let mut y = rect.top() + 28.0;
    let line_h = 20.0;

    let used = state.system.storage_used_gb;
    let total = state.system.storage_total_gb;
    let pct = if total > 0.0 { used / total } else { 0.0 };

    painter.text(
        Pos2::new(label_x, y),
        Align2::LEFT_TOP,
        "ROOT",
        FontId::monospace(9.0),
        theme.low(),
    );
    painter.text(
        Pos2::new(value_x, y),
        Align2::RIGHT_TOP,
        format!("{:.0}G/{:.0}G", used, total),
        FontId::monospace(10.0),
        theme.full(),
    );
    y += 12.0;

    draw_parallelogram_bar(painter, Pos2::new(bar_x, y), bar_max_width * pct, bar_pct_color(pct, theme));
    y += BAR_HEIGHT + 6.0;

    painter.text(
        Pos2::new(label_x, y),
        Align2::LEFT_TOP,
        format!("{:.0}% used", pct * 100.0),
        FontId::monospace(8.0),
        theme.dimmed(),
    );
    y += line_h + 6.0;

    let free = total - used;
    painter.text(
        Pos2::new(label_x, y),
        Align2::LEFT_TOP,
        format!("{:.0}G free", free),
        FontId::monospace(8.0),
        theme.dimmed(),
    );
}

fn bar_pct_color(pct: f32, theme: &Theme) -> egui::Color32 {
    if pct > 0.9 {
        egui::Color32::from_rgba_unmultiplied(255, 80, 80, 255)
    } else if pct > 0.75 {
        egui::Color32::from_rgba_unmultiplied(255, 180, 60, 255)
    } else {
        theme.full()
    }
}

fn draw_parallelogram_bar(painter: &Painter, pos: Pos2, width: f32, color: egui::Color32) {
    if width < 2.0 { return; }
    let slant = 3.0;
    let points = vec![
        Pos2::new(pos.x + slant, pos.y),
        Pos2::new(pos.x + width, pos.y),
        Pos2::new(pos.x + width - slant, pos.y + BAR_HEIGHT),
        Pos2::new(pos.x, pos.y + BAR_HEIGHT),
    ];
    painter.add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
}