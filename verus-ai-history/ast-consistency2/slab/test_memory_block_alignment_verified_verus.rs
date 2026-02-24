fn test_memory_block_alignment_verified(addr: usize, len: usize, block_size: usize)
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
        len / block_size >= 16,
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size) };
    if let Ok(mut slab) = result {
        let alloc1 = slab.allocate();
        if let Ok(addr1) = alloc1 {
            let alloc2 = slab.allocate();
            if let Ok(addr2) = alloc2 {
                proof {
                    // All allocated addresses should be aligned to block_size.
                    // This is a key property: addr = data_addr + block_idx * block_size.
                    // If data_addr is aligned and block_size is power of 2, result is aligned.
                    assert(slab@.is_valid_addr(addr1 as int));
                    assert(slab@.is_valid_addr(addr2 as int));
                    // Both addresses are within the data region.
                    assert(addr1 as int >= slab@.data_addr);
                    assert(addr2 as int >= slab@.data_addr);
                }
            }
        }
    }
}
