use egui::{Pos2, Align2, FontId, Color32, Rect, Stroke, Painter};
use crate::app::AppState;
use crate::theme::Theme;
use super::draw_panel_frame;

const BAR_HEIGHT: f32 = 4.0;

pub fn draw_hardware(painter: &Painter, rect: Rect, state: &AppState, theme: &Theme) {
    draw_panel_frame(painter, rect, "HARDWARE", theme);

    let chamfer = 10.0;
    let label_x = rect.left() + chamfer + 4.0;
    let value_x = rect.right() - 50.0;
    let bar_max_width = rect.width() - 70.0;

    let mut y = rect.top() + 40.0;
    let line_h = 30.0;

    // CPU
    painter.text(
        Pos2::new(label_x, y),
        Align2::LEFT_TOP,
        "CPU",
        FontId::monospace(15.0),
        theme.low(),
    );
    let cpu_pct = state.system.cpu_usage / 100.0;
    draw_parallelogram_bar(painter, Pos2::new(label_x, y + 16.0), bar_max_width * cpu_pct, theme.full());
    painter.text(
        Pos2::new(value_x, y),
        Align2::RIGHT_TOP,
        format!("{:>5.1}%", state.system.cpu_usage),
        FontId::monospace(16.0),
        theme.full(),
    );
    y += line_h + 16.0;

    // CPU freq & temp
    let freq_text = format!("{:.2}GHZ", state.system.cpu_freq_ghz);
    painter.text(
        Pos2::new(label_x, y),
        Align2::LEFT_TOP,
        freq_text,
        FontId::monospace(15.0),
        theme.dimmed(),
    );
    if let Some(temp) = state.system.cpu_temp_c {
        let temp_text = format!("{:.0}C", temp);
        painter.text(
            Pos2::new(rect.right() - 45.0, y),
            Align2::RIGHT_TOP,
            temp_text,
            FontId::monospace(15.0),
            theme.dimmed(),
        );
    }
    y += line_h;

    // Memory
    painter.text(
        Pos2::new(label_x, y),
        Align2::LEFT_TOP,
        "MEM",
        FontId::monospace(15.0),
        theme.low(),
    );
    let mem_pct = state.system.mem_used_gb / state.system.mem_total_gb;
    draw_parallelogram_bar(painter, Pos2::new(label_x, y + 16.0), bar_max_width * mem_pct, theme.full());
    let mem_text = format!("{:.1}G/{:.0}G", state.system.mem_used_gb, state.system.mem_total_gb);
    painter.text(
        Pos2::new(value_x, y),
        Align2::RIGHT_TOP,
        mem_text,
        FontId::monospace(16.0),
        theme.full(),
    );
    y += line_h + 16.0;

    // Swap
    painter.text(
        Pos2::new(label_x, y),
        Align2::LEFT_TOP,
        "SWP",
        FontId::monospace(15.0),
        theme.low(),
    );
    let swap_pct = if state.system.swap_total_gb > 0.0 {
        state.system.swap_used_gb / state.system.swap_total_gb
    } else {
        0.0
    };
    draw_parallelogram_bar(painter, Pos2::new(label_x, y + 16.0), bar_max_width * swap_pct, theme.dimmed());
    let swap_text = format!("{:.1}G/{:.0}G", state.system.swap_used_gb, state.system.swap_total_gb);
    painter.text(
        Pos2::new(value_x, y),
        Align2::RIGHT_TOP,
        swap_text,
        FontId::monospace(16.0),
        theme.dimmed(),
    );
    y += line_h + 24.0;

    // CPU waveform history
    if !state.cpu_history.is_empty() {
        draw_waveform(painter, Rect::from_min_max(
            Pos2::new(rect.left() + chamfer, y),
            Pos2::new(rect.right() - chamfer, rect.bottom() - chamfer),
        ), &state.cpu_history, theme.mid());
    }
}

fn draw_parallelogram_bar(painter: &Painter, pos: Pos2, width: f32, color: Color32) {
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

fn draw_waveform(painter: &Painter, rect: Rect, history: &std::collections::VecDeque<f32>, color: Color32) {
    if history.len() < 2 { return; }
    let n = history.len() as f32;
    let pts: Vec<Pos2> = history.iter().enumerate().map(|(i, &v)| {
        let x = rect.left() + (i as f32 / (n - 1.0)) * rect.width();
        let y = rect.bottom() - (v / 100.0).min(1.0) * rect.height();
        Pos2::new(x, y)
    }).collect();
    for i in 0..pts.len() - 1 {
        painter.line_segment([pts[i], pts[i + 1]], Stroke::new(1.5, color));
    }
}