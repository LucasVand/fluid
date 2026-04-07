use eframe::wgpu::{CommandEncoder, ComputePass, ComputePassDescriptor};

pub struct ComputePassBuilder<'a> {
    label: Option<&'a str>,
}

impl<'a> ComputePassBuilder<'a> {
    pub fn new() -> Self {
        Self { label: None }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn build<'b>(self, encoder: &'b mut CommandEncoder) -> ComputePass<'b> {
        encoder.begin_compute_pass(&ComputePassDescriptor {
            label: self.label,
            timestamp_writes: None,
        })
    }
}

impl<'a> Default for ComputePassBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}
