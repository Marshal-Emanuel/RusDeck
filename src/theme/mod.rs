use egui::Color32;

#[derive(Clone)]
pub struct Theme {
    pub background: Color32,
    pub accent: Color32,
    pub terminal_text: Color32,
    pub terminal_bg: Color32,
}

impl Theme {
    pub fn full(&self) -> Color32 {
        self.with_alpha(255)
    }

    pub fn high(&self) -> Color32 {
        self.with_alpha(191)
    }

    pub fn mid(&self) -> Color32 {
        self.with_alpha(140)
    }

    pub fn low(&self) -> Color32 {
        self.with_alpha(102)
    }

    pub fn dimmed(&self) -> Color32 {
        self.with_alpha(64)
    }

    pub fn faint(&self) -> Color32 {
        self.with_alpha(20)
    }

    pub fn ghost(&self) -> Color32 {
        self.with_alpha(8)
    }

    pub fn with_alpha(&self, a: u8) -> Color32 {
        let [r, g, b, _] = self.accent.to_array();
        Color32::from_rgba_unmultiplied(r, g, b, a)
    }
}

impl Theme {
    pub fn default_white() -> Self {
        Self {
            background: Color32::from_rgba_unmultiplied(10, 15, 13, 255),
            accent: Color32::from_rgba_unmultiplied(214, 226, 230, 255),
            terminal_text: Color32::from_rgba_unmultiplied(207, 221, 225, 255),
            terminal_bg: Color32::from_rgba_unmultiplied(2, 2, 2, 255),
        }
    }

    pub fn tron() -> Self {
        Self {
            background: Color32::from_rgba_unmultiplied(10, 15, 13, 255),
            accent: Color32::from_rgba_unmultiplied(0, 171, 255, 255),
            terminal_text: Color32::from_rgba_unmultiplied(207, 221, 225, 255),
            terminal_bg: Color32::from_rgba_unmultiplied(2, 2, 2, 255),
        }
    }
}