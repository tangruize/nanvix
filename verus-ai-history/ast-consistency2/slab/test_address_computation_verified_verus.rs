fn test_address_computation_verified(addr: usize, len: usize, block_size: usize)
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
        // Must have at least 8 blocks for a valid slab.
        len / block_size >= 8,
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size) };
    if let Ok(mut slab) = result {
        let alloc_result = slab.allocate();
        if let Ok(alloc_addr) = alloc_result {
            proof {
                // Verify is_valid_addr holds for allocated address.
                assert(slab@.is_valid_addr(alloc_addr as int));
                // Verify block index is within bounds.
                let block_idx = slab@.addr_to_block_idx(alloc_addr as int);
                assert(0 <= block_idx < slab@.num_data_blocks);
                // Verify the block is allocated.
                assert(slab@.is_allocated(block_idx));
            }
        }
    }
}
