use eframe::wgpu::{
    BindGroupLayout, BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState,
    DepthStencilState, Device, FragmentState, FrontFace, MultisampleState,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology,
    RenderPipeline, RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource,
    StencilState, TextureFormat, VertexBufferLayout, VertexState, wgc::device,
};

pub struct PipelineBuilder<'a> {
    device: &'a Device,
    bind_group_layouts: &'a [&'a BindGroupLayout],

    module: Option<ShaderModule>,
    vertex: Option<&'a str>,
    fragment: Option<&'a str>,
    primitive: Option<PrimitiveState>,

    vertex_buffers: Vec<VertexBufferLayout<'a>>,

    depth_stencil: Option<DepthStencilState>,
    color_targets: Vec<Option<ColorTargetState>>,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self {
            device,
            bind_group_layouts: &[],
            module: None,
            primitive: None,
            vertex: None,
            fragment: None,
            vertex_buffers: Vec::new(),
            color_targets: Vec::new(),
            depth_stencil: None,
        }
    }
    pub fn bind_group_layout(mut self, bind_group_layout: &'a [&'a BindGroupLayout]) -> Self {
        self.bind_group_layouts = bind_group_layout;

        self
    }
    pub fn shader(mut self, module: &'static str, label: &'a str) -> Self {
        self.module = Some(self.device.create_shader_module(ShaderModuleDescriptor {
            label: Some(label),
            source: ShaderSource::Wgsl(module.into()),
        }));

        self
    }
    pub fn vertex_entry(mut self, entry: &'a str) -> Self {
        self.vertex = Some(entry);
        self
    }
    pub fn fragment_entry(mut self, entry: &'a str) -> Self {
        self.fragment = Some(entry);
        self
    }

    pub fn primitive(mut self, topology: PrimitiveTopology) -> Self {
        self.primitive = Some(PrimitiveState {
            topology: topology,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: eframe::wgpu::PolygonMode::Fill,
            conservative: false,
        });

        self
    }
    pub fn vertex_buffers(mut self, vertex_buffers: Vec<VertexBufferLayout<'a>>) -> Self {
        self.vertex_buffers = vertex_buffers;

        self
    }
    pub fn depth(mut self, format: TextureFormat) -> Self {
        self.depth_stencil = Some(DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Less,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        });
        self
    }

    pub fn color_format(mut self, format: TextureFormat) -> Self {
        self.color_targets = vec![Some(ColorTargetState {
            format,
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: ColorWrites::ALL,
        })];
        self
    }
    pub fn build(self, label: &'a str) -> RenderPipeline {
        let module = self.module.unwrap();
        let layout = self
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &self.bind_group_layouts,
                push_constant_ranges: &[],
            });
        self.device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&layout),
                vertex: VertexState {
                    module: &module,
                    entry_point: self.vertex,
                    compilation_options: PipelineCompilationOptions::default(),
                    buffers: &self.vertex_buffers,
                },
                primitive: self.primitive.unwrap(),
                depth_stencil: self.depth_stencil,
                multisample: MultisampleState::default(),
                fragment: Some(FragmentState {
                    module: &module,
                    entry_point: self.fragment,
                    compilation_options: PipelineCompilationOptions::default(),
                    targets: &self.color_targets,
                }),
                multiview: None,
                cache: None,
            })
    }
}
