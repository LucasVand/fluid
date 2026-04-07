use eframe::wgpu::{CommandEncoder, CommandEncoderDescriptor, Device};

pub struct CommandEncoderBuilder<'a> {
    device: &'a Device,
    label: Option<&'a str>,
}

impl<'a> CommandEncoderBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self {
            device,
            label: None,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn build(self) -> CommandEncoder {
        self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: self.label,
        })
    }
}
