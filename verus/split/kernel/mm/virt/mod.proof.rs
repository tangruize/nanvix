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
    pub proof fn lemma_align_down_le(addr: int, alignment: int)
        requires
            alignment > 0,
            addr >= 0,
        ensures
            spec_align_down(addr, alignment) <= addr,
    {
        vstd::arithmetic::div_mod::lemma_fundamental_div_mod(addr, alignment);
        vstd::arithmetic::mul::lemma_mul_is_commutative(alignment, addr / alignment);
        assert(addr % alignment >= 0);
    }


    /// Proof that align_down produces an aligned result.
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
        vstd::arithmetic::div_mod::lemma_fundamental_div_mod(addr, alignment);
        vstd::arithmetic::mul::lemma_mul_is_commutative(alignment, addr / alignment);
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
        vstd::arithmetic::mul::lemma_mul_nonnegative(i, ps);
        vstd::arithmetic::mul::lemma_mul_inequality(i + 1, size / ps, ps);
        assert((i + 1) * ps <= (size / ps) * ps);
        assert(size == (size / ps) * ps + size % ps);
        assert(size % ps == 0);
        assert((i + 1) * ps <= size);
        vstd::arithmetic::mul::lemma_mul_is_distributive_add(ps, i, 1);
        assert((i + 1) * ps == i * ps + 1 * ps);
        assert(i * ps < size);
        assert(page_addr == start + i * ps);
    }


    /// Proof that consecutive pages within a region maintain page table ordering.
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
    // Loop Bound Reconciliation Proofs
    //==============================================================================================

    /// Proof reconciling the original loop bound with the spec page count.
    ///
    /// # Description
    ///
    /// The original computes `end = raw_vaddr + (region.size() - 1)` and loops
    /// `while raw_vaddr < end`. The spec uses `i < size / PAGE_SIZE` as the page count.
    ///
    /// For a page-aligned region of size S starting at A:
    /// - The last page address is `A + S - PAGE_SIZE`.
    /// - The loop condition `A + S - PAGE_SIZE < A + S - 1` holds because PAGE_SIZE >= 2.
    /// - Therefore all `S / PAGE_SIZE` pages are visited.
    ///
    /// # Single-Page Edge Case
    ///
    /// When S == PAGE_SIZE (single page): `end = A + PAGE_SIZE - 1 = A + 4095`.
    /// Since `A < A + 4095`, the loop body executes exactly once, mapping the page.
    /// This is correct: `S / PAGE_SIZE = 1`, so exactly one page should be mapped.
    pub proof fn lemma_loop_bound_matches_page_count(start: int, size: int)
        requires
            start >= 0,
            size > 0,
            size % INIT_PAGE_SIZE as int == 0,
            INIT_PAGE_SIZE > 1,
        ensures
            // The loop condition admits exactly size/PAGE_SIZE iterations.
            // Last page addr < end (loop continues through all pages).
            spec_nth_page_addr(start, size / INIT_PAGE_SIZE as int - 1)
                < spec_loop_end(start, size),
            // Single-page case: loop executes at least once.
            size == INIT_PAGE_SIZE as int ==> start < spec_loop_end(start, size),
    {
        let ps: int = INIT_PAGE_SIZE as int;
        let n: int = size / ps;
        let last_page: int = spec_nth_page_addr(start, n - 1);
        let end: int = spec_loop_end(start, size);
        // last_page = start + (n-1) * ps = start + n*ps - ps = start + size - ps.
        // end = start + size - 1.
        // last_page < end iff start + size - ps < start + size - 1 iff ps > 1.
        vstd::arithmetic::mul::lemma_mul_is_distributive_sub(ps, n, 1);
        assert(n * ps == size);
        assert((n - 1) * ps == n * ps - 1 * ps);
        assert(last_page == start + size - ps);
        assert(end == start + size - 1);
        assert(ps > 1);
    }


    /// Proof that a valid MemRegion does not overflow on the end computation.
    ///
    /// # Description
    ///
    /// The original `let end: usize = raw_vaddr + (region.size() - 1);` requires
    /// that `start + size - 1 <= usize::MAX`. This is guaranteed by
    /// `MemRegion::spec_is_valid()` which requires `start + size - 1 <= usize::MAX`.
    pub proof fn lemma_end_no_overflow(start: int, size: int)
        requires
            start >= 0,
            size > 0,
            start + size - 1 <= usize::MAX as int,
        ensures
            spec_loop_end(start, size) >= 0,
            spec_loop_end(start, size) <= usize::MAX as int,
    {
        // Trivially follows from spec_loop_end(start, size) = start + size - 1.
    }

    //==============================================================================================
    // Identity Mapping Proof
    //==============================================================================================

    /// Proof that non-MMIO init paddr equals vaddr (identity mapping).
    ///
    /// # Description
    ///
    /// For non-MMIO memory regions in the Nanvix kernel, the init function
    /// computes `paddr = FrameAddress::new(PageAligned::from_address(
    /// PhysicalAddress::from_raw_value(raw_vaddr)))`. Since PhysicalAddress
    /// wraps VirtualAddress directly and identity mapping is used for kernel
    /// memory (see virt/mod.rs line 125: "Identity map memory regions"), the
    /// physical address equals the virtual address.
    ///
    /// This is verified by the chain:
    /// - `PhysicalAddress::from_raw_value(raw_vaddr)` creates a PhysicalAddress = vaddr
    /// - `PageAligned::from_address(phys_addr)` preserves the value (page-aligned input)
    /// - `FrameAddress::new(page_aligned)` preserves the value
    pub proof fn lemma_non_mmio_paddr_is_identity(vaddr: int)
        requires
            vaddr >= 0,
            vaddr % INIT_PAGE_SIZE as int == 0,
        ensures
            spec_init_paddr(vaddr, vaddr, false) == vaddr,
    {
        // Follows from definition: spec_init_paddr(vaddr, _, false) == vaddr.
    }


    /// Proof that MMIO paddr is constant across all pages in a region.
    ///
    /// # Description
    ///
    /// In the original init(), MMIO paddr is recomputed on each iteration using
    /// `region.start()` (NOT the current `raw_vaddr`). This means all pages in
    /// an MMIO region map to the SAME physical frame. This behavior is captured
    /// by `spec_init_paddr(_, region_start, true) == spec_mmio_paddr(region_start)`.
    ///
    /// This is a potential bug in the original code (all MMIO pages sharing a frame),
    /// but the verification faithfully models the actual behavior.
    pub proof fn lemma_mmio_paddr_constant(vaddr1: int, vaddr2: int, region_start: int)
        requires
            vaddr1 >= 0,
            vaddr2 >= 0,
        ensures
            spec_init_paddr(vaddr1, region_start, true)
                == spec_init_paddr(vaddr2, region_start, true),
    {
        // Both equal spec_mmio_paddr(region_start) by definition.
    }

    //==============================================================================================
    // Region Ordering Proofs
    //==============================================================================================

    /// Proof that non-overlapping sorted regions produce ordered page table bases.
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

    //==============================================================================================
    // Page Table Decision Proof
    //==============================================================================================

    /// Proof that the Ordering::Less branch is unreachable for sorted inputs.
    ///
    /// # Description
    ///
    /// When processing pages in non-decreasing virtual address order, the
    /// page table base for the current address is always >= the base for the
    /// previous address. Therefore `PgtabDecision::Overlap` never occurs.
    pub proof fn lemma_no_overlap_for_sorted_inputs(prev_vaddr: int, curr_vaddr: int)
        requires
            0 <= prev_vaddr <= curr_vaddr,
        ensures
            spec_pgtab_base(curr_vaddr) >= spec_pgtab_base(prev_vaddr),
    {
        Self::lemma_sorted_addrs_sorted_pgtab_bases(prev_vaddr, curr_vaddr);
    }
}

} // verus!
