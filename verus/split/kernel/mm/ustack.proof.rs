// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Proofs and lemmas.

verus! {


/// Lemma: Verify internal consistency of constants.
///
/// This proves that our constant definitions are internally consistent.
/// External linkage (matching kernel config) must be verified separately.
proof fn lemma_constants_valid()
    ensures
        PAGE_SIZE == 4096,
        PAGE_ALIGNMENT == PAGE_SIZE,
        USER_STACK_SIZE == 512 * KILOBYTE,
        USER_STACK_PAGES == USER_STACK_SIZE / PAGE_SIZE,
        USER_STACK_SIZE % PAGE_SIZE == 0,
        KILOBYTE == 1024,
{
    assert(PAGE_SIZE == 4096usize);
    assert(PAGE_ALIGNMENT == 4096usize);
    assert(KILOBYTE == 1024usize);
    assert(512usize * 1024usize == 524288usize);
    assert(USER_STACK_SIZE == 524288usize);
    assert(524288usize / 4096usize == 128usize);
    assert(USER_STACK_PAGES == 128usize);
    assert(524288usize % 4096usize == 0usize);
}

//==================================================================================================

/// Lemma: Page alignment is preserved under addition of page-aligned values.
///
/// Uses modular arithmetic: (a + b) % p = ((a % p) + (b % p)) % p = (0 + 0) % p = 0.
proof fn lemma_page_aligned_add(a: int, b: int)
    requires
        spec_is_page_aligned(a),
        spec_is_page_aligned(b),
    ensures
        spec_is_page_aligned(a + b),
{
    let p = PAGE_SIZE as int;
    // By preconditions: a % p == 0 and b % p == 0.
    // By modular arithmetic: (a + b) % p == ((a % p) + (b % p)) % p == (0 + 0) % p == 0.
    assert(a % p == 0);
    assert(b % p == 0);
    assert((a + b) % p == 0);
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


/// Lemma: USER_STACK_SIZE is page-aligned.
proof fn lemma_user_stack_size_aligned()
    ensures
        spec_is_size_aligned(USER_STACK_SIZE as int),
        USER_STACK_SIZE as int == USER_STACK_PAGES as int * (PAGE_SIZE as int),
{
    assert(USER_STACK_SIZE == 524288);
    assert(PAGE_SIZE == 4096);
    assert(USER_STACK_PAGES == 128);
    assert(128 * 4096 == 524288);
}


/// Lemma: A well-formed stack has aligned top.
proof fn lemma_well_formed_has_aligned_top(view: UserStackView)
    requires
        view.is_well_formed(),
    ensures
        view.top_is_aligned(),
{
    lemma_user_stack_size_aligned();
    lemma_page_aligned_add(view.base_addr, view.size());
}


/// Lemma: Stack pages are disjoint from each other.
proof fn lemma_pages_disjoint(view: UserStackView, i: int, j: int)
    requires
        view.is_well_formed(),
        0 <= i < view.num_pages(),
        0 <= j < view.num_pages(),
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
proof fn lemma_page_in_bounds(view: UserStackView, page_idx: int, offset: int)
    requires
        view.is_well_formed(),
        0 <= page_idx < view.num_pages(),
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
