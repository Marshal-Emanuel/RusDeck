use egui::{Pos2, Align2, FontId, Color32, Rect, Stroke, Painter};
use crate::app::AppState;
use crate::theme::Theme;
use super::draw_panel_frame;

const BAR_HEIGHT: f32 = 6.0;

pub fn draw_hardware(painter: &Painter, rect: Rect, state: &AppState, theme: &Theme) {
    draw_panel_frame(painter, rect, "HARDWARE", theme);

    let chamfer = 12.0;
    let label_x = rect.left() + chamfer + 4.0;
    let value_x = rect.right() - 16.0;
    let bar_max_width = rect.width() - 32.0;

    let mut y = rect.top() + 32.0;

    // CPU Row
    painter.text(
        Pos2::new(label_x, y),
        Align2::LEFT_TOP,
        "CPU",
        FontId::monospace(14.0),
        theme.high(),
    );
    painter.text(
        Pos2::new(value_x, y),
        Align2::RIGHT_TOP,
        format!("{:>5.1}%", state.system.cpu_usage),
        FontId::monospace(14.0),
        theme.full(),
    );
    y += 18.0;
    let cpu_pct = state.system.cpu_usage / 100.0;
    draw_segmented_bar(painter, Pos2::new(label_x, y), bar_max_width, cpu_pct, theme.accent, theme);
    y += BAR_HEIGHT + 6.0;

    // CPU secondary info row (frequency and temperature)
    let freq_text = format!("{:.2} GHz", state.system.cpu_freq_ghz);
    painter.text(
        Pos2::new(label_x, y),
        Align2::LEFT_TOP,
        freq_text,
        FontId::monospace(11.0),
        theme.dimmed(),
    );
    if let Some(temp) = state.system.cpu_temp_c {
        let temp_text = format!("{:.0}°C", temp);
        painter.text(
            Pos2::new(value_x, y),
            Align2::RIGHT_TOP,
            temp_text,
            FontId::monospace(11.0),
            theme.dimmed(),
        );
    }
    y += 18.0;

    // Memory Row
    painter.text(
        Pos2::new(label_x, y),
        Align2::LEFT_TOP,
        "MEM",
        FontId::monospace(14.0),
        theme.high(),
    );
    painter.text(
        Pos2::new(value_x, y),
        Align2::RIGHT_TOP,
        format!("{:.1}G / {:.0}G", state.system.mem_used_gb, state.system.mem_total_gb),
        FontId::monospace(14.0),
        theme.full(),
    );
    y += 18.0;
    let mem_pct = state.system.mem_used_gb / state.system.mem_total_gb;
    draw_segmented_bar(painter, Pos2::new(label_x, y), bar_max_width, mem_pct, theme.accent, theme);
    y += BAR_HEIGHT + 10.0;

    // Swap Row
    painter.text(
        Pos2::new(label_x, y),
        Align2::LEFT_TOP,
        "SWP",
        FontId::monospace(14.0),
        theme.high(),
    );
    painter.text(
        Pos2::new(value_x, y),
        Align2::RIGHT_TOP,
        format!("{:.1}G / {:.0}G", state.system.swap_used_gb, state.system.swap_total_gb),
        FontId::monospace(14.0),
        theme.dimmed(),
    );
    y += 18.0;
    let swap_pct = if state.system.swap_total_gb > 0.0 {
        state.system.swap_used_gb / state.system.swap_total_gb
    } else {
        0.0
    };
    draw_segmented_bar(painter, Pos2::new(label_x, y), bar_max_width, swap_pct, theme.dimmed(), theme);
    y += BAR_HEIGHT + 16.0;

    // CPU waveform history
    if !state.cpu_history.is_empty() {
        draw_waveform(painter, Rect::from_min_max(
            Pos2::new(rect.left() + 8.0, y),
            Pos2::new(rect.right() - 8.0, rect.bottom() - 12.0),
        ), &state.cpu_history, theme);
    }
}

fn draw_segmented_bar(painter: &Painter, pos: Pos2, max_width: f32, pct: f32, active_color: Color32, theme: &Theme) {
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
            theme.ghost() // Inactive segment uses alpha 8 (very faint neon placeholder)
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

fn draw_waveform(painter: &Painter, rect: Rect, history: &std::collections::VecDeque<f32>, theme: &Theme) {
    if history.len() < 2 { return; }

    // Left margin for graph scale labels
    let label_width = 30.0;
    let g_left = rect.left() + label_width;
    let g_rect = Rect::from_min_max(Pos2::new(g_left, rect.top() + 6.0), Pos2::new(rect.right() - 6.0, rect.bottom() - 6.0));

    // 1. Draw Axis Labels
    painter.text(
        Pos2::new(g_left - 6.0, g_rect.top()),
        Align2::RIGHT_CENTER,
        "100%",
        FontId::monospace(9.0),
        theme.low(),
    );
    painter.text(
        Pos2::new(g_left - 6.0, g_rect.center().y),
        Align2::RIGHT_CENTER,
        "50%",
        FontId::monospace(9.0),
        theme.dimmed(),
    );
    painter.text(
        Pos2::new(g_left - 6.0, g_rect.bottom()),
        Align2::RIGHT_CENTER,
        "0%",
        FontId::monospace(9.0),
        theme.low(),
    );

    // 2. Draw Gridlines
    painter.line_segment([Pos2::new(g_left, g_rect.top()), Pos2::new(g_rect.right(), g_rect.top())], Stroke::new(1.0, theme.low()));
    painter.line_segment([Pos2::new(g_left, g_rect.bottom()), Pos2::new(g_rect.right(), g_rect.bottom())], Stroke::new(1.0, theme.low()));
    
    // Mid dashed gridline (simulated segments)
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
    let n = history.len() as f32;
    let pts: Vec<Pos2> = history.iter().enumerate().map(|(i, &v)| {
        let x = g_rect.left() + (i as f32 / (n - 1.0)) * g_rect.width();
        let y = g_rect.bottom() - (v / 100.0).min(1.0) * g_rect.height();
        Pos2::new(x, y)
    }).collect();

    // 4. Draw shaded path under the line segment-by-segment (using convex quads/trapezoids to prevent triangulation bugs)
    for i in 0..pts.len() - 1 {
        let points = vec![
            pts[i],
            pts[i + 1],
            Pos2::new(pts[i + 1].x, g_rect.bottom()),
            Pos2::new(pts[i].x, g_rect.bottom()),
        ];
        painter.add(egui::Shape::convex_polygon(points, theme.ghost(), Stroke::NONE));
    }

    // 5. Draw the actual line
    for i in 0..pts.len() - 1 {
        painter.line_segment([pts[i], pts[i + 1]], Stroke::new(1.2, theme.mid()));
    }

    // 6. Find and draw the highest point (PEAK tracker)
    let mut max_val = 0.0_f32;
    let mut max_idx = 0;
    for (i, &v) in history.iter().enumerate() {
        if v > max_val {
            max_val = v;
            max_idx = i;
        }
    }

    if max_val > 0.0 {
        let mx = g_rect.left() + (max_idx as f32 / (n - 1.0)) * g_rect.width();
        let my = g_rect.bottom() - (max_val / 100.0).min(1.0) * g_rect.height();
        let peak_pos = Pos2::new(mx, my);

        // Neon dot
        painter.circle_filled(peak_pos, 3.0, theme.full());
        painter.circle_filled(peak_pos, 6.0, theme.faint()); // glow

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
            format!("MAX: {:.1}%", max_val),
            FontId::monospace(9.0),
            theme.high(),
        );
    }
}