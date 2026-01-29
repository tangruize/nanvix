# Review: kstack (claude-opus-4.5)

## Grade: B+

## Issues Found

### Critical

None.

### High

- **Location**: `KernelStack::new()` signature difference
- **Description**: The original implementation takes `&mut VirtMemoryManager` and calls `mm.alloc_kpages()` to allocate pages. The verified version takes `(base_addr: usize, num_pages: usize)` directly, completely abstracting away the memory allocator interaction. This means the verified code cannot prove that:
  1. The allocation actually succeeds when memory is available.
  2. The returned pages are contiguous (relies on `VirtMemoryManager` contract).
  3. Pages are properly deallocated on drop.
- **Suggested Fix**: Model `VirtMemoryManager` as a tracked resource with a spec for `alloc_kpages` that guarantees contiguous page allocation, or document this abstraction boundary explicitly and verify the allocator separately.

- **Location**: `Drop` trait implementation
- **Description**: The original has a `Drop` implementation that pops and drops each `KernelPage`. The verified version has no `Drop` implementation, meaning resource cleanup semantics are not verified. This is a critical gap for memory safety in a kernel.
- **Suggested Fix**: Add a verified `Drop` or explicit `release()` method that proves all pages are returned to the allocator.

### Medium

- **Location**: `size()` method semantics
- **Description**: Original `size()` returns the constant `config::kernel::KSTACK_SIZE` (32768 bytes = 8 pages), while verified version computes `num_pages * PAGE_SIZE`. The original always returns 32KB regardless of actual allocation, while verified dynamically computes size. This is semantically different.
- **Suggested Fix**: Add constant `KSTACK_SIZE` and verify that `num_pages * PAGE_SIZE == KSTACK_SIZE` invariant, or document the abstraction choice.

- **Location**: `base()` return type
- **Description**: Original returns `PageAligned<VirtualAddress>` (a typed wrapper ensuring alignment), while verified returns raw `usize`. The original's type system enforces alignment at the type level; verified only asserts it in postconditions.
- **Suggested Fix**: Consider adding a `PageAligned` newtype wrapper in the verified code to match original's stronger typing.

- **Location**: Added functions not in original
- **Description**: Verified code adds `num_pages()`, `contains()`, `page_index()`, `initial_sp()`, and `has_room()` methods that don't exist in the original. While these are useful for verification, they change the API surface.
- **Suggested Fix**: Mark these as `#[doc(hidden)]` or move to a separate verification-only trait, or add them to the original implementation.

### Low

- **Location**: `fmt::Debug` implementation
- **Description**: Original has custom `Debug` formatting. Verified code uses `#[derive(Debug)]` which produces different output format. Minor cosmetic difference.
- **Suggested Fix**: Not critical, but could add custom `Debug` for parity.

- **Location**: `DEFAULT_KSTACK_PAGES` constant
- **Description**: Verified defines `DEFAULT_KSTACK_PAGES = 4` but the actual config uses 32768/4096 = 8 pages. This constant is unused and potentially misleading.
- **Suggested Fix**: Either remove unused constant or set to correct value (8).

- **Location**: Proof lemmas
- **Description**: `lemma_well_formed_has_aligned_top`, `lemma_pages_disjoint`, and `lemma_page_in_bounds` are defined but not called from any exec code or other proofs. They may be useful documentation but add verification overhead.
- **Suggested Fix**: Document their purpose or integrate them into the main invariant proofs.

## Positive Observations

- **No unsafe assume/external_body**: The verified code contains no `assume`, `external_body`, or `trusted` annotations, meaning all proofs are fully checked.

- **Strong invariants**: The `inv()` spec covers comprehensive properties:
  - Page alignment of base, size, and top
  - Non-empty guarantee (num_pages > 0)
  - No overflow protection
  - Page contiguity
  - Top > base ordering

- **Well-structured abstraction**: `KernelStackView` cleanly separates specification from implementation, following Verus best practices.

- **Address containment proofs**: The `contains()` and `page_index()` functions with their specs enable reasoning about stack bounds, useful for verifying stack overflow protection.

- **Page disjointness lemma**: `lemma_pages_disjoint` proves non-overlapping pages, important for memory isolation.

- **Overflow protection**: The `no_overflow` spec and runtime check in `new()` properly guard against address arithmetic overflow.

- **All 18 verification conditions pass**: Clean verification with no errors.

## Summary

The verified implementation provides solid formal guarantees for the core kernel stack data structure, including page alignment, contiguity, and bounds safety. The verification successfully proves the key memory safety invariants without relying on assumptions.

However, there are significant abstraction gaps:

1. **Allocator interaction is completely abstracted**: The original's integration with `VirtMemoryManager` and the guarantee that allocated pages are actually contiguous is assumed rather than verified.

2. **Resource cleanup is not verified**: The `Drop` implementation is missing, leaving deallocation correctness unproven.

3. **API differences**: The verified API is more general (parameterized by num_pages) while the original uses a fixed stack size from configuration.

**Recommendations**:
- Priority 1: Document the abstraction boundary at the allocator interface clearly.
- Priority 2: Add verification of cleanup/drop semantics.
- Priority 3: Align the `size()` semantics to match original's constant-based approach, or verify the abstraction is valid.

The verification is a good foundation but would benefit from tighter coupling to the original's exact semantics and coverage of resource lifecycle.
