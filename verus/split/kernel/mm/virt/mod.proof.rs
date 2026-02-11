// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Proofs and lemmas for the virtual memory init module.

verus! {

//==================================================================================================
// Helper struct for proof methods callable from exec code.
//==================================================================================================

/// Helper struct providing verified proof methods for the init module.
///
/// # Description
///
/// This struct exists solely to provide proof methods that can be called
/// from exec code via `VirtProofs::method_name(...)`. This pattern is
/// necessary because standalone proof functions defined in separate
/// `verus!` blocks cannot be directly called from other `verus!` blocks.
pub struct VirtProofs;

impl VirtProofs {
    //==============================================================================================
    // Alignment Proofs
    //==============================================================================================

    /// Proof that align_down produces a value <= the input.
    ///
    /// # Description
    ///
    /// For any positive alignment, the aligned-down value is at most the original.
    /// This follows from the floor-division property: (a / k) * k <= a.
    pub proof fn lemma_align_down_le(addr: int, alignment: int)
        requires
            alignment > 0,
            addr >= 0,
        ensures
            spec_align_down(addr, alignment) <= addr,
    {
        assert(addr == (addr / alignment) * alignment + (addr % alignment));
        assert(addr % alignment >= 0);
    }


    /// Proof that align_down produces an aligned result.
    ///
    /// # Description
    ///
    /// The result of align_down is always divisible by the alignment.
    pub proof fn lemma_align_down_aligned(addr: int, alignment: int)
        requires
            alignment > 0,
            addr >= 0,
        ensures
            spec_align_down(addr, alignment) % alignment == 0,
    {
        let q: int = addr / alignment;
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
        vstd::arithmetic::div_mod::lemma_div_is_ordered(a, b, alignment);
        let qa: int = a / alignment;
        let qb: int = b / alignment;
        assert(qa <= qb);
        vstd::arithmetic::mul::lemma_mul_inequality(qa, qb, alignment);
    }


    /// Proof that align_down is idempotent.
    ///
    /// # Description
    ///
    /// align_down(align_down(x, k), k) == align_down(x, k).
    pub proof fn lemma_align_down_idempotent(addr: int, alignment: int)
        requires
            alignment > 0,
            addr >= 0,
        ensures
            spec_align_down(spec_align_down(addr, alignment), alignment)
                == spec_align_down(addr, alignment),
    {
        let aligned: int = spec_align_down(addr, alignment);
        Self::lemma_align_down_aligned(addr, alignment);
        assert(aligned % alignment == 0);
        vstd::arithmetic::div_mod::lemma_div_by_multiple(addr / alignment, alignment);
    }


    /// Proof that aligned values are fixed points of align_down.
    pub proof fn lemma_aligned_is_fixed_point(addr: int, alignment: int)
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

    //==============================================================================================
    // Page Table Base Proofs
    //==============================================================================================

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
        Self::lemma_align_down_monotone(a, b, INIT_PGTAB_ALIGNMENT as int);
    }


    /// Proof that page table bases are always page-table-aligned.
    pub proof fn lemma_pgtab_base_aligned(vaddr: int)
        requires
            vaddr >= 0,
        ensures
            spec_pgtab_base(vaddr) % INIT_PGTAB_ALIGNMENT as int == 0,
    {
        Self::lemma_align_down_aligned(vaddr, INIT_PGTAB_ALIGNMENT as int);
    }


    /// Proof that the page table base is at most the virtual address.
    pub proof fn lemma_pgtab_base_le_vaddr(vaddr: int)
        requires
            vaddr >= 0,
        ensures
            spec_pgtab_base(vaddr) <= vaddr,
    {
        Self::lemma_align_down_le(vaddr, INIT_PGTAB_ALIGNMENT as int);
    }

    //==============================================================================================
    // Page Coverage Proofs
    //==============================================================================================

    /// Proof that page iteration covers the entire region.
    ///
    /// # Description
    ///
    /// For a page-aligned region [start, start+size), every page at offset
    /// i * PAGE_SIZE (for 0 <= i < size/PAGE_SIZE) falls within the region.
    pub proof fn lemma_page_iteration_covers_region(start: int, size: int, i: int)
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
        let ps: int = INIT_PAGE_SIZE as int;
        let page_addr: int = spec_nth_page_addr(start, i);
        // page_addr >= start because i >= 0 and PAGE_SIZE > 0.
        vstd::arithmetic::mul::lemma_mul_nonnegative(i, ps);
        // i * PAGE_SIZE < size because i < size / PAGE_SIZE (and size is page-aligned).
        // From i < size / PAGE_SIZE: i * PAGE_SIZE < size.
        // Proof: size = (size / ps) * ps + size % ps = (size / ps) * ps (since size % ps == 0).
        // i < size / ps, so i + 1 <= size / ps, so (i + 1) * ps <= (size / ps) * ps = size.
        // Therefore i * ps + ps <= size, i.e., i * ps <= size - ps < size.
        vstd::arithmetic::mul::lemma_mul_inequality(i + 1, size / ps, ps);
        assert((i + 1) * ps <= (size / ps) * ps);
        assert(size == (size / ps) * ps + size % ps);
        assert(size % ps == 0);
        assert((i + 1) * ps <= size);
        vstd::arithmetic::mul::lemma_mul_is_distributive_add(ps, i, 1);
        assert((i + 1) * ps == i * ps + 1 * ps);
        assert(i * ps < size);
        // page_addr = start + i * ps, and page_addr % ps == 0.
        assert(page_addr == start + i * ps);
    }


    /// Proof that consecutive pages within a region maintain page table ordering.
    ///
    /// # Description
    ///
    /// If page i comes before page j in a region, then the page table base
    /// for page i is <= the page table base for page j.
    pub proof fn lemma_consecutive_pages_ordered_bases(start: int, i: int, j: int)
        requires
            start >= 0,
            0 <= i <= j,
        ensures
            spec_pgtab_base(spec_nth_page_addr(start, i))
                <= spec_pgtab_base(spec_nth_page_addr(start, j)),
    {
        let addr_i: int = spec_nth_page_addr(start, i);
        let addr_j: int = spec_nth_page_addr(start, j);
        let ps: int = INIT_PAGE_SIZE as int;
        vstd::arithmetic::mul::lemma_mul_inequality(i, j, ps);
        assert(addr_i <= addr_j);
        vstd::arithmetic::mul::lemma_mul_nonnegative(i, ps);
        vstd::arithmetic::mul::lemma_mul_nonnegative(j, ps);
        assert(addr_i >= 0);
        assert(addr_j >= 0);
        Self::lemma_sorted_addrs_sorted_pgtab_bases(addr_i, addr_j);
    }

    //==============================================================================================
    // Identity Mapping Proof
    //==============================================================================================

    /// Proof documenting the identity mapping property for non-MMIO regions.
    ///
    /// # Description
    ///
    /// For non-MMIO memory regions in the Nanvix kernel, the physical address
    /// equals the virtual address (identity mapping). Evidenced by:
    /// - `PhysicalAddress` wraps `VirtualAddress` directly.
    /// - `PhysicalAddress::into_virtual_address()` is the identity function.
    /// - The init function explicitly performs "identity map memory regions".
    pub proof fn lemma_identity_mapping_non_mmio(vaddr: int)
        requires
            vaddr >= 0,
            vaddr % INIT_PAGE_SIZE as int == 0,
        ensures
            vaddr == vaddr,
    {
        // Trivially true. Documents the identity mapping design decision.
    }

    //==============================================================================================
    // Region Ordering Proofs
    //==============================================================================================

    /// Proof that non-overlapping sorted regions produce ordered page table bases.
    ///
    /// # Description
    ///
    /// If region A ends before region B starts (A.end <= B.start),
    /// and both are page-aligned, then any page in A has a page table base
    /// <= any page in B.
    pub proof fn lemma_non_overlapping_regions_ordered(
        a_start: int, a_size: int, b_start: int, b_size: int, i: int, j: int)
        requires
            a_start >= 0,
            a_size > 0,
            b_start >= 0,
            b_size > 0,
            a_start + a_size <= b_start,
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
        let ps: int = INIT_PAGE_SIZE as int;

        Self::lemma_page_iteration_covers_region(a_start, a_size, i);
        assert(a_page < a_start + a_size);
        assert(a_start + a_size <= b_start);

        vstd::arithmetic::mul::lemma_mul_nonnegative(j, ps);
        assert(b_page >= b_start);
        assert(a_page <= b_page);

        vstd::arithmetic::mul::lemma_mul_nonnegative(i, ps);
        assert(a_page >= 0);
        assert(b_page >= 0);

        Self::lemma_sorted_addrs_sorted_pgtab_bases(a_page, b_page);
    }
}

} // verus!
