use egui::{Pos2, Align2, FontId, Rect, Painter};
use crate::app::AppState;
use crate::theme::Theme;
use super::draw_panel_frame;

pub fn draw_system_logs(painter: &Painter, rect: Rect, state: &AppState, theme: &Theme) {
    draw_panel_frame(painter, rect, "SYSTEM_LOGS", theme);

    let chamfer = 10.0;
    let text_x = rect.left() + chamfer + 4.0;
    let _max_w = rect.width() - chamfer * 2.0 - 8.0;

    let mut y = rect.top() + 22.0;
    let row_h = 14.0;
    let max_rows = ((rect.height() - 30.0) / row_h) as usize;

    for (i, log) in state.logs.iter().take(max_rows).enumerate() {
        let base_alpha = (max_rows - i) as f32 / max_rows as f32 * 0.8 + 0.2;
        let ts_alpha = base_alpha * 0.6;
        let msg_alpha = base_alpha;

        painter.text(
            Pos2::new(text_x, y),
            Align2::LEFT_TOP,
            &log.timestamp,
            FontId::monospace(7.0),
            theme.with_alpha((ts_alpha * 200.0) as u8),
        );
        painter.text(
            Pos2::new(text_x + 10.0, y),
            Align2::LEFT_TOP,
            &log.message,
            FontId::monospace(7.0),
            theme.with_alpha((msg_alpha * 180.0) as u8),
        );
        y += row_h;
    }
}