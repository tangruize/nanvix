    unsafe fn allocate(&mut self, layout: Layout) -> Result<*mut u8, AllocError> {
        match Kheap::layout_to_allocator(&layout)? {
            SlabSize::Slab8 => self.slab_8_bytes.allocate().map_err(|_| AllocError),
            SlabSize::Slab16 => self.slab_16_bytes.allocate().map_err(|_| AllocError),
            SlabSize::Slab32 => self.slab_32_bytes.allocate().map_err(|_| AllocError),
            SlabSize::Slab64 => self.slab_64_bytes.allocate().map_err(|_| AllocError),
            SlabSize::Slab128 => self.slab_128_bytes.allocate().map_err(|_| AllocError),
            SlabSize::Slab256 => self.slab_256_bytes.allocate().map_err(|_| AllocError),
            SlabSize::Slab512 => self.slab_512_bytes.allocate().map_err(|_| AllocError),
            SlabSize::Slab4096 => self.slab_4096_bytes.allocate().map_err(|_| AllocError),
        }
    }
