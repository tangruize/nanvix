    unsafe fn from_raw_parts(addr: usize, size: usize) -> Result<Kheap, Error> {
        // Check if start address is not page aligned.
        if !addr.is_multiple_of(mem::PAGE_SIZE) {
            return Err(Error::new(ErrorCode::InvalidArgument, "unaligned start address"));
        }

        // Check if size is less than minimum heap size.
        if size < MIN_HEAP_SIZE {
            return Err(Error::new(
                ErrorCode::InvalidArgument,
                "heap size is less than minimum heap size",
            ));
        }

        // Check if size is not a multiple of heap size.
        if !size.is_multiple_of(MIN_HEAP_SIZE) {
            error!("size is not a multiple of page size");
            return Err(Error::new(
                ErrorCode::InvalidArgument,
                "size is not a multiple of page size",
            ));
        }

        let heap_start_addr: *mut u8 = addr as *mut u8;
        let slab_size: usize = size / NUM_OF_SLABS;
        info!("heap size: {} MB", size / constants::MEGABYTE);
        info!("slab size: {} KB", slab_size / constants::KILOBYTE);
        Ok(Kheap {
            slab_8_bytes: Slab::from_raw_parts(
                heap_start_addr,
                slab_size,
                SlabSize::Slab8 as usize,
            )?,
            slab_16_bytes: Slab::from_raw_parts(
                heap_start_addr.add(slab_size),
                slab_size,
                SlabSize::Slab16 as usize,
            )?,
            slab_32_bytes: Slab::from_raw_parts(
                heap_start_addr.add(2 * slab_size),
                slab_size,
                SlabSize::Slab32 as usize,
            )?,
            slab_64_bytes: Slab::from_raw_parts(
                heap_start_addr.add(3 * slab_size),
                slab_size,
                SlabSize::Slab64 as usize,
            )?,
            slab_128_bytes: Slab::from_raw_parts(
                heap_start_addr.add(4 * slab_size),
                slab_size,
                SlabSize::Slab128 as usize,
            )?,
            slab_256_bytes: Slab::from_raw_parts(
                heap_start_addr.add(5 * slab_size),
                slab_size,
                SlabSize::Slab256 as usize,
            )?,
            slab_512_bytes: Slab::from_raw_parts(
                heap_start_addr.add(6 * slab_size),
                slab_size,
                SlabSize::Slab512 as usize,
            )?,
            slab_4096_bytes: Slab::from_raw_parts(
                heap_start_addr.add(7 * slab_size),
                slab_size,
                SlabSize::Slab4096 as usize,
            )?,
        })
    }
