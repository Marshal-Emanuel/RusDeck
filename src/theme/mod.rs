use egui::Color32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeVariant {
    Default,
    Dracula,
    Nord,
    TokyoNight,
    Monokai,
    CatppuccinMocha,
    OneDark,
}

impl ThemeVariant {
    pub fn name(&self) -> &'static str {
        match self {
            ThemeVariant::Default => "Default",
            ThemeVariant::Dracula => "Dracula",
            ThemeVariant::Nord => "Nord",
            ThemeVariant::TokyoNight => "Tokyo Night",
            ThemeVariant::Monokai => "Monokai",
            ThemeVariant::CatppuccinMocha => "Catppuccin Mocha",
            ThemeVariant::OneDark => "One Dark",
        }
    }

    pub fn preview(&self) -> Color32 {
        match self {
            ThemeVariant::Default => Color32::from_rgb(214, 226, 230),
            ThemeVariant::Dracula => Color32::from_rgb(189, 147, 249),
            ThemeVariant::Nord => Color32::from_rgb(136, 192, 208),
            ThemeVariant::TokyoNight => Color32::from_rgb(122, 162, 247),
            ThemeVariant::Monokai => Color32::from_rgb(166, 226, 46),
            ThemeVariant::CatppuccinMocha => Color32::from_rgb(137, 180, 250),
            ThemeVariant::OneDark => Color32::from_rgb(97, 175, 239),
        }
    }
}

pub const ALL_THEMES: &[ThemeVariant] = &[
    ThemeVariant::Default,
    ThemeVariant::Dracula,
    ThemeVariant::Nord,
    ThemeVariant::TokyoNight,
    ThemeVariant::Monokai,
    ThemeVariant::CatppuccinMocha,
    ThemeVariant::OneDark,
];

#[derive(Clone)]
pub struct Theme {
    pub background: Color32,
    pub accent: Color32,
    pub terminal_text: Color32,
    pub terminal_bg: Color32,
}

impl Theme {
    pub fn from_variant(variant: ThemeVariant) -> Self {
        match variant {
            ThemeVariant::Default => Self {
                background: Color32::from_rgba_unmultiplied(10, 15, 13, 255),
                accent: Color32::from_rgba_unmultiplied(214, 226, 230, 255),
                terminal_text: Color32::from_rgba_unmultiplied(207, 221, 225, 255),
                terminal_bg: Color32::from_rgba_unmultiplied(2, 2, 2, 255),
            },
            ThemeVariant::Dracula => Self {
                background: Color32::from_rgba_unmultiplied(30, 30, 46, 255),
                accent: Color32::from_rgba_unmultiplied(189, 147, 249, 255),
                terminal_text: Color32::from_rgba_unmultiplied(248, 248, 242, 255),
                terminal_bg: Color32::from_rgba_unmultiplied(40, 42, 54, 255),
            },
            ThemeVariant::Nord => Self {
                background: Color32::from_rgba_unmultiplied(46, 52, 64, 255),
                accent: Color32::from_rgba_unmultiplied(136, 192, 208, 255),
                terminal_text: Color32::from_rgba_unmultiplied(216, 222, 233, 255),
                terminal_bg: Color32::from_rgba_unmultiplied(59, 66, 82, 255),
            },
            ThemeVariant::TokyoNight => Self {
                background: Color32::from_rgba_unmultiplied(26, 27, 38, 255),
                accent: Color32::from_rgba_unmultiplied(122, 162, 247, 255),
                terminal_text: Color32::from_rgba_unmultiplied(169, 177, 214, 255),
                terminal_bg: Color32::from_rgba_unmultiplied(36, 40, 59, 255),
            },
            ThemeVariant::Monokai => Self {
                background: Color32::from_rgba_unmultiplied(39, 40, 34, 255),
                accent: Color32::from_rgba_unmultiplied(166, 226, 46, 255),
                terminal_text: Color32::from_rgba_unmultiplied(248, 248, 242, 255),
                terminal_bg: Color32::from_rgba_unmultiplied(30, 31, 28, 255),
            },
            ThemeVariant::CatppuccinMocha => Self {
                background: Color32::from_rgba_unmultiplied(30, 30, 46, 255),
                accent: Color32::from_rgba_unmultiplied(137, 180, 250, 255),
                terminal_text: Color32::from_rgba_unmultiplied(205, 214, 244, 255),
                terminal_bg: Color32::from_rgba_unmultiplied(24, 24, 37, 255),
            },
            ThemeVariant::OneDark => Self {
                background: Color32::from_rgba_unmultiplied(30, 33, 39, 255),
                accent: Color32::from_rgba_unmultiplied(97, 175, 239, 255),
                terminal_text: Color32::from_rgba_unmultiplied(171, 178, 191, 255),
                terminal_bg: Color32::from_rgba_unmultiplied(40, 44, 52, 255),
            },
        }
    }

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
