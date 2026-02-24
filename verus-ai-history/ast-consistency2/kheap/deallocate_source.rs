    unsafe fn deallocate(&mut self, ptr: *mut u8, layout: Layout) -> Result<(), AllocError> {
        match Kheap::layout_to_allocator(&layout)? {
            SlabSize::Slab8 => self.slab_8_bytes.deallocate(ptr).map_err(|_| AllocError),
            SlabSize::Slab16 => self.slab_16_bytes.deallocate(ptr).map_err(|_| AllocError),
            SlabSize::Slab32 => self.slab_32_bytes.deallocate(ptr).map_err(|_| AllocError),
            SlabSize::Slab64 => self.slab_64_bytes.deallocate(ptr).map_err(|_| AllocError),
            SlabSize::Slab128 => self.slab_128_bytes.deallocate(ptr).map_err(|_| AllocError),
            SlabSize::Slab256 => self.slab_256_bytes.deallocate(ptr).map_err(|_| AllocError),
            SlabSize::Slab512 => self.slab_512_bytes.deallocate(ptr).map_err(|_| AllocError),
            SlabSize::Slab4096 => self.slab_4096_bytes.deallocate(ptr).map_err(|_| AllocError),
        }
    }
