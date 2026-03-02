fn test_slab_from_raw_parts_allocate_verified(
    addr: usize,
    len: usize,
    block_size: usize,
)
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
        // Initially all data blocks are not allocated.
        assert(forall|i: int| 0 <= i < slab@.num_data_blocks ==> !slab@.is_allocated(i));

        let alloc_result = slab.allocate();
        if let Ok(alloc_addr) = alloc_result {
            proof {
                // Allocated address should be valid.
                assert(slab@.is_valid_addr(alloc_addr as int));
                // Block should be allocated.
                let block_idx = slab@.addr_to_block_idx(alloc_addr as int);
                assert(slab@.is_allocated(block_idx));
            }

            let dealloc_result = slab.deallocate(alloc_addr);
            if let Ok(()) = dealloc_result {
                proof {
                    // Block should be freed.
                    let block_idx = slab@.addr_to_block_idx(alloc_addr as int);
                    assert(!slab@.is_allocated(block_idx));
                }
            }
        }
    }
}
