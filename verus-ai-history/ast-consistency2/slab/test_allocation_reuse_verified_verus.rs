fn test_allocation_reuse_verified(addr: usize, len: usize, block_size: usize)
    requires
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        Slab::spec_is_power_of_two(block_size as int),
        addr % block_size == 0,
        addr > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        len / block_size >= 8,
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size) };
    if let Ok(mut slab) = result {
        // Allocate a block.
        let alloc1 = slab.allocate();
        if let Ok(addr1) = alloc1 {
            // Deallocate.
            let dealloc = slab.deallocate(addr1);
            if let Ok(()) = dealloc {
                // Allocate again - should succeed.
                let alloc2 = slab.allocate();
                if let Ok(addr2) = alloc2 {
                    proof {
                        // The second allocation should be valid.
                        assert(slab@.is_valid_addr(addr2 as int));
                        assert(slab@.is_allocated(slab@.addr_to_block_idx(addr2 as int)));
                    }
                }
            }
        }
    }
}
