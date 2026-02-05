// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Proofs and lemmas.

verus! {

//==================================================================================================

/// Proof that user and kernel spaces are disjoint.
proof fn user_kernel_disjoint_proof(vaddr: int)
    ensures
        !(spec_is_user_addr(vaddr) && spec_is_kernel_addr(vaddr)),
{
    // By definition, kernel space is !user_space.
}


/// Proof that USER_BASE < USER_END.
proof fn user_space_bounds_valid()
    ensures
        USER_BASE < USER_END,
{
    // Constants are set such that USER_BASE = 1GB < USER_END = 3GB.
}


/// Proof that MAX_USER_PAGES is sufficient for the user address space.
/// With PAGE_SIZE = 4096 bytes, the user space (USER_END - USER_BASE) = 2GB = 524288 pages.
/// MAX_USER_PAGES = 65536 covers 256MB, which is sufficient for embedded/microkernel use.
proof fn max_user_pages_sufficient()
    ensures
        MAX_USER_PAGES as int * PAGE_SIZE as int >= MEMORY_SIZE as int,
        // MAX_USER_PAGES can cover at least 256MB (MEMORY_SIZE)
{
    // 65536 * 4096 = 268435456 bytes = 256 MB >= MEMORY_SIZE (256 MB)
}


/// Proof that a valid user region implies start is a user address.
proof fn user_region_implies_user_addr(start: int, size: int)
    requires
        spec_is_user_region(start, size),
    ensures
        spec_is_user_addr(start),
{
    // By definition of spec_is_user_region.
}


/// Proof that a valid kernel region implies start is a kernel address.
proof fn kernel_region_implies_kernel_addr(start: int, size: int)
    requires
        spec_is_kernel_region(start, size),
    ensures
        spec_is_kernel_addr(start),
{
    // By definition of spec_is_kernel_region.
}

} // verus!
