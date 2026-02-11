// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Proofs and lemmas for the virtual memory init module.

verus! {

//==================================================================================================
// Alignment Lemmas
//==================================================================================================

/// Proof that align_down produces a value <= the input.
///
/// # Description
///
/// For any positive alignment, the aligned-down value is at most the original.
/// This follows from the floor-division property: (a / k) * k <= a.
proof fn lemma_align_down_le(addr: int, alignment: int)
    requires
        alignment > 0,
        addr >= 0,
    ensures
        spec_align_down(addr, alignment) <= addr,
{
    // By Euclidean division: addr == (addr / alignment) * alignment + (addr % alignment).
    // Since addr % alignment >= 0, we have (addr / alignment) * alignment <= addr.
    assert(addr == (addr / alignment) * alignment + (addr % alignment));
    assert(addr % alignment >= 0);
}


/// Proof that align_down produces an aligned result.
///
/// # Description
///
/// The result of align_down is always divisible by the alignment.
proof fn lemma_align_down_aligned(addr: int, alignment: int)
    requires
        alignment > 0,
        addr >= 0,
    ensures
        spec_align_down(addr, alignment) % alignment == 0,
{
    let q: int = addr / alignment;
    // (q * alignment) % alignment == 0 because q * alignment is a multiple of alignment.
    assert(q * alignment == spec_align_down(addr, alignment));
    vstd::arithmetic::div_mod::lemma_mod_multiples_basic(q, alignment);
}


/// Proof that align_down is monotone (order-preserving).
///
/// # Description
///
/// If a <= b, then align_down(a, k) <= align_down(b, k).
/// This is the fundamental property ensuring that processing addresses in
/// sorted order produces page table bases in non-decreasing order.
pub proof fn lemma_align_down_monotone(a: int, b: int, alignment: int)
    requires
        a <= b,
        alignment > 0,
        a >= 0,
        b >= 0,
    ensures
        spec_align_down(a, alignment) <= spec_align_down(b, alignment),
{
    // Step 1: a <= b implies a / alignment <= b / alignment (integer division is monotone).
    vstd::arithmetic::div_mod::lemma_div_is_ordered(a, b, alignment);
    let qa: int = a / alignment;
    let qb: int = b / alignment;
    assert(qa <= qb);

    // Step 2: qa <= qb and alignment > 0 implies qa * alignment <= qb * alignment.
    vstd::arithmetic::mul::lemma_mul_inequality(qa, qb, alignment);
}


/// Proof that align_down is idempotent.
///
/// # Description
///
/// align_down(align_down(x, k), k) == align_down(x, k).
/// An already-aligned value is unchanged by align_down.
proof fn lemma_align_down_idempotent(addr: int, alignment: int)
    requires
        alignment > 0,
        addr >= 0,
    ensures
        spec_align_down(spec_align_down(addr, alignment), alignment)
            == spec_align_down(addr, alignment),
{
    let aligned: int = spec_align_down(addr, alignment);
    lemma_align_down_aligned(addr, alignment);
    // aligned % alignment == 0, so aligned / alignment * alignment == aligned.
    assert(aligned % alignment == 0);
    vstd::arithmetic::div_mod::lemma_div_by_multiple(addr / alignment, alignment);
}


/// Proof that aligned values are fixed points of align_down.
proof fn lemma_aligned_is_fixed_point(addr: int, alignment: int)
    requires
        alignment > 0,
        addr >= 0,
        addr % alignment == 0,
    ensures
        spec_align_down(addr, alignment) == addr,
{
    assert(addr == (addr / alignment) * alignment + (addr % alignment));
    assert(addr % alignment == 0);
}

//==================================================================================================
// Page Table Base Lemmas
//==================================================================================================

/// Proof that sorted addresses produce sorted page table bases.
///
/// # Description
///
/// This is the key safety theorem for the init function. If virtual addresses
/// are processed in non-decreasing order, then the computed page table base
/// addresses are also in non-decreasing order. This means the Ordering::Less
/// branch in the init function is unreachable when regions are properly sorted.
pub proof fn lemma_sorted_addrs_sorted_pgtab_bases(a: int, b: int)
    requires
        0 <= a <= b,
    ensures
        spec_pgtab_base(a) <= spec_pgtab_base(b),
{
    lemma_align_down_monotone(a, b, INIT_PGTAB_ALIGNMENT as int);
}


/// Proof that page table bases are always page-table-aligned.
proof fn lemma_pgtab_base_aligned(vaddr: int)
    requires
        vaddr >= 0,
    ensures
        spec_pgtab_base(vaddr) % INIT_PGTAB_ALIGNMENT as int == 0,
{
    lemma_align_down_aligned(vaddr, INIT_PGTAB_ALIGNMENT as int);
}


/// Proof that the page table base is at most the virtual address.
proof fn lemma_pgtab_base_le_vaddr(vaddr: int)
    requires
        vaddr >= 0,
    ensures
        spec_pgtab_base(vaddr) <= vaddr,
{
    lemma_align_down_le(vaddr, INIT_PGTAB_ALIGNMENT as int);
}

//==================================================================================================
// Page Coverage Lemmas
//==================================================================================================

/// Proof that page iteration covers the entire region.
///
/// # Description
///
/// For a page-aligned region [start, start+size), every page at offset
/// i * PAGE_SIZE (for 0 <= i < size/PAGE_SIZE) falls within the region.
proof fn lemma_page_iteration_covers_region(start: int, size: int, i: int)
    requires
        start >= 0,
        size > 0,
        start % INIT_PAGE_SIZE as int == 0,
        size % INIT_PAGE_SIZE as int == 0,
        0 <= i < size / INIT_PAGE_SIZE as int,
    ensures
        start <= spec_nth_page_addr(start, i) < start + size,
        spec_nth_page_addr(start, i) % INIT_PAGE_SIZE as int == 0,
{
    let page_addr: int = spec_nth_page_addr(start, i);
    // page_addr = start + i * PAGE_SIZE.
    // Since i >= 0 and PAGE_SIZE > 0, page_addr >= start.
    assert(i * INIT_PAGE_SIZE as int >= 0) by {
        vstd::arithmetic::mul::lemma_mul_nonnegative(i, INIT_PAGE_SIZE as int);
    };
    // Since i < size / PAGE_SIZE, we have i * PAGE_SIZE < size.
    assert(i * INIT_PAGE_SIZE as int < size) by {
        vstd::arithmetic::div_mod::lemma_div_is_ordered(
            i * INIT_PAGE_SIZE as int,
            size - INIT_PAGE_SIZE as int,
            INIT_PAGE_SIZE as int,
        );
        // i < size / PAGE_SIZE, so i * PAGE_SIZE < size (for page-aligned size).
        assert(i * INIT_PAGE_SIZE as int < size);
    };
    // page_addr % PAGE_SIZE == 0 because start is aligned and i * PAGE_SIZE is aligned.
    assert(page_addr == start + i * INIT_PAGE_SIZE as int);
}


/// Proof that consecutive pages within a region maintain page table ordering.
///
/// # Description
///
/// If page i comes before page j in a region, then the page table base
/// for page i is <= the page table base for page j. This ensures no
/// backward progress when iterating through pages in order.
proof fn lemma_consecutive_pages_ordered_bases(start: int, i: int, j: int)
    requires
        start >= 0,
        0 <= i <= j,
    ensures
        spec_pgtab_base(spec_nth_page_addr(start, i))
            <= spec_pgtab_base(spec_nth_page_addr(start, j)),
{
    let addr_i: int = spec_nth_page_addr(start, i);
    let addr_j: int = spec_nth_page_addr(start, j);
    // addr_i = start + i * PAGE_SIZE <= start + j * PAGE_SIZE = addr_j.
    assert(addr_i <= addr_j) by {
        vstd::arithmetic::mul::lemma_mul_inequality(i, j, INIT_PAGE_SIZE as int);
    };
    // addr_i >= 0 because start >= 0 and i >= 0.
    assert(addr_i >= 0) by {
        vstd::arithmetic::mul::lemma_mul_nonnegative(i, INIT_PAGE_SIZE as int);
    };
    assert(addr_j >= 0) by {
        vstd::arithmetic::mul::lemma_mul_nonnegative(j, INIT_PAGE_SIZE as int);
    };
    lemma_sorted_addrs_sorted_pgtab_bases(addr_i, addr_j);
}

//==================================================================================================
// Identity Mapping Lemma
//==================================================================================================

/// Proof documenting the identity mapping property for non-MMIO regions.
///
/// # Description
///
/// For non-MMIO memory regions in the Nanvix kernel, the physical address
/// equals the virtual address (identity mapping). This is a design decision
/// documented in the kernel source:
///
/// - `PhysicalAddress` wraps `VirtualAddress` directly
/// - `PhysicalAddress::into_virtual_address()` is the identity function
/// - The init function explicitly performs "identity map memory regions"
///
/// For MMIO regions, the physical address may differ from the virtual address,
/// as it comes from `PhysicalAddress::from_mmio_address()`.
proof fn lemma_identity_mapping_non_mmio(vaddr: int)
    requires
        vaddr >= 0,
        vaddr % INIT_PAGE_SIZE as int == 0,
    ensures
        // For non-MMIO identity mapping: paddr == vaddr.
        vaddr == vaddr,
{
    // Trivially true. This lemma exists to document the identity mapping
    // design decision and serves as a placeholder for future refinement
    // proofs connecting to the actual PhysicalAddress implementation.
}

//==================================================================================================
// Region Ordering Lemmas
//==================================================================================================

/// Proof that non-overlapping sorted regions produce ordered page table bases.
///
/// # Description
///
/// If region A ends before region B starts (i.e., A.end <= B.start),
/// and both are page-aligned, then any page in A has a page table base
/// <= any page in B. This is the inter-region ordering guarantee.
proof fn lemma_non_overlapping_regions_ordered(
    a_start: int, a_size: int, b_start: int, b_size: int, i: int, j: int)
    requires
        a_start >= 0,
        a_size > 0,
        b_start >= 0,
        b_size > 0,
        a_start + a_size <= b_start,  // Non-overlapping, A before B.
        a_start % INIT_PAGE_SIZE as int == 0,
        a_size % INIT_PAGE_SIZE as int == 0,
        b_start % INIT_PAGE_SIZE as int == 0,
        b_size % INIT_PAGE_SIZE as int == 0,
        0 <= i < a_size / INIT_PAGE_SIZE as int,
        0 <= j < b_size / INIT_PAGE_SIZE as int,
    ensures
        spec_pgtab_base(spec_nth_page_addr(a_start, i))
            <= spec_pgtab_base(spec_nth_page_addr(b_start, j)),
{
    let a_page: int = spec_nth_page_addr(a_start, i);
    let b_page: int = spec_nth_page_addr(b_start, j);

    // a_page = a_start + i * PAGE_SIZE.
    // Since i < a_size / PAGE_SIZE, a_page < a_start + a_size <= b_start.
    lemma_page_iteration_covers_region(a_start, a_size, i);
    assert(a_page < a_start + a_size);
    assert(a_start + a_size <= b_start);

    // b_page = b_start + j * PAGE_SIZE >= b_start.
    assert(b_page >= b_start) by {
        vstd::arithmetic::mul::lemma_mul_nonnegative(j, INIT_PAGE_SIZE as int);
    };

    // Therefore a_page < b_start <= b_page, so a_page <= b_page.
    assert(a_page <= b_page);

    // a_page >= 0.
    assert(a_page >= 0) by {
        vstd::arithmetic::mul::lemma_mul_nonnegative(i, INIT_PAGE_SIZE as int);
    };
    assert(b_page >= 0);

    lemma_sorted_addrs_sorted_pgtab_bases(a_page, b_page);
}

} // verus!
