use crate::renderer::utils::{
    BindGroupBuilder, BindGroupLayoutBuilder, BufferBuilder, CommandEncoderBuilder,
    ComputePipelineBuilder,
};
use bytemuck::{Pod, Zeroable};
use eframe::wgpu::{wgt::PollType, *};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
struct CellRange {
    start: u32,
    end: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
struct SpatialLookupEntry {
    cell_key: u32,
    particle_idx: u32,
}

pub struct IndirectStage {
    pub pipeline: ComputePipeline,
    pub bind_group: BindGroup,
    pub device: Device,
    pub spatial_lookup_buffer: Buffer,
    pub cell_ranges_buffer: Buffer,
    pub indirect_buffer: Buffer,
    pub particle_count: usize,
}

impl IndirectStage {
    pub fn create(
        device: &Device,
        spatial_lookup_buffer: &Buffer,
        cell_ranges_buffer: &Buffer,
        indirect_buffer: &Buffer,
        particle_count: usize,
    ) -> Self {
        let bgl = BindGroupLayoutBuilder::new(device)
            .buffer(0, ShaderStages::COMPUTE, true)
            .buffer(1, ShaderStages::COMPUTE, false)
            .buffer(2, ShaderStages::COMPUTE, false)
            .build("Indirect Bind Group");

        let pipeline = ComputePipelineBuilder::new(device)
            .shader(
                include_str!("../../../shaders/calculate_indirect.wgsl"),
                "Indirect Shader",
            )
            .bind_group_layout(&[&bgl])
            .entry_point("main")
            .build("Indirect Pipeline");

        let bind_group = BindGroupBuilder::new(device, &bgl)
            .buffer(0, spatial_lookup_buffer)
            .buffer(1, cell_ranges_buffer)
            .buffer(2, indirect_buffer)
            .build("Indirect Bind Group");

        Self {
            pipeline,
            bind_group,
            device: device.clone(),
            spatial_lookup_buffer: spatial_lookup_buffer.clone(),
            cell_ranges_buffer: cell_ranges_buffer.clone(),
            indirect_buffer: indirect_buffer.clone(),
            particle_count,
        }
    }

    pub fn execute(&self, pass: &mut ComputePass) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.dispatch_workgroups(1, 1, 1);
    }

    pub fn debug_print_ranges(&self, queue: &Queue) {
        // Create staging buffers
        let cell_ranges_staging = BufferBuilder::new(&self.device)
            .size(self.cell_ranges_buffer.size())
            .usages(BufferUsages::COPY_DST | BufferUsages::MAP_READ)
            .build("Cell Ranges Staging");

        let size = std::mem::size_of::<u32>() as u64;
        let indirect_staging = BufferBuilder::new(&self.device)
            .size(size * 4)
            .usages(BufferUsages::COPY_DST | BufferUsages::MAP_READ)
            .build("Indirect Staging");

        // Copy buffers
        let mut encoder = CommandEncoderBuilder::new(&self.device).build();
        encoder.copy_buffer_to_buffer(
            &self.cell_ranges_buffer,
            0,
            &cell_ranges_staging,
            0,
            self.cell_ranges_buffer.size(),
        );
        encoder.copy_buffer_to_buffer(&self.indirect_buffer, 0, &indirect_staging, 0, size * 4);
        queue.submit(std::iter::once(encoder.finish()));

        // Map and read
        let cell_ranges_slice = cell_ranges_staging.slice(..);
        let indirect_slice = indirect_staging.slice(..);

        cell_ranges_slice.map_async(MapMode::Read, |_| {});
        indirect_slice.map_async(MapMode::Read, |_| {});
        let _ = self.device.poll(PollType::wait_indefinitely());

        let cell_ranges_mapped = cell_ranges_slice.get_mapped_range();
        let cell_ranges_result: Vec<CellRange> = bytemuck::cast_slice(&cell_ranges_mapped).to_vec();

        let indirect_mapped = indirect_slice.get_mapped_range();
        let indirect_data: &[u32] = bytemuck::cast_slice(&indirect_mapped);
        let workgroup_count = indirect_data[0];

        println!("\n=== Indirect Stage Debug Info ===");
        println!("Total Workgroups: {}", workgroup_count);
        println!("\nCell Ranges:");

        let mut total_particles_in_ranges = 0u32;
        let mut expected_idx = 0u32;
        let mut has_gaps = false;

        for i in 0..(workgroup_count as usize) {
            let range = cell_ranges_result[i];
            let particles_in_range = range.end - range.start;
            total_particles_in_ranges += particles_in_range;

            // Check for gaps
            if range.start != expected_idx {
                println!(
                    "  ✗ Workgroup {}: indices {}-{} ({} particles) - GAP from {}!",
                    i, range.start, range.end, particles_in_range, expected_idx
                );
                has_gaps = true;
            } else {
                println!(
                    "  Workgroup {}: indices {}-{} ({} particles)",
                    i, range.start, range.end, particles_in_range
                );
            }
            expected_idx = range.end;
        }

        if !has_gaps && total_particles_in_ranges as usize == self.particle_count {
            println!("\n✓ All particles accounted for with no gaps!");
        } else if has_gaps {
            println!("\n✗ GAPS found in ranges!");
        }
        println!();

        drop(cell_ranges_mapped);
        drop(indirect_mapped);
    }
}
