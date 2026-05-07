use egui::{Pos2, Align2, FontId, Rect, Stroke, Painter};
use crate::app::AppState;
use crate::theme::Theme;
use super::draw_panel_frame;

pub fn draw_load_history(painter: &Painter, rect: Rect, state: &AppState, theme: &Theme) {
    draw_panel_frame(painter, rect, "SYS_LOAD_HISTORY", theme);

    let chamfer = 10.0;
    let value_x = rect.right() - chamfer - 4.0;

    let current = state.system.cpu_load;
    painter.text(
        Pos2::new(rect.left() + chamfer + 4.0, rect.top() + 25.0),
        Align2::LEFT_TOP,
        "LOAD AVG",
        FontId::monospace(8.0),
        theme.low(),
    );
    painter.text(
        Pos2::new(value_x, rect.top() + 25.0),
        Align2::RIGHT_TOP,
        format!("{:.2}", current),
        FontId::monospace(11.0),
        theme.full(),
    );

    if !state.load_history.is_empty() {
        let graph_rect = Rect::from_min_max(
            Pos2::new(rect.left() + chamfer, rect.top() + 50.0),
            Pos2::new(rect.right() - chamfer, rect.bottom() - chamfer),
        );

        let max_val = state.load_history.iter().cloned().fold(0.0_f32, f32::max).max(1.0);
        let n = state.load_history.len() as f32;

        if n >= 2.0 {
            let pts: Vec<Pos2> = state.load_history.iter().enumerate().map(|(i, &v)| {
                let x = graph_rect.left() + (i as f32 / (n - 1.0)) * graph_rect.width();
                let y = graph_rect.bottom() - (v / max_val).min(1.0) * graph_rect.height();
                Pos2::new(x, y)
            }).collect();

            for i in 0..pts.len() - 1 {
                painter.line_segment([pts[i], pts[i + 1]], Stroke::new(1.5, theme.mid()));
            }

            let peak_line_y = graph_rect.top();
            painter.line_segment(
                [Pos2::new(graph_rect.left(), peak_line_y), Pos2::new(graph_rect.right(), peak_line_y)],
                Stroke::new(0.5, theme.dimmed()),
            );
            painter.text(
                Pos2::new(graph_rect.right(), peak_line_y + 1.0),
                Align2::RIGHT_TOP,
                format!("{:.1}", max_val),
                FontId::monospace(7.0),
                theme.dimmed(),
            );
        }
    }
}