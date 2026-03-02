fn test_allocate_out_of_bounds_verified(addr: usize, len: usize, block_size: usize)
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
    if let Ok(slab) = result {
        proof {
            // An out-of-bounds address would NOT satisfy is_valid_addr.
            // For example, an address beyond the slab's data region:
            let invalid_addr = slab@.data_addr + slab@.num_data_blocks * slab@.block_size;
            // This address is NOT valid:
            assert(!slab@.is_valid_addr(invalid_addr));
            // Therefore, calling deallocate(invalid_addr) would violate the precondition.
            // This is the verus way of expressing "out of bounds deallocation fails".
        }
    }
}
