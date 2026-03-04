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
    if let Ok((slab, tracked_perms)) = result {
        let Tracked(slab_perms) = tracked_perms;
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
    if let Ok((mut slab, mut tracked_perms)) = result {
        let Tracked(mut slab_perms) = tracked_perms;
        // Initially all data blocks are not allocated.
        assert(forall|i: int| 0 <= i < slab@.num_data_blocks ==> !slab@.is_allocated(i));

        let alloc_result = slab.allocate(Tracked(&mut slab_perms));
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
    let slab = unsafe { Slab::from_raw_parts(addr, len, block_size, Tracked(mem)) };
    if let Ok((s, _tracked_perms)) = slab {
        assert(s.inv());
        assert(forall|i: int| 0 <= i < s@.num_data_blocks ==> !s@.is_allocated(i));
        assert(s@.block_size == block_size as int);
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
    if let Ok((mut slab, mut tracked_perms)) = result {
        let Tracked(mut slab_perms) = tracked_perms;
        // Allocate a block.
        let block = slab.allocate(Tracked(&mut slab_perms));
        if let Ok(block_addr) = block {
            proof {
                // Block should be allocated.
                let block_idx = slab@.addr_to_block_idx(block_addr as int);
                assert(slab@.is_allocated(block_idx));
            }

            // Deallocate the block.
            let dealloc_result = slab.deallocate(block_addr);
            if let Ok(()) = dealloc_result {
                proof {
                    // Block should be freed.
                    let block_idx = slab@.addr_to_block_idx(block_addr as int);
                    assert(!slab@.is_allocated(block_idx));
                }
            }
        }
    }
}


/// Verifiable test: double deallocation requires the block to be allocated.
/// In Verus, this is expressed as a precondition on deallocate.
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
    if let Ok((mut slab, mut tracked_perms)) = result {
        let Tracked(mut slab_perms) = tracked_perms;
        let block = slab.allocate(Tracked(&mut slab_perms));
        if let Ok(block_addr) = block {
            // First deallocation should succeed.
            let dealloc1 = slab.deallocate(block_addr);
            if let Ok(()) = dealloc1 {
                proof {
                    // After deallocation, block is NOT allocated.
                    let block_idx = slab@.addr_to_block_idx(block_addr as int);
                    assert(!slab@.is_allocated(block_idx));
                    // Therefore, a second deallocation would violate the precondition:
                    // old(self)@.is_allocated(old(self)@.addr_to_block_idx(addr as int)).
                }
            }
        }
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
    if let Ok((slab, tracked_perms)) = result {
        let Tracked(slab_perms) = tracked_perms;
        proof {
            // An out-of-bounds address would NOT satisfy is_valid_addr.
            let invalid_addr = slab@.data_addr + slab@.num_data_blocks * slab@.block_size;
            assert(!slab@.is_valid_addr(invalid_addr));
            // Therefore, calling deallocate(invalid_addr) would violate the precondition.
        }
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
    if let Ok((mut slab, mut tracked_perms)) = result {
        let Tracked(mut slab_perms) = tracked_perms;
        let alloc1 = slab.allocate(Tracked(&mut slab_perms));
        if let Ok(addr1) = alloc1 {
            let alloc2 = slab.allocate(Tracked(&mut slab_perms));
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
    if let Ok((mut slab, mut tracked_perms)) = result {
        let Tracked(mut slab_perms) = tracked_perms;
        let alloc_result = slab.allocate(Tracked(&mut slab_perms));
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
    if let Ok((mut slab, mut tracked_perms)) = result {
        let Tracked(mut slab_perms) = tracked_perms;
        // Allocate a block.
        let alloc1 = slab.allocate(Tracked(&mut slab_perms));
        if let Ok(addr1) = alloc1 {
            // Deallocate.
            let dealloc = slab.deallocate(addr1);
            if let Ok(()) = dealloc {
                // Allocate again - should succeed.
                let alloc2 = slab.allocate(Tracked(&mut slab_perms));
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
    if let Ok((mut slab, mut tracked_perms)) = result {
        let Tracked(mut slab_perms) = tracked_perms;
        let alloc1 = slab.allocate(Tracked(&mut slab_perms));
        if let Ok(addr1) = alloc1 {
            let alloc2 = slab.allocate(Tracked(&mut slab_perms));
            if let Ok(addr2) = alloc2 {
                proof {
                    // All allocated addresses are valid and within the data region.
                    assert(slab@.is_valid_addr(addr1 as int));
                    assert(slab@.is_valid_addr(addr2 as int));
                    assert(addr1 as int >= slab@.data_addr);
                    assert(addr2 as int >= slab@.data_addr);
                }
            }
        }
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
    if let Ok((mut slab, mut tracked_perms)) = result {
        let Tracked(mut slab_perms) = tracked_perms;
        let alloc1 = slab.allocate(Tracked(&mut slab_perms));
        if let Ok(addr1) = alloc1 {
            let alloc2 = slab.allocate(Tracked(&mut slab_perms));
            if let Ok(addr2) = alloc2 {
                proof {
                    let idx1 = slab@.addr_to_block_idx(addr1 as int);
                    let idx2 = slab@.addr_to_block_idx(addr2 as int);
                    // Both blocks are allocated.
                    assert(slab@.is_allocated(idx1));
                    assert(slab@.is_allocated(idx2));
                }

                // Deallocate block 1.
                let dealloc = slab.deallocate(addr1);
                if let Ok(()) = dealloc {
                    proof {
                        let idx1 = slab@.addr_to_block_idx(addr1 as int);
                        let idx2 = slab@.addr_to_block_idx(addr2 as int);
                        // Block 1 is now free.
                        assert(!slab@.is_allocated(idx1));
                        // Block 2 should still be allocated.
                        assert(slab@.is_allocated(idx2));
                    }
                }
            }
        }
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
    if let Ok((slab, tracked_perms)) = result {
        let Tracked(slab_perms) = tracked_perms;
        proof {
            // All data blocks should be free in a fresh slab.
            assert(forall|i: int| 0 <= i < slab@.num_data_blocks ==> !slab@.is_allocated(i));
        }
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
    if let Ok((mut slab, mut tracked_perms)) = result {
        let Tracked(mut slab_perms) = tracked_perms;
        proof {
            // The invariant guarantees index blocks are always marked used.
            slab.lemma_index_blocks_always_set();
        }

        // After allocation, index blocks remain used (invariant preserved).
        let alloc = slab.allocate(Tracked(&mut slab_perms));
        if let Ok(_) = alloc {
            proof {
                // Invariant still holds after allocation.
                slab.lemma_index_blocks_always_set();
            }
        }
    }
}

} // verus!
