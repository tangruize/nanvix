# Review: ustack Exec Consistency (claude-opus-4.6)

## Grade: A

## Issues Found

### Critical

- None.

### Minor

1. **Debug format discrepancy**: The original `fmt::Debug` prints `self.base` and `self.top()` which are `PageAligned<VirtualAddress>` values (using their own `Debug` impl, which may format differently from raw `usize`). The Verus version prints `self.base_addr` and `self.base_addr + USER_STACK_SIZE` as raw `usize`. The format string template is identical (`"UserStack {{ base: {:?}, top: {:?}, size={:?} }}"`), but the rendered output will differ if `PageAligned<VirtualAddress>`'s `Debug` impl wraps the value (e.g., `PageAligned(VirtualAddress(0x1000))` vs `4096`). This is cosmetic and acceptable for verification purposes, but worth noting.

2. **`has_room` subtraction could underflow**: The exec body `current_sp - self.base_addr >= growth` could underflow if `current_sp < self.base_addr`. However, the precondition `self@.contains_addr(current_sp as int) || current_sp as int == self.spec_top()` guarantees `current_sp >= self.base_addr`, so this is safe. The precondition is sound, but this is a subtle correctness dependency worth highlighting.

3. **`try_new` dead code paths**: The alignment and overflow checks in `try_new` (lines 297-303) are dead code since the preconditions already guarantee alignment and no overflow. The ensures clause proves `result.is_ok()` unconditionally. This is documented as "defense-in-depth" but could mislead readers into thinking the function is truly fallible. The `Result` return type is somewhat misleading given the preconditions make errors unreachable.

## Consistency Assessment

### Original API Coverage (5 items)

| Original Function | Verus Equivalent | Status |
|---|---|---|
| `UserStack::new(PageAligned<VirtualAddress>) -> Self` | `UserStack::new(PageAlignedAddr) -> Self` | ✅ Equivalent — type substitution is sound |
| `UserStack::size(&self) -> usize` | `UserStack::size(&self) -> usize` | ✅ Identical body (`USER_STACK_SIZE`) |
| `UserStack::base(&self) -> PageAligned<VirtualAddress>` | `UserStack::base(&self) -> PageAlignedAddr` | ✅ Equivalent — returns wrapped base |
| `UserStack::top(&self) -> PageAligned<VirtualAddress>` | `UserStack::top(&self) -> PageAlignedAddr` | ✅ Equivalent — `from_raw_unchecked` replaces `from_raw_value().unwrap()` with proven alignment |
| `fmt::Debug for UserStack` | `fmt::Debug for UserStack` | ✅ Added — format string matches, placed outside `verus!` block |

### Type Abstraction (PageAlignedAddr ↔ PageAligned\<VirtualAddress\>)

The `PageAlignedAddr` wrapper is a well-justified stand-in for `PageAligned<VirtualAddress>`:
- Invariant (`addr % PAGE_SIZE == 0`) matches the kernel type's guarantee.
- `from_raw_unchecked` with a precondition is semantically equivalent to `from_raw_value().unwrap()` when alignment is machine-proven.
- `into_raw()` mirrors `into_raw_value()`.
- The abstraction is necessary because Verus cannot import kernel types.

### Extension Functions (7 items)

| Function | Justification |
|---|---|
| `try_new` | Fallible constructor for raw `usize` — verification helper |
| `base_raw` / `top_raw` | Raw accessors used by `initial_sp` — reasonable helpers |
| `contains` | Bounds check — standard stack utility |
| `page_index` | Page computation — useful verified extension |
| `initial_sp` | Stack pointer initialization — practical extension |
| `has_room` | Growth check — practical extension |

All extensions are clearly documented as beyond the original API. None shadow or alter the semantics of original functions.

### Struct Field Mapping

| Original | Verus | Notes |
|---|---|---|
| `base: PageAligned<VirtualAddress>` | `base_addr: usize` | Alignment enforced via `inv()` postcondition instead of type wrapper |

### Verification Quality

- **26 verified, 0 errors** — all functions and lemmas pass.
- **No `assume`, `admit`, `trusted`, or `external_body`** — no verification shortcuts.
- Proof structure is clean: lemmas for constant validity, alignment preservation, page disjointness, and page-in-bounds.
- Spec/proof/exec separation is well-organized across three files.

### Equivalence Justifications

1. **`new` rename**: Sound. The original `from_aligned` → `new` rename correctly matches the original API. The `try_new` fallible variant is properly separated.
2. **`size` equivalence**: Sound. Body is identical (`USER_STACK_SIZE`). Verus return syntax is ghost-only.
3. **`base` equivalence**: Sound. `PageAlignedAddr::from_raw_unchecked(self.base_addr)` wraps the raw field identically to `self.base` returning the stored `PageAligned<VirtualAddress>`.
4. **`top` equivalence**: Sound. `from_raw_unchecked` with proven alignment replaces `from_raw_value().unwrap()`. `self.base_addr + USER_STACK_SIZE` ≡ `self.base.into_raw_value() + self.size()` by field/constant equivalence.
5. **`fmt` addition**: Sound. Format string is identical. Placed outside `verus!` due to Verus trait impl limitation.

### Documentation Bug Note

The consistency report correctly identifies a documentation bug in the original kernel: comments say base = "highest address" and top = "lowest address", but the implementation computes `top = base + size` (top > base). The Verus version documents the implementation-consistent semantics. This is a valid observation, not a consistency issue.

## Summary

The exec consistency fixes are thorough and well-executed. All 5 original API functions (`new`, `size`, `base`, `top`, `fmt`) are faithfully represented in the Verus exec code with sound type abstractions. The `PageAlignedAddr` wrapper properly mirrors `PageAligned<VirtualAddress>` with machine-checked alignment invariants. Extension functions are clearly separated and documented. Verification passes cleanly with 26 proofs, no shortcuts, and well-structured spec/proof/exec separation. The only substantive observation is the cosmetic debug format difference due to type abstraction, which does not affect correctness. Grade A (not A+ due to the `try_new` dead-code-path design choice and minor debug format discrepancy).
