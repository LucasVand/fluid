use eframe::wgpu::{Buffer, BufferDescriptor, BufferUsages, Device, Queue};

pub struct BufferAllocation {
    pub offset: u64,
    pub size: u64,
    pub label: String,
}

pub struct SharedBuffer {
    buffer: Buffer,
    allocations: Vec<BufferAllocation>,
    current_offset: u64,
    max_size: u64,
}

impl SharedBuffer {
    const ALIGNMENT: u64 = 16; // 16-byte alignment for GPU

    pub fn new(device: &Device, size: u64) -> Self {
        Self::with_usages(
            device,
            size,
            BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        )
    }

    pub fn with_usages(device: &Device, size: u64, usages: BufferUsages) -> Self {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Generic Shared Buffer"),
            size,
            usage: usages,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            allocations: Vec::new(),
            current_offset: 0,
            max_size: size,
        }
    }

    /// Allocate space in the buffer for data and write it
    pub fn allocate(&mut self, queue: &Queue, data: &[u8], label: &str) -> u64 {
        // Calculate aligned offset
        let aligned_offset = self.align_offset(self.current_offset);
        let data_size = data.len() as u64;

        // Check if we have enough space
        if aligned_offset + data_size > self.max_size {
            panic!(
                "SharedBuffer: not enough space for '{}'. Need {} bytes at offset {}, but max is {}",
                label, data_size, aligned_offset, self.max_size
            );
        }

        // Write data to buffer
        queue.write_buffer(&self.buffer, aligned_offset, data);

        // Track allocation
        let allocation_index = self.allocations.len() as u64;
        self.allocations.push(BufferAllocation {
            offset: aligned_offset,
            size: data_size,
            label: label.to_string(),
        });

        // Update current offset to account for alignment and next aligned boundary
        self.current_offset = aligned_offset + data_size;
        self.current_offset = self.align_offset(self.current_offset);

        allocation_index
    }

    /// Allocate space in the buffer without writing initial content
    pub fn allocate_empty(&mut self, size: u64, label: &str) -> u64 {
        // Calculate aligned offset
        let aligned_offset = self.align_offset(self.current_offset);

        // Check if we have enough space
        if aligned_offset + size > self.max_size {
            panic!(
                "SharedBuffer: not enough space for '{}'. Need {} bytes at offset {}, but max is {}",
                label, size, aligned_offset, self.max_size
            );
        }

        // Track allocation
        let allocation_index = self.allocations.len() as u64;
        self.allocations.push(BufferAllocation {
            offset: aligned_offset,
            size,
            label: label.to_string(),
        });

        // Update current offset to account for alignment and next aligned boundary
        self.current_offset = aligned_offset + size;
        self.current_offset = self.align_offset(self.current_offset);

        allocation_index
    }

    /// Allocate space in the buffer with 256-byte alignment for dynamic uniform offsets
    pub fn allocate_uniform(&mut self, queue: &Queue, data: &[u8], label: &str) -> u64 {
        // Calculate 256-byte aligned offset
        let aligned_offset = self.align_offset_uniform(self.current_offset);
        let data_size = data.len() as u64;

        // Check if we have enough space
        if aligned_offset + data_size > self.max_size {
            panic!(
                "SharedBuffer: not enough space for uniform '{}'. Need {} bytes at offset {}, but max is {}",
                label, data_size, aligned_offset, self.max_size
            );
        }

        // Write data to buffer
        queue.write_buffer(&self.buffer, aligned_offset, data);

        // Track allocation
        let allocation_index = self.allocations.len() as u64;
        self.allocations.push(BufferAllocation {
            offset: aligned_offset,
            size: data_size,
            label: label.to_string(),
        });

        // Update current offset to next 256-byte boundary
        self.current_offset = aligned_offset + data_size;
        self.current_offset = self.align_offset_uniform(self.current_offset);

        allocation_index
    }

    /// Allocate space in the buffer with 256-byte alignment without writing initial content
    pub fn allocate_uniform_empty(&mut self, size: u64, label: &str) -> u64 {
        // Calculate 256-byte aligned offset
        let aligned_offset = self.align_offset_uniform(self.current_offset);

        // Check if we have enough space
        if aligned_offset + size > self.max_size {
            panic!(
                "SharedBuffer: not enough space for uniform '{}'. Need {} bytes at offset {}, but max is {}",
                label, size, aligned_offset, self.max_size
            );
        }

        // Track allocation
        let allocation_index = self.allocations.len() as u64;
        self.allocations.push(BufferAllocation {
            offset: aligned_offset,
            size,
            label: label.to_string(),
        });

        // Update current offset to next 256-byte boundary
        self.current_offset = aligned_offset + size;
        self.current_offset = self.align_offset_uniform(self.current_offset);

        allocation_index
    }
    pub fn update(&self, queue: &Queue, index: u64, new_data: &[u8]) {
        let alloc = self
            .allocations
            .get(index as usize)
            .unwrap_or_else(|| panic!("SharedBuffer: allocation index {} out of bounds", index));

        if new_data.len() as u64 > alloc.size {
            panic!(
                "SharedBuffer: new data ({} bytes) for '{}' exceeds allocated size ({} bytes)",
                new_data.len(),
                alloc.label,
                alloc.size
            );
        }

        queue.write_buffer(&self.buffer, alloc.offset, new_data);
    }

    /// Get the byte offset of an allocation for use in shaders
    pub fn get_offset(&self, index: u64) -> u64 {
        self.allocations
            .get(index as usize)
            .unwrap_or_else(|| panic!("SharedBuffer: allocation index {} out of bounds", index))
            .offset
    }

    /// Get the size of an allocation
    pub fn get_size(&self, index: u64) -> u64 {
        self.allocations
            .get(index as usize)
            .unwrap_or_else(|| panic!("SharedBuffer: allocation index {} out of bounds", index))
            .size
    }

    /// Get reference to the underlying GPU buffer
    pub fn get_buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Get a buffer slice for an allocation
    pub fn get_slice(&self, index: u64) -> eframe::wgpu::BufferSlice {
        let alloc = self
            .allocations
            .get(index as usize)
            .unwrap_or_else(|| panic!("SharedBuffer: allocation index {} out of bounds", index));

        self.buffer.slice(alloc.offset..alloc.offset + alloc.size)
    }

    /// Get all allocations (for debugging or iteration)
    pub fn allocations(&self) -> &[BufferAllocation] {
        &self.allocations
    }

    /// Check how much free space is available
    pub fn available_space(&self) -> u64 {
        let next_aligned = self.align_offset(self.current_offset);
        if next_aligned >= self.max_size {
            0
        } else {
            self.max_size - next_aligned
        }
    }

    /// Align offset to 16-byte boundary
    fn align_offset(&self, offset: u64) -> u64 {
        ((offset + Self::ALIGNMENT - 1) / Self::ALIGNMENT) * Self::ALIGNMENT
    }

    /// Align offset to 256-byte boundary for dynamic uniform offsets
    fn align_offset_uniform(&self, offset: u64) -> u64 {
        const UNIFORM_ALIGNMENT: u64 = 256;
        ((offset + UNIFORM_ALIGNMENT - 1) / UNIFORM_ALIGNMENT) * UNIFORM_ALIGNMENT
    }
}
