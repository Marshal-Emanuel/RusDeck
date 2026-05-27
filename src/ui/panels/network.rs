use egui::{Pos2, Align2, FontId, Rect, Stroke, Painter};
use crate::app::AppState;
use crate::theme::Theme;
use super::draw_panel_frame;

const BAR_HEIGHT: f32 = 6.0;

pub fn draw_network(painter: &Painter, rect: Rect, state: &AppState, theme: &Theme) {
    draw_panel_frame(painter, rect, "NETWORK", theme);

    let chamfer = 12.0;
    let label_x = rect.left() + chamfer + 4.0;
    let value_x = rect.right() - 16.0;
    let bar_max_width = rect.width() - 32.0;

    let mut y = rect.top() + 32.0;

    // Interface and IP Address on the same row for compactness
    painter.text(
        Pos2::new(label_x, y),
        Align2::LEFT_TOP,
        &state.network.interface,
        FontId::monospace(14.0),
        theme.high(),
    );
    painter.text(
        Pos2::new(value_x, y),
        Align2::RIGHT_TOP,
        format!("IP: {}", state.network.ip),
        FontId::monospace(11.0),
        theme.dimmed(),
    );
    y += 22.0;

    // RX Row
    painter.text(
        Pos2::new(label_x, y),
        Align2::LEFT_TOP,
        "RX",
        FontId::monospace(13.0),
        theme.low(),
    );
    painter.text(
        Pos2::new(value_x, y),
        Align2::RIGHT_TOP,
        format_rate(state.network.rx_rate),
        FontId::monospace(13.0),
        theme.full(),
    );
    y += 18.0;
    draw_segmented_bar(painter, Pos2::new(label_x, y), bar_max_width, rate_pct(state.network.rx_rate), theme.accent, theme);
    y += BAR_HEIGHT + 8.0;

    // TX Row
    painter.text(
        Pos2::new(label_x, y),
        Align2::LEFT_TOP,
        "TX",
        FontId::monospace(13.0),
        theme.low(),
    );
    painter.text(
        Pos2::new(value_x, y),
        Align2::RIGHT_TOP,
        format_rate(state.network.tx_rate),
        FontId::monospace(13.0),
        theme.full(),
    );
    y += 18.0;
    draw_segmented_bar(painter, Pos2::new(label_x, y), bar_max_width, rate_pct(state.network.tx_rate), theme.accent, theme);
    y += BAR_HEIGHT + 14.0;

    // Network dual history graph
    if !state.network.rx_history.is_empty() {
        let graph_rect = Rect::from_min_max(
            Pos2::new(rect.left() + 8.0, y),
            Pos2::new(rect.right() - 8.0, rect.bottom() - 12.0),
        );
        draw_dual_waveform(painter, graph_rect, &state.network.rx_history, &state.network.tx_history, theme);
    }
}

fn format_rate(bytes_per_sec: f64) -> String {
    if bytes_per_sec >= 1_000_000.0 {
        format!("{:.1} MB/s", bytes_per_sec / 1_000_000.0)
    } else if bytes_per_sec >= 1_000.0 {
        format!("{:.1} KB/s", bytes_per_sec / 1_000.0)
    } else {
        format!("{:.0} B/s", bytes_per_sec)
    }
}

fn rate_pct(bytes_per_sec: f64) -> f32 {
    (bytes_per_sec / 10_000_000.0).min(1.0) as f32
}

fn draw_segmented_bar(painter: &Painter, pos: Pos2, max_width: f32, pct: f32, active_color: egui::Color32, theme: &Theme) {
    let segments = 24;
    let gap = 2.0;
    let seg_w = (max_width - (segments - 1) as f32 * gap) / segments as f32;
    let slant = 2.0;
    let h = BAR_HEIGHT;

    for i in 0..segments {
        let active = (i as f32 / segments as f32) < pct;
        let color = if active {
            active_color
        } else {
            theme.ghost() // Inactive segments are faint
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

fn draw_dual_waveform(painter: &Painter, rect: Rect, rx_history: &std::collections::VecDeque<f64>, tx_history: &std::collections::VecDeque<f64>, theme: &Theme) {
    let len = rx_history.len().min(tx_history.len());
    if len < 2 { return; }

    // Left margin for graph scale labels
    let label_width = 30.0;
    let g_left = rect.left() + label_width;
    let g_rect = Rect::from_min_max(Pos2::new(g_left, rect.top() + 6.0), Pos2::new(rect.right() - 6.0, rect.bottom() - 6.0));

    let max_val = rx_history.iter().chain(tx_history.iter()).cloned().fold(100_000.0_f64, f64::max);
    let n = len as f32;

    // 1. Draw Axis Labels
    painter.text(
        Pos2::new(g_left - 6.0, g_rect.top()),
        Align2::RIGHT_CENTER,
        format_rate_compact(max_val),
        FontId::monospace(8.0),
        theme.low(),
    );
    painter.text(
        Pos2::new(g_left - 6.0, g_rect.center().y),
        Align2::RIGHT_CENTER,
        format_rate_compact(max_val / 2.0),
        FontId::monospace(8.0),
        theme.dimmed(),
    );
    painter.text(
        Pos2::new(g_left - 6.0, g_rect.bottom()),
        Align2::RIGHT_CENTER,
        "0B",
        FontId::monospace(8.0),
        theme.low(),
    );

    // 2. Draw Gridlines
    painter.line_segment([Pos2::new(g_left, g_rect.top()), Pos2::new(g_rect.right(), g_rect.top())], Stroke::new(1.0, theme.low()));
    painter.line_segment([Pos2::new(g_left, g_rect.bottom()), Pos2::new(g_rect.right(), g_rect.bottom())], Stroke::new(1.0, theme.low()));
    
    // Mid dashed gridline
    let grid_segments = 15;
    let w = g_rect.width();
    let seg_w = w / (grid_segments as f32 * 2.0 - 1.0);
    for i in 0..grid_segments {
        let sx = g_left + (i as f32 * 2.0) * seg_w;
        painter.line_segment(
            [Pos2::new(sx, g_rect.center().y), Pos2::new(sx + seg_w, g_rect.center().y)],
            Stroke::new(1.0, theme.faint()),
        );
    }

    // 3. Compute points
    let rx_pts: Vec<Pos2> = rx_history.iter().take(len).enumerate().map(|(i, &v)| {
        let x = g_rect.left() + (i as f32 / (n - 1.0)) * g_rect.width();
        let y = g_rect.bottom() - (v / max_val).min(1.0) as f32 * g_rect.height();
        Pos2::new(x, y)
    }).collect();

    let tx_pts: Vec<Pos2> = tx_history.iter().take(len).enumerate().map(|(i, &v)| {
        let x = g_rect.left() + (i as f32 / (n - 1.0)) * g_rect.width();
        let y = g_rect.bottom() - (v / max_val).min(1.0) as f32 * g_rect.height();
        Pos2::new(x, y)
    }).collect();

    // 4. Draw shaded paths under the lines segment-by-segment (using convex quads/trapezoids to prevent triangulation bugs)
    // RX Shading
    for i in 0..rx_pts.len() - 1 {
        let points = vec![
            rx_pts[i],
            rx_pts[i + 1],
            Pos2::new(rx_pts[i + 1].x, g_rect.bottom()),
            Pos2::new(rx_pts[i].x, g_rect.bottom()),
        ];
        painter.add(egui::Shape::convex_polygon(points, theme.ghost(), Stroke::NONE));
    }

    // TX Shading (ultra faint secondary fill to avoid cluttering)
    for i in 0..tx_pts.len() - 1 {
        let points = vec![
            tx_pts[i],
            tx_pts[i + 1],
            Pos2::new(tx_pts[i + 1].x, g_rect.bottom()),
            Pos2::new(tx_pts[i].x, g_rect.bottom()),
        ];
        painter.add(egui::Shape::convex_polygon(points, theme.with_alpha(4), Stroke::NONE));
    }

    // 5. Draw the actual lines
    for i in 0..rx_pts.len() - 1 {
        painter.line_segment([rx_pts[i], rx_pts[i + 1]], Stroke::new(1.2, theme.full())); // RX
    }
    for i in 0..tx_pts.len() - 1 {
        painter.line_segment([tx_pts[i], tx_pts[i + 1]], Stroke::new(1.0, theme.mid())); // TX
    }

    // 6. Peak value tracker
    if max_val > 0.0 {
        let mut peak_val = 0.0_f64;
        let mut peak_idx = 0;
        let mut is_rx = true;
        for (i, &v) in rx_history.iter().take(len).enumerate() {
            if v > peak_val { peak_val = v; peak_idx = i; is_rx = true; }
        }
        for (i, &v) in tx_history.iter().take(len).enumerate() {
            if v > peak_val { peak_val = v; peak_idx = i; is_rx = false; }
        }

        let mx = g_rect.left() + (peak_idx as f32 / (n - 1.0)) * g_rect.width();
        let my = g_rect.bottom() - (peak_val / max_val).min(1.0) as f32 * g_rect.height();
        let peak_pos = Pos2::new(mx, my);

        // Peak Neon Dot
        let peak_color = if is_rx { theme.full() } else { theme.mid() };
        painter.circle_filled(peak_pos, 3.0, peak_color);
        painter.circle_filled(peak_pos, 6.0, theme.faint());

        // Dotted peak line
        let peak_stroke = Stroke::new(1.0, theme.dimmed());
        let mut sx = g_left;
        while sx < g_rect.right() {
            painter.line_segment(
                [Pos2::new(sx, my), Pos2::new((sx + 3.0).min(g_rect.right()), my)],
                peak_stroke,
            );
            sx += 6.0;
        }

        // Draw Peak value label text at the top-right of the dotted line
        painter.text(
            Pos2::new(g_rect.right() - 4.0, my - 3.0),
            Align2::RIGHT_BOTTOM,
            format!("MAX: {}", format_rate(peak_val)),
            FontId::monospace(8.0),
            theme.high(),
        );
    }

    // Legend on top
    painter.text(
        Pos2::new(g_rect.right(), g_rect.top() - 8.0),
        Align2::RIGHT_BOTTOM,
        format!("\u{25ac} RX  \u{25ac} TX"),
        FontId::monospace(9.0),
        theme.dimmed(),
    );
}

fn format_rate_compact(bytes_per_sec: f64) -> String {
    if bytes_per_sec >= 1_000_000.0 {
        format!("{:.0}M", bytes_per_sec / 1_000_000.0)
    } else if bytes_per_sec >= 1_000.0 {
        format!("{:.0}K", bytes_per_sec / 1_000.0)
    } else {
        format!("{:.0}B", bytes_per_sec)
    }
}