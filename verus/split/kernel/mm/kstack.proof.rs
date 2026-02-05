// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Proofs and lemmas.

verus! {

//==================================================================================================

/// Lemma: Page alignment is preserved under addition of page-aligned values.
proof fn lemma_page_aligned_add(a: int, b: int)
    requires
        spec_is_page_aligned(a),
        spec_is_page_aligned(b),
    ensures
        spec_is_page_aligned(a + b),
{
    assert((a + b) % (PAGE_SIZE as int) == 0) by {
        assert(a % (PAGE_SIZE as int) == 0);
        assert(b % (PAGE_SIZE as int) == 0);
    }
}


/// Lemma: Page multiplication produces page-aligned results.
proof fn lemma_page_mult_aligned(n: int)
    requires
        n >= 0,
    ensures
        spec_is_page_aligned(n * (PAGE_SIZE as int)),
{
    assert((n * (PAGE_SIZE as int)) % (PAGE_SIZE as int) == 0);
}


/// Lemma: A well-formed stack has aligned top.
proof fn lemma_well_formed_has_aligned_top(view: KernelStackView)
    requires
        view.is_well_formed(),
    ensures
        view.top_is_aligned(),
{
    lemma_page_mult_aligned(view.num_pages);
    lemma_page_aligned_add(view.base_addr, view.size());
}


/// Lemma: Stack pages are disjoint from each other.
proof fn lemma_pages_disjoint(view: KernelStackView, i: int, j: int)
    requires
        view.is_well_formed(),
        0 <= i < view.num_pages,
        0 <= j < view.num_pages,
        i != j,
    ensures
        view.page_end(i) <= view.page_start(j) || view.page_end(j) <= view.page_start(i),
{
    // Pages are laid out sequentially, so if i < j, page i ends before page j starts.
    if i < j {
        assert(view.page_end(i) == view.base_addr + (i + 1) * (PAGE_SIZE as int));
        assert(view.page_start(j) == view.base_addr + j * (PAGE_SIZE as int));
        assert(i + 1 <= j);
        assert(view.page_end(i) <= view.page_start(j));
    } else {
        // j < i
        assert(view.page_end(j) == view.base_addr + (j + 1) * (PAGE_SIZE as int));
        assert(view.page_start(i) == view.base_addr + i * (PAGE_SIZE as int));
        assert(j + 1 <= i);
        assert(view.page_end(j) <= view.page_start(i));
    }
}


/// Lemma: All addresses in a page are within stack bounds.
proof fn lemma_page_in_bounds(view: KernelStackView, page_idx: int, offset: int)
    requires
        view.is_well_formed(),
        0 <= page_idx < view.num_pages,
        0 <= offset < PAGE_SIZE as int,
    ensures
        view.contains_addr(view.page_start(page_idx) + offset),
{
    let addr = view.page_start(page_idx) + offset;
    assert(addr >= view.base_addr);
    assert(addr < view.page_end(page_idx));
    assert(view.page_end(page_idx) <= view.top());
    assert(addr < view.top());
}

} // verus!
