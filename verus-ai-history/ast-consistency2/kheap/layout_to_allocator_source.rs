    pub fn layout_to_allocator(layout: &Layout) -> Result<SlabSize, AllocError> {
        match layout.size() {
            1..=8 => Ok(SlabSize::Slab8),
            9..=16 => Ok(SlabSize::Slab16),
            17..=32 => Ok(SlabSize::Slab32),
            33..=64 => Ok(SlabSize::Slab64),
            65..=128 => Ok(SlabSize::Slab128),
            129..=256 => Ok(SlabSize::Slab256),
            257..=512 => Ok(SlabSize::Slab512),
            4096 => Ok(SlabSize::Slab4096),
            _ => Err(AllocError),
        }
    }
