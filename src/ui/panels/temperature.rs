use egui::{Pos2, Align2, FontId, Rect, Stroke, Painter};
use crate::app::AppState;
use crate::theme::Theme;
use super::draw_panel_frame;

pub fn draw_temperature(painter: &Painter, rect: Rect, state: &AppState, theme: &Theme) {
    draw_panel_frame(painter, rect, "TEMPERATURE", theme);

    let chamfer = 12.0;
    let label_x = rect.left() + chamfer + 4.0;
    let value_x = rect.right() - 16.0;
    let bar_max_width = rect.width() - 32.0;

    let mut y = rect.top() + 32.0;

    if let Some(temp) = state.system.cpu_temp_c {
        painter.text(
            Pos2::new(label_x, y),
            Align2::LEFT_TOP,
            "CPU",
            FontId::monospace(16.0),
            theme.high(),
        );
        painter.text(
            Pos2::new(value_x, y),
            Align2::RIGHT_TOP,
            format!("{:.0}°C", temp),
            FontId::monospace(20.0),
            theme.full(),
        );
        y += 24.0;

        // Temperature bar (0-100°C range)
        let temp_pct = (temp / 100.0).min(1.0);
        let bar_color = if temp >= 80.0 {
            Color32::from_rgb(255, 80, 80)
        } else if temp >= 60.0 {
            Color32::from_rgb(255, 200, 80)
        } else {
            theme.accent
        };
        draw_segmented_bar(painter, Pos2::new(label_x, y), bar_max_width, temp_pct, bar_color, theme);
    } else {
        painter.text(
            Pos2::new(label_x, y),
            Align2::LEFT_TOP,
            "CPU",
            FontId::monospace(16.0),
            theme.high(),
        );
        painter.text(
            Pos2::new(value_x, y),
            Align2::RIGHT_TOP,
            "N/A",
            FontId::monospace(20.0),
            theme.dimmed(),
        );
    }
}

fn draw_segmented_bar(painter: &Painter, pos: Pos2, max_width: f32, pct: f32, active_color: egui::Color32, theme: &Theme) {
    let segments = 24;
    let gap = 2.0;
    let seg_w = (max_width - (segments - 1) as f32 * gap) / segments as f32;
    let slant = 2.0;
    let h = 8.0;

    for i in 0..segments {
        let active = (i as f32 / segments as f32) < pct;
        let color = if active {
            active_color
        } else {
            theme.ghost()
        };
        let sx = pos.x + i as f32 * (seg_w + gap);
        let points = vec![
            Pos2::new(sx + slant, pos.y),
            Pos2::new(sx + seg_w, pos.y),
            Pos2::new(sx + seg_w - slant, pos.y + h),
            Pos2::new(sx, pos.y + h),
        ];
        painter.add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
    }
}

use egui::Color32;