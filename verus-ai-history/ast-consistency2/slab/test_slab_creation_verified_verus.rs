fn test_slab_creation_verified(addr: usize, len: usize, block_size: usize)
    requires
        // Simulating: vec![0u32; 1024] with block_size 4
        // len = 1024 * 4 = 4096 bytes, block_size = 4
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
    let slab = unsafe { Slab::from_raw_parts(addr, len, block_size) };
    if let Ok(s) = slab {
        assert(s.inv());
        assert(forall|i: int| 0 <= i < s@.num_data_blocks ==> !s@.is_allocated(i));
        assert(s@.block_size == block_size as int);
    }
}
