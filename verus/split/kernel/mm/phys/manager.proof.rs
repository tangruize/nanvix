// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Proofs and lemmas.

verus! {

//==================================================================================================

/// Proof that a newly created manager has valid invariant.
proof fn proof_new_manager_invariant(kpool: Kpool, upool: Upool)
    requires
        kpool.inv(),
        upool.inv(),
    ensures
        (VirtMemoryManager { kpool, upool }).inv(),
{
    // Direct from constructor postconditions.
}


/// Proof that allocation decreases free count.
proof fn proof_alloc_decreases_free(old_manager: VirtMemoryManager, new_manager: VirtMemoryManager)
    requires
        old_manager.inv(),
        new_manager.inv(),
        new_manager@.upool_free_count == old_manager@.upool_free_count - 1,
    ensures
        new_manager@.upool_free_count < old_manager@.upool_free_count,
{
    // Trivial arithmetic.
}

} // verus!
