# Review: ustack (claude-opus-4.5)

## Grade: A

## Issues Found

### Critical

None.

### High

None.

### Medium

1. **Location:** Constructor signature difference (`new`)
   - **Description:** The original `UserStack::new(base: PageAligned<VirtualAddress>) -> Self` is infallible, taking a type-level guaranteed aligned address. The verified version `new(base_addr: usize) -> Result<Self, Error>` returns a `Result`. While the postcondition proves `result.is_ok()` when preconditions hold, this is a semantic change: original callers never handle errors.
   - **Suggested Fix:** Consider adding `new_unchecked(base_addr: usize) -> Self` that requires preconditions and returns `Self` directly, mirroring the original infallible API. The existing `new` can remain for defensive use.

2. **Location:** Constants duplication (`PAGE_SIZE`, `USER_STACK_SIZE`, etc.)
   - **Description:** Constants are duplicated from kernel config (`config::memory_layout::USER_STACK_SIZE = 512 * KILOBYTE = 524288`, `arch::PAGE_SIZE = 1 << 12 = 4096`). The module correctly notes this and mentions CI verification, but the CI script `scripts/verify-verus-constants.sh` referenced does not appear to exist.
   - **Suggested Fix:** Create the mentioned CI script or add a compile-time assertion mechanism that validates constant consistency.

### Low

1. **Location:** Return type abstraction (`base()`, `top()`)
   - **Description:** The original returns `PageAligned<VirtualAddress>`; the verified version returns `usize` with alignment postconditions. This is a reasonable abstraction, and the `PageAlignedAddr` wrapper type provides an alternative type-safe API via `base_aligned()` and `top_aligned()`.
   - **Suggested Fix:** None needed. The dual API (raw `usize` and `PageAlignedAddr`) provides flexibility.

2. **Location:** Extended API (`contains`, `page_index`, `initial_sp`, `has_room`)
   - **Description:** These methods do not exist in the original module. They demonstrate additional verified properties but extend beyond strict equivalence.
   - **Suggested Fix:** None needed. Extensions are clearly documented and useful for verifying client code.

3. **Location:** `Debug` trait implementation
   - **Description:** The original has `impl fmt::Debug for UserStack` for formatting. The verified version uses `#[derive(Debug)]` which is equivalent for printing but loses the custom format string. This is a formatting detail, not a correctness issue.
   - **Suggested Fix:** Acceptable difference; custom debug formatting is not a verification concern.

## Positive Observations

1. **No `assume` or `external_body`:** The verification is fully self-contained with zero unjustified assumptions. All 27 verification conditions pass cleanly.

2. **Comprehensive invariant:** The `inv()` predicate covers all essential properties:
   - Base alignment (`is_well_formed`)
   - No overflow in address arithmetic
   - Constant size (`USER_STACK_SIZE`)
   - Correct top computation (`top = base + size`)
   - Size and top alignment
   - Top > base ordering
   - Page contiguity

3. **Strong specification layer:** The `UserStackView` abstraction cleanly separates specification from implementation with well-defined spec functions:
   - `size()`, `num_pages()`, `top()`
   - `is_base_aligned()`, `size_is_aligned()`, `top_is_aligned()`
   - `no_overflow()`, `top_greater_than_base()`
   - `pages_are_contiguous()`, `contains_addr()`, `addr_in_page()`

4. **Complete function coverage:**
   - `new()` ✓ (with proven success under preconditions)
   - `size()` ✓ (returns constant, proves alignment)
   - `base()` ✓ (proves page alignment)
   - `top()` ✓ (proves alignment, correct computation, ordering)
   - `fmt::Debug` — formatting trait, not verification-relevant

5. **Verified safety properties:**
   - Page alignment preserved for base, top, and all derived addresses
   - No arithmetic overflow in `base + size` computation
   - Bounds correctness for `contains` and `page_index`
   - Stack growth validation via `has_room`

6. **Proven algebraic lemmas:**
   - `lemma_page_aligned_add`: alignment preservation under addition
   - `lemma_page_mult_aligned`: multiples of page size are aligned
   - `lemma_user_stack_size_aligned`: USER_STACK_SIZE is page-aligned
   - `lemma_well_formed_has_aligned_top`: well-formed stacks have aligned top
   - `lemma_pages_disjoint`: stack pages do not overlap
   - `lemma_page_in_bounds`: addresses within pages are within stack bounds

7. **Excellent documentation:** The module header explains:
   - Return type abstraction decisions
   - Constructor signature differences
   - API equivalence reasoning
   - The base/top documentation bug in the original (lines 59, 76-77 say base is highest, but implementation computes `top = base + size`)

8. **Type-safe API option:** The `PageAlignedAddr` newtype and `base_aligned()` / `top_aligned()` methods mirror the original's type-level alignment guarantees.

9. **Correct constant values:** Verified against kernel sources:
   - `PAGE_SIZE = 4096` matches `arch::x86::mem::constants::PAGE_SIZE = 1 << 12 = 4096` ✓
   - `USER_STACK_SIZE = 524288` matches `config::memory_layout::USER_STACK_SIZE = 512 * 1024 = 524288` ✓
   - `USER_STACK_PAGES = 128` matches `524288 / 4096 = 128` ✓

## Summary

The verification of `ustack.rs` is **high quality** and well-suited for a kernel memory management component. All core functions from the original source are verified with appropriate pre/postconditions. The invariant is comprehensive and sufficient to prove the essential memory safety properties: page alignment, no overflow, correct address arithmetic, address ordering, and bounds validity.

**Key strengths:**
- Zero `assume` or `external_body` usage — fully self-contained verification
- Rich specification layer with `UserStackView` providing clean abstraction
- Comprehensive invariant covering all memory safety properties
- Useful algebraic lemmas that could be reused for related modules
- Correctly identifies documentation bug in original source

**Minor gaps:**
- Constructor returns `Result` vs. being infallible (mitigated by postcondition)
- Constants are duplicated without automated synchronization check
- CI verification script mentioned but not verified to exist

**Recommendations:**
1. Create `scripts/verify-verus-constants.sh` or equivalent to automate constant verification
2. Consider adding `new_unchecked()` for callers who prove preconditions
3. File issue to fix original documentation (base/top confusion at lines 59, 76-77)

The verification captures all essential correctness properties for a user stack data structure and is production-ready. The abstraction decisions are well-documented and justified.
