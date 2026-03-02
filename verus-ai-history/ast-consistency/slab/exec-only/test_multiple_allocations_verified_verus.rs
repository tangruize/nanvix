fn test_multiple_allocations_verified(addr: usize, len: usize, block_size: usize)
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
        // Need at least 16 blocks for this test (enough for index + 2 data blocks).
        len / block_size >= 16,
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size) };
    if let Ok(mut slab) = result {
        let alloc1 = slab.allocate();
        if let Ok(addr1) = alloc1 {
            let alloc2 = slab.allocate();
            if let Ok(addr2) = alloc2 {
                // Two allocations return different addresses.
                assert(addr1 != addr2);
                proof {
                    // Both blocks are allocated.
                    let idx1 = slab@.addr_to_block_idx(addr1 as int);
                    let idx2 = slab@.addr_to_block_idx(addr2 as int);
                    assert(slab@.is_allocated(idx1));
                    assert(slab@.is_allocated(idx2));
                    // Block indices are different.
                    assert(idx1 != idx2);
                }
            }
        }
    }
}
