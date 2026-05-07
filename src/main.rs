mod app;
mod monitor;
mod theme;
mod ui;

use std::sync::{Arc, RwLock};
use app::AppState;
use theme::Theme;

fn main() -> eframe::Result<()> {
    let state = Arc::new(RwLock::new(AppState::new()));

    let state_monitor = Arc::clone(&state);
    std::thread::spawn(move || {
        monitor::start_monitor_thread(state_monitor);
    });

    eframe::run_native(
        "RusDeck",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_fullscreen(true)
                .with_decorations(false),
            ..Default::default()
        },
        Box::new(move |cc| {
            ui::setup_visuals(&cc.egui_ctx);
            Box::new(RusDeckApp {
                state,
                theme: Theme::default_white(),
            })
        }),
    )
}

struct RusDeckApp {
    state: Arc<RwLock<AppState>>,
    theme: Theme,
}

impl eframe::App for RusDeckApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let state = self.state.read().unwrap();
        ui::draw(ctx, &state, &self.theme);
    }
}