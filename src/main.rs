use eframe::egui::{Pos2, Rect};
use fluid::fluid_app::FluidApp;

fn main() -> Result<(), eframe::Error> {
    let initial_size = Rect::from_points(&[Pos2::new(0.0, 0.0), Pos2::new(1200.0, 700.0)]);

    let opts = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([initial_size.size().x, initial_size.size().y]),
        ..Default::default()
    };

    eframe::run_native(
        "Fluid",
        opts,
        Box::new(|cc| Ok(Box::new(FluidApp::new(cc, initial_size)))),
    )
}
