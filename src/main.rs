use std::sync::Arc;

use eframe::{
    egui::{Pos2, Rect},
    egui_wgpu::{WgpuConfiguration, WgpuSetup, WgpuSetupCreateNew},
    wgpu::{DeviceDescriptor, Features, Limits},
};
use fluid::fluid_app::FluidApp;

fn main() -> Result<(), eframe::Error> {
    let initial_size = Rect::from_points(&[Pos2::new(0.0, 0.0), Pos2::new(1200.0, 700.0)]);

    let opts = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: WgpuConfiguration {
            wgpu_setup: WgpuSetup::CreateNew(WgpuSetupCreateNew {
                device_descriptor: Arc::new(|_a| {
                    let features = Features::PUSH_CONSTANTS;
                    DeviceDescriptor {
                        required_limits: Limits {
                            max_push_constant_size: 32,
                            ..Default::default()
                        },
                        label: Some("Custom Descripter"),
                        required_features: features,
                        trace: eframe::wgpu::Trace::Off,
                        ..Default::default()
                    }
                }),
                ..Default::default()
            }),
            ..Default::default()
        },
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
