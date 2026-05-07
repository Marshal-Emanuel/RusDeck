use egui::{Pos2, Align2, FontId, Rect, Stroke, Painter};
use crate::app::AppState;
use crate::theme::Theme;
use super::draw_panel_frame;

const BAR_HEIGHT: f32 = 3.0;

pub fn draw_network(painter: &Painter, rect: Rect, state: &AppState, theme: &Theme) {
    draw_panel_frame(painter, rect, "NETWORK", theme);

    let chamfer = 10.0;
    let label_x = rect.left() + chamfer + 4.0;
    let value_x = rect.right() - 4.0;
    let bar_max_width = rect.width() - 20.0;

    let mut y = rect.top() + 25.0;
    let line_h = 16.0;

    painter.text(
        Pos2::new(label_x, y),
        Align2::LEFT_TOP,
        &state.network.interface,
        FontId::monospace(10.0),
        theme.high(),
    );
    y += line_h;

    painter.text(
        Pos2::new(label_x, y),
        Align2::LEFT_TOP,
        format!("IP {}", state.network.ip),
        FontId::monospace(8.0),
        theme.dimmed(),
    );
    y += line_h + 4.0;

    painter.text(
        Pos2::new(label_x, y),
        Align2::LEFT_TOP,
        "RX",
        FontId::monospace(8.0),
        theme.low(),
    );
    painter.text(
        Pos2::new(value_x, y),
        Align2::RIGHT_TOP,
        format_rate(state.network.rx_rate),
        FontId::monospace(9.0),
        theme.full(),
    );
    y += 10.0;
    draw_parallelogram_bar(painter, Pos2::new(label_x, y), bar_max_width * rate_pct(state.network.rx_rate), theme.full());
    y += line_h;

    painter.text(
        Pos2::new(label_x, y),
        Align2::LEFT_TOP,
        "TX",
        FontId::monospace(8.0),
        theme.low(),
    );
    painter.text(
        Pos2::new(value_x, y),
        Align2::RIGHT_TOP,
        format_rate(state.network.tx_rate),
        FontId::monospace(9.0),
        theme.full(),
    );
    y += 10.0;
    draw_parallelogram_bar(painter, Pos2::new(label_x, y), bar_max_width * rate_pct(state.network.tx_rate), theme.full());
    y += line_h + 6.0;

    if !state.network.rx_history.is_empty() {
        let graph_rect = Rect::from_min_max(
            Pos2::new(rect.left() + chamfer, y),
            Pos2::new(rect.right() - chamfer, rect.bottom() - chamfer),
        );
        draw_dual_waveform(painter, graph_rect, &state.network.rx_history, &state.network.tx_history, theme);
    }
}

fn format_rate(bytes_per_sec: f64) -> String {
    if bytes_per_sec >= 1_000_000.0 {
        format!("{:.1}MB/s", bytes_per_sec / 1_000_000.0)
    } else if bytes_per_sec >= 1_000.0 {
        format!("{:.1}KB/s", bytes_per_sec / 1_000.0)
    } else {
        format!("{:.0}B/s", bytes_per_sec)
    }
}

fn rate_pct(bytes_per_sec: f64) -> f32 {
    (bytes_per_sec / 10_000_000.0).min(1.0) as f32
}

fn draw_parallelogram_bar(painter: &Painter, pos: Pos2, width: f32, color: egui::Color32) {
    if width < 2.0 { return; }
    let slant = 2.0;
    let points = vec![
        Pos2::new(pos.x + slant, pos.y),
        Pos2::new(pos.x + width, pos.y),
        Pos2::new(pos.x + width - slant, pos.y + BAR_HEIGHT),
        Pos2::new(pos.x, pos.y + BAR_HEIGHT),
    ];
    painter.add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
}

fn draw_dual_waveform(painter: &Painter, rect: Rect, rx_history: &std::collections::VecDeque<f64>, tx_history: &std::collections::VecDeque<f64>, theme: &Theme) {
    let len = rx_history.len().min(tx_history.len());
    if len < 2 { return; }

    let max_val = rx_history.iter().chain(tx_history.iter()).cloned().fold(1.0_f64, f64::max);
    let n = len as f32;

    let rx_pts: Vec<Pos2> = rx_history.iter().take(len as usize).enumerate().map(|(i, &v)| {
        let x = rect.left() + (i as f32 / (n - 1.0)) * rect.width();
        let y = rect.bottom() - (v / max_val).min(1.0) as f32 * rect.height();
        Pos2::new(x, y)
    }).collect();

    let tx_pts: Vec<Pos2> = tx_history.iter().take(len as usize).enumerate().map(|(i, &v)| {
        let x = rect.left() + (i as f32 / (n - 1.0)) * rect.width();
        let y = rect.bottom() - (v / max_val).min(1.0) as f32 * rect.height();
        Pos2::new(x, y)
    }).collect();

    for i in 0..rx_pts.len() - 1 {
        painter.line_segment([rx_pts[i], rx_pts[i + 1]], Stroke::new(1.0, theme.full()));
    }
    for i in 0..tx_pts.len() - 1 {
        painter.line_segment([tx_pts[i], tx_pts[i + 1]], Stroke::new(1.0, theme.mid()));
    }

    painter.text(
        Pos2::new(rect.right(), rect.top() - 4.0),
        Align2::RIGHT_BOTTOM,
        format!("\u{2016}RX \u{2014}TX"),
        FontId::monospace(7.0),
        theme.dimmed(),
    );
}