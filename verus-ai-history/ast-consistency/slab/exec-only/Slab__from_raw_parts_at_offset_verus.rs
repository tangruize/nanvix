    pub unsafe fn from_raw_parts_at_offset(
        base_addr: usize,
        slab_size: usize,
        offset: usize,
        block_size: usize,
    ) -> (result: Result<Slab, Error>)
        requires
            base_addr > 0,
            slab_size > 0,
            slab_size < i32::MAX as usize,
            offset < 8,
            block_size > 0,
            block_size < i32::MAX as usize,
            block_size <= slab_size,
            Self::spec_is_power_of_two(block_size as int),
            // Alignment: base_addr + offset * slab_size must be aligned to block_size.
            ((base_addr as int) + (offset as int) * (slab_size as int)) % (block_size as int) == 0,
            // No overflow: the END of this slab region (base_addr + (offset+1) * slab_size) fits.
            (base_addr as int) + ((offset as int) + 1) * (slab_size as int) <= (usize::MAX as int),
            // Overflow check: offset * slab_size fits in usize.
            (offset as int) * (slab_size as int) <= (usize::MAX as int),
            // Overflow check: base_addr + offset * slab_size fits in usize.
            (base_addr as int) + (offset as int) * (slab_size as int) <= (usize::MAX as int),
            // Additional preconditions for from_raw_parts:
            // Total number of blocks must be a multiple of 8.
            (slab_size / block_size) % (u8::BITS as usize) == 0,
            // Ensure we have enough blocks for a valid slab (at least 8).
            slab_size / block_size >= 8,
        ensures
            result is Ok ==> {
                let slab = result->Ok_0;
                &&& slab.inv()
                &&& slab@.block_size == block_size as int
                &&& slab@.num_data_blocks > 0
                // Freshly initialized: no blocks allocated (Set-based, no forall).
                &&& slab@.allocated_blocks =~= Set::<int>::empty()
                // Critical: data region is within the assigned slice.
                &&& slab@.data_addr >= (base_addr as int) + (offset as int) * (slab_size as int)
                &&& slab@.data_addr + slab@.num_data_blocks * slab@.block_size
                    <= (base_addr as int) + ((offset as int) + 1) * (slab_size as int)
                // Alignment: data_addr is aligned to block_size.
                &&& slab@.is_aligned()
            },
    {
        // Calculate the address for this slab region.
        // Preconditions ensure no overflow.
        let offset_times_slab: usize = offset * slab_size;
        let addr: usize = base_addr + offset_times_slab;

        // Prove the precondition for from_raw_parts: addr + slab_size <= usize::MAX.
        proof {
            // addr = base_addr + offset * slab_size.
            assert((addr as int) == (base_addr as int) + (offset as int) * (slab_size as int));

            // Use the distributive lemma: (offset + 1) * slab_size = offset * slab_size + slab_size.
            Self::lemma_mul_distribute((offset as int), (slab_size as int));
            assert(((offset as int) + 1int) * (slab_size as int)
                   == (offset as int) * (slab_size as int) + (slab_size as int));

            // addr + slab_size = base_addr + offset * slab_size + slab_size
            //                  = base_addr + (offset + 1) * slab_size.
            assert((addr as int) + (slab_size as int)
                   == (base_addr as int) + (offset as int) * (slab_size as int) + (slab_size as int));
            assert((addr as int) + (slab_size as int)
                   == (base_addr as int) + ((offset as int) + 1int) * (slab_size as int));

            // From precondition: base_addr + (offset + 1) * slab_size <= usize::MAX.
            assert((base_addr as int) + ((offset as int) + 1int) * (slab_size as int) <= (usize::MAX as int));
            assert((addr as int) + (slab_size as int) <= (usize::MAX as int));
        }

        // Use the existing from_raw_parts to create the slab.
        Self::from_raw_parts(addr, slab_size, block_size)
    }
