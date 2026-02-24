fn test_index_blocks_always_used_verified(addr: usize, len: usize, block_size: usize)
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
        proof {
            // The invariant guarantees index blocks are always marked used.
            slab.lemma_index_blocks_always_set();
        }

        // After allocation, index blocks remain used (invariant preserved).
        let alloc = slab.allocate();
        if let Ok(_) = alloc {
            proof {
                // Invariant still holds after allocation.
                slab.lemma_index_blocks_always_set();
            }
        }
    }
}
