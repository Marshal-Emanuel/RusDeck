use egui::{Pos2, Rect};

pub struct Layout {
    pub topbar: Rect,
    pub hardware: Rect,
    pub filesystem: Rect,
    pub network: Rect,
    pub processes: Rect,
    pub system_logs: Rect,
    pub load_history: Rect,
}

impl Layout {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        let pad = 8.0;
        let gap = 6.0;
        let topbar_h = 32.0;
        let bottom_h = 130.0;
        let side_w = 210.0;
        let content_top = topbar_h + pad;
        let content_bottom = screen_height - pad;

        let mid_left = pad + side_w + gap;
        let mid_right = screen_width - pad - side_w - gap;
        let mid_width = mid_right - mid_left;

        let main_top = content_top + gap;
        let main_bottom = content_bottom - bottom_h - gap - gap;
        let main_height = main_bottom - main_top;

        let bottom_top = content_bottom - bottom_h;

        Self {
            topbar: Rect::from_min_max(
                Pos2::new(pad, pad),
                Pos2::new(screen_width - pad, pad + topbar_h),
            ),
            hardware: Rect::from_min_max(
                Pos2::new(pad, main_top),
                Pos2::new(pad + side_w, main_bottom),
            ),
            filesystem: Rect::from_min_max(
                Pos2::new(mid_left, main_top),
                Pos2::new(mid_right, main_bottom),
            ),
            network: Rect::from_min_max(
                Pos2::new(screen_width - pad - side_w, main_top),
                Pos2::new(screen_width - pad, main_bottom - gap - main_height / 2.0),
            ),
            processes: Rect::from_min_max(
                Pos2::new(screen_width - pad - side_w, main_bottom - main_height / 2.0),
                Pos2::new(screen_width - pad, main_bottom),
            ),
            system_logs: Rect::from_min_max(
                Pos2::new(pad, bottom_top),
                Pos2::new(pad + mid_width * 0.4, content_bottom),
            ),
            load_history: Rect::from_min_max(
                Pos2::new(mid_left + mid_width * 0.4 + gap, bottom_top),
                Pos2::new(mid_right, content_bottom),
            ),
        }
    }
}