use eframe::wgpu::{
    BindGroupLayout, ComputePipeline, ComputePipelineDescriptor, Device, PipelineCompilationOptions,
    PipelineLayoutDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource,
};

pub struct ComputePipelineBuilder<'a> {
    device: &'a Device,
    bind_group_layouts: &'a [&'a BindGroupLayout],
    module: Option<ShaderModule>,
    entry_point: Option<&'a str>,
}

impl<'a> ComputePipelineBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self {
            device,
            bind_group_layouts: &[],
            module: None,
            entry_point: None,
        }
    }

    pub fn bind_group_layout(mut self, bind_group_layouts: &'a [&'a BindGroupLayout]) -> Self {
        self.bind_group_layouts = bind_group_layouts;
        self
    }

    pub fn shader(mut self, wgsl_source: &'static str, label: &'a str) -> Self {
        self.module = Some(self.device.create_shader_module(ShaderModuleDescriptor {
            label: Some(label),
            source: ShaderSource::Wgsl(wgsl_source.into()),
        }));
        self
    }

    pub fn entry_point(mut self, entry: &'a str) -> Self {
        self.entry_point = Some(entry);
        self
    }

    pub fn build(self, label: &'a str) -> ComputePipeline {
        let module = self.module.expect(
            "ComputePipelineBuilder: shader module not set. Call .shader(wgsl_code, label) before build()",
        );
        let entry_point = self.entry_point.expect(
            "ComputePipelineBuilder: entry point not set. Call .entry_point(entry) before build()",
        );

        let layout = self
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: self.bind_group_layouts,
                push_constant_ranges: &[],
            });

        self.device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some(label),
            layout: Some(&layout),
            module: &module,
            entry_point: Some(entry_point),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        })
    }
}
