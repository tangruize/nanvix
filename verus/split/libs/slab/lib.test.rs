// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Slab Allocator - Verified Tests.
// Verified test functions that prove key slab allocator properties.

verus! {

//==================================================================================================
// Verified Test Functions
//==================================================================================================

/// Verifiable test: from_raw_parts creates a valid slab with expected properties.
fn test_slab_from_raw_parts_verified(
    addr: *mut u8,
    len: usize,
    block_size: usize,
    Tracked(mem): Tracked<PointsToRaw>,
)
    requires
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        is_pow2(block_size as int),
        (addr as usize) % block_size == 0,
        addr as int > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        len / block_size >= 8,
        mem.is_range(addr as int, len as int),
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size, Tracked(mem)) };
    match result {
        Ok(init_pair) => {
            let (slab, Tracked(slab_perms)) = init_pair;
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
        },
        Err(_) => {},
    }
}


/// Verifiable test: from_raw_parts followed by allocate/deallocate works correctly.
fn test_slab_from_raw_parts_allocate_verified(
    addr: *mut u8,
    len: usize,
    block_size: usize,
    Tracked(mem): Tracked<PointsToRaw>,
)
    requires
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        is_pow2(block_size as int),
        (addr as usize) % block_size == 0,
        addr as int > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        len / block_size >= 8,
        mem.is_range(addr as int, len as int),
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size, Tracked(mem)) };
    match result {
        Ok(init_pair) => {
            let (mut slab, perms_tracked) = init_pair;
            let Tracked(mut slab_perms) = perms_tracked;
            // Initially all data blocks are not allocated.
            assert(forall|i: int| 0 <= i < slab@.num_data_blocks ==> !slab@.is_allocated(i));

            match slab.allocate(Tracked(&mut slab_perms)) {
                Ok(alloc_pair) => {
                    let (alloc_addr, Tracked(block_perm)) = alloc_pair;
                    proof {
                        // Allocated address should be valid.
                        assert(slab@.is_valid_addr(alloc_addr as int));
                        // Block should be allocated.
                        let block_idx = slab@.addr_to_block_idx(alloc_addr as int);
                        assert(slab@.is_allocated(block_idx));
                    }

                    match unsafe { slab.deallocate(alloc_addr, Tracked(block_perm), Tracked(&mut slab_perms)) } {
                        Ok(()) => {
                            proof {
                                // Block should be freed.
                                let block_idx = slab@.addr_to_block_idx(alloc_addr as int);
                                assert(!slab@.is_allocated(block_idx));
                            }
                        },
                        Err(_) => {},
                    }
                },
                Err(_) => {},
            }
        },
        Err(_) => {},
    }
}

//==================================================================================================

/// Verifiable test: slab creation with valid parameters.
fn test_slab_creation_verified(
    addr: *mut u8,
    len: usize,
    block_size: usize,
    Tracked(mem): Tracked<PointsToRaw>,
)
    requires
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        is_pow2(block_size as int),
        (addr as usize) % block_size == 0,
        addr as int > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        len / block_size >= 8,
        mem.is_range(addr as int, len as int),
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size, Tracked(mem)) };
    match result {
        Ok(init_pair) => {
            let (s, Tracked(slab_perms)) = init_pair;
            assert(s.inv());
            assert(forall|i: int| 0 <= i < s@.num_data_blocks ==> !s@.is_allocated(i));
            assert(s@.block_size == block_size as int);
        },
        Err(_) => {},
    }
}


/// Verifiable test: allocating a block and then deallocating it.
fn test_allocate_deallocate_verified(
    addr: *mut u8,
    len: usize,
    block_size: usize,
    Tracked(mem): Tracked<PointsToRaw>,
)
    requires
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        is_pow2(block_size as int),
        (addr as usize) % block_size == 0,
        addr as int > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        len / block_size >= 8,
        mem.is_range(addr as int, len as int),
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size, Tracked(mem)) };
    match result {
        Ok(init_pair) => {
            let (mut slab, perms_tracked) = init_pair;
            let Tracked(mut slab_perms) = perms_tracked;
            // Allocate a block.
            match slab.allocate(Tracked(&mut slab_perms)) {
                Ok(alloc_pair) => {
                    let (block_addr, Tracked(block_perm)) = alloc_pair;
                    proof {
                        // Block should be allocated.
                        let block_idx = slab@.addr_to_block_idx(block_addr as int);
                        assert(slab@.is_allocated(block_idx));
                    }

                    // Deallocate the block.
                    match unsafe { slab.deallocate(block_addr, Tracked(block_perm), Tracked(&mut slab_perms)) } {
                        Ok(()) => {
                            proof {
                                // Block should be freed.
                                let block_idx = slab@.addr_to_block_idx(block_addr as int);
                                assert(!slab@.is_allocated(block_idx));
                            }
                        },
                        Err(_) => {},
                    }
                },
                Err(_) => {},
            }
        },
        Err(_) => {},
    }
}


/// Verifiable test: double deallocation is prevented by linear permissions.
/// With the updated API, block_perm is consumed on first deallocation,
/// so a second deallocation is statically impossible.
fn test_double_deallocate_verified(
    addr: *mut u8,
    len: usize,
    block_size: usize,
    Tracked(mem): Tracked<PointsToRaw>,
)
    requires
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        is_pow2(block_size as int),
        (addr as usize) % block_size == 0,
        addr as int > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        len / block_size >= 8,
        mem.is_range(addr as int, len as int),
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size, Tracked(mem)) };
    match result {
        Ok(init_pair) => {
            let (mut slab, perms_tracked) = init_pair;
            let Tracked(mut slab_perms) = perms_tracked;
            match slab.allocate(Tracked(&mut slab_perms)) {
                Ok(alloc_pair) => {
                    let (block_addr, Tracked(block_perm)) = alloc_pair;
                    // First deallocation should succeed.
                    match unsafe { slab.deallocate(block_addr, Tracked(block_perm), Tracked(&mut slab_perms)) } {
                        Ok(()) => {
                            proof {
                                // After deallocation, block is NOT allocated.
                                let block_idx = slab@.addr_to_block_idx(block_addr as int);
                                assert(!slab@.is_allocated(block_idx));
                                // A second deallocation is impossible because block_perm
                                // has been consumed by the first deallocation.
                                // The linear permission system statically prevents double-free.
                            }
                        },
                        Err(_) => {},
                    }
                },
                Err(_) => {},
            }
        },
        Err(_) => {},
    }
}


/// Verifiable test: deallocating an out-of-bounds address would violate preconditions.
/// In Verus, this is expressed as: deallocate requires is_valid_addr(addr).
fn test_allocate_out_of_bounds_verified(
    addr: *mut u8,
    len: usize,
    block_size: usize,
    Tracked(mem): Tracked<PointsToRaw>,
)
    requires
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        is_pow2(block_size as int),
        (addr as usize) % block_size == 0,
        addr as int > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        len / block_size >= 8,
        mem.is_range(addr as int, len as int),
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size, Tracked(mem)) };
    match result {
        Ok(init_pair) => {
            let (slab, Tracked(slab_perms)) = init_pair;
            proof {
                // An out-of-bounds address would NOT satisfy is_valid_addr.
                let invalid_addr = slab@.data_addr + slab@.num_data_blocks * slab@.block_size;
                assert(!slab@.is_valid_addr(invalid_addr));
                // Therefore, calling deallocate(invalid_addr, ...) would violate the precondition.
            }
        },
        Err(_) => {},
    }
}


/// Verifiable test: multiple allocations return different addresses.
fn test_multiple_allocations_verified(
    addr: *mut u8,
    len: usize,
    block_size: usize,
    Tracked(mem): Tracked<PointsToRaw>,
)
    requires
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        is_pow2(block_size as int),
        (addr as usize) % block_size == 0,
        addr as int > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        len / block_size >= 16,
        mem.is_range(addr as int, len as int),
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size, Tracked(mem)) };
    match result {
        Ok(init_pair) => {
            let (mut slab, perms_tracked) = init_pair;
            let Tracked(mut slab_perms) = perms_tracked;
            match slab.allocate(Tracked(&mut slab_perms)) {
                Ok(alloc_pair1) => {
                    let (addr1, Tracked(block_perm1)) = alloc_pair1;
                    match slab.allocate(Tracked(&mut slab_perms)) {
                        Ok(alloc_pair2) => {
                            let (addr2, Tracked(block_perm2)) = alloc_pair2;
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
                        },
                        Err(_) => {},
                    }
                },
                Err(_) => {},
            }
        },
        Err(_) => {},
    }
}


/// Verifiable test: address computation properties.
fn test_address_computation_verified(
    addr: *mut u8,
    len: usize,
    block_size: usize,
    Tracked(mem): Tracked<PointsToRaw>,
)
    requires
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        is_pow2(block_size as int),
        (addr as usize) % block_size == 0,
        addr as int > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        len / block_size >= 8,
        mem.is_range(addr as int, len as int),
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size, Tracked(mem)) };
    match result {
        Ok(init_pair) => {
            let (mut slab, perms_tracked) = init_pair;
            let Tracked(mut slab_perms) = perms_tracked;
            match slab.allocate(Tracked(&mut slab_perms)) {
                Ok(alloc_pair) => {
                    let (alloc_addr, Tracked(block_perm)) = alloc_pair;
                    proof {
                        // Verify is_valid_addr holds for allocated address.
                        assert(slab@.is_valid_addr(alloc_addr as int));
                        // Verify block index is within bounds.
                        let block_idx = slab@.addr_to_block_idx(alloc_addr as int);
                        assert(0 <= block_idx < slab@.num_data_blocks);
                        // Verify the block is allocated.
                        assert(slab@.is_allocated(block_idx));
                    }
                },
                Err(_) => {},
            }
        },
        Err(_) => {},
    }
}

//==================================================================================================

/// Verifiable test: after deallocation, the same block can be reallocated.
fn test_allocation_reuse_verified(
    addr: *mut u8,
    len: usize,
    block_size: usize,
    Tracked(mem): Tracked<PointsToRaw>,
)
    requires
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        is_pow2(block_size as int),
        (addr as usize) % block_size == 0,
        addr as int > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        len / block_size >= 8,
        mem.is_range(addr as int, len as int),
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size, Tracked(mem)) };
    match result {
        Ok(init_pair) => {
            let (mut slab, perms_tracked) = init_pair;
            let Tracked(mut slab_perms) = perms_tracked;
            // Allocate a block.
            match slab.allocate(Tracked(&mut slab_perms)) {
                Ok(alloc_pair1) => {
                    let (addr1, Tracked(block_perm1)) = alloc_pair1;
                    // Deallocate.
                    match unsafe { slab.deallocate(addr1, Tracked(block_perm1), Tracked(&mut slab_perms)) } {
                        Ok(()) => {
                            // Allocate again - should succeed.
                            match slab.allocate(Tracked(&mut slab_perms)) {
                                Ok(alloc_pair2) => {
                                    let (addr2, Tracked(block_perm2)) = alloc_pair2;
                                    proof {
                                        // The second allocation should be valid.
                                        assert(slab@.is_valid_addr(addr2 as int));
                                        assert(slab@.is_allocated(slab@.addr_to_block_idx(addr2 as int)));
                                    }
                                },
                                Err(_) => {},
                            }
                        },
                        Err(_) => {},
                    }
                },
                Err(_) => {},
            }
        },
        Err(_) => {},
    }
}


/// Verifiable test: all allocated addresses are aligned to block_size.
fn test_memory_block_alignment_verified(
    addr: *mut u8,
    len: usize,
    block_size: usize,
    Tracked(mem): Tracked<PointsToRaw>,
)
    requires
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        is_pow2(block_size as int),
        (addr as usize) % block_size == 0,
        addr as int > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        len / block_size >= 16,
        mem.is_range(addr as int, len as int),
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size, Tracked(mem)) };
    match result {
        Ok(init_pair) => {
            let (mut slab, perms_tracked) = init_pair;
            let Tracked(mut slab_perms) = perms_tracked;
            match slab.allocate(Tracked(&mut slab_perms)) {
                Ok(alloc_pair1) => {
                    let (addr1, Tracked(block_perm1)) = alloc_pair1;
                    match slab.allocate(Tracked(&mut slab_perms)) {
                        Ok(alloc_pair2) => {
                            let (addr2, Tracked(block_perm2)) = alloc_pair2;
                            proof {
                                // All allocated addresses are valid and within the data region.
                                assert(slab@.is_valid_addr(addr1 as int));
                                assert(slab@.is_valid_addr(addr2 as int));
                                assert(addr1 as int >= slab@.data_addr);
                                assert(addr2 as int >= slab@.data_addr);
                            }
                        },
                        Err(_) => {},
                    }
                },
                Err(_) => {},
            }
        },
        Err(_) => {},
    }
}


/// Verifiable test: deallocating one block doesn't affect other allocated blocks.
fn test_no_data_corruption_verified(
    addr: *mut u8,
    len: usize,
    block_size: usize,
    Tracked(mem): Tracked<PointsToRaw>,
)
    requires
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        is_pow2(block_size as int),
        (addr as usize) % block_size == 0,
        addr as int > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        len / block_size >= 16,
        mem.is_range(addr as int, len as int),
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size, Tracked(mem)) };
    match result {
        Ok(init_pair) => {
            let (mut slab, perms_tracked) = init_pair;
            let Tracked(mut slab_perms) = perms_tracked;
            match slab.allocate(Tracked(&mut slab_perms)) {
                Ok(alloc_pair1) => {
                    let (addr1, Tracked(block_perm1)) = alloc_pair1;
                    match slab.allocate(Tracked(&mut slab_perms)) {
                        Ok(alloc_pair2) => {
                            let (addr2, Tracked(block_perm2)) = alloc_pair2;
                            proof {
                                let idx1 = slab@.addr_to_block_idx(addr1 as int);
                                let idx2 = slab@.addr_to_block_idx(addr2 as int);
                                // Both blocks are allocated.
                                assert(slab@.is_allocated(idx1));
                                assert(slab@.is_allocated(idx2));
                            }

                            // Deallocate block 1.
                            match unsafe { slab.deallocate(addr1, Tracked(block_perm1), Tracked(&mut slab_perms)) } {
                                Ok(()) => {
                                    proof {
                                        let idx1 = slab@.addr_to_block_idx(addr1 as int);
                                        let idx2 = slab@.addr_to_block_idx(addr2 as int);
                                        // Block 1 is now free.
                                        assert(!slab@.is_allocated(idx1));
                                        // Block 2 should still be allocated.
                                        assert(slab@.is_allocated(idx2));
                                    }
                                },
                                Err(_) => {},
                            }
                        },
                        Err(_) => {},
                    }
                },
                Err(_) => {},
            }
        },
        Err(_) => {},
    }
}


/// Verifiable test: fresh slab has all data blocks free.
fn test_fresh_slab_all_free_verified(
    addr: *mut u8,
    len: usize,
    block_size: usize,
    Tracked(mem): Tracked<PointsToRaw>,
)
    requires
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        is_pow2(block_size as int),
        (addr as usize) % block_size == 0,
        addr as int > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        len / block_size >= 8,
        mem.is_range(addr as int, len as int),
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size, Tracked(mem)) };
    match result {
        Ok(init_pair) => {
            let (slab, Tracked(slab_perms)) = init_pair;
            proof {
                // All data blocks should be free in a fresh slab.
                assert(forall|i: int| 0 <= i < slab@.num_data_blocks ==> !slab@.is_allocated(i));
            }
        },
        Err(_) => {},
    }
}


/// Verifiable test: index blocks are always marked as used.
fn test_index_blocks_always_used_verified(
    addr: *mut u8,
    len: usize,
    block_size: usize,
    Tracked(mem): Tracked<PointsToRaw>,
)
    requires
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        is_pow2(block_size as int),
        (addr as usize) % block_size == 0,
        addr as int > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        len / block_size >= 8,
        mem.is_range(addr as int, len as int),
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size, Tracked(mem)) };
    match result {
        Ok(init_pair) => {
            let (mut slab, perms_tracked) = init_pair;
            let Tracked(mut slab_perms) = perms_tracked;
            proof {
                // The invariant guarantees index blocks are always marked used.
                slab.lemma_index_blocks_always_set();
            }

            // After allocation, index blocks remain used (invariant preserved).
            match slab.allocate(Tracked(&mut slab_perms)) {
                Ok(alloc_pair) => {
                    let (alloc_addr, Tracked(block_perm)) = alloc_pair;
                    proof {
                        // Invariant still holds after allocation.
                        slab.lemma_index_blocks_always_set();
                    }
                },
                Err(_) => {},
            }
        },
        Err(_) => {},
    }
}

} // verus!
