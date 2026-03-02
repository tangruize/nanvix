fn test_slab_from_raw_parts_verified(
    addr: usize,
    len: usize,
    block_size: usize,
)
    requires
        // Length must be valid and non-zero.
        len > 0,
        len < i32::MAX as usize,
        // Block size must be valid.
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        // Block size must be a power of two.
        Slab::spec_is_power_of_two(block_size as int),
        // Start address must be aligned to block size.
        addr % block_size == 0,
        addr > 0,
        // Memory region fits in address space.
        (addr as int) + (len as int) <= (usize::MAX as int),
        // Total number of blocks must be a multiple of 8.
        (len / block_size) % (u8::BITS as usize) == 0,
        // Must have at least 8 blocks for a valid slab.
        len / block_size >= 8,
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size) };
    if let Ok(slab) = result {
        // Slab should satisfy invariant.
        assert(slab.inv());
        // All data blocks are not allocated.
        assert(forall|i: int| 0 <= i < slab@.num_data_blocks ==> !slab@.is_allocated(i));
        // Block size should match.
        assert(slab@.block_size == block_size as int);
        // Data address should be properly aligned.
        assert(slab@.data_addr % (block_size as int) == 0);
        // Number of data blocks should be positive.
        assert(slab@.num_data_blocks > 0);
    }
}
