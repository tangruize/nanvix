# Exec Consistency Fix: ustack

## Summary
- Mismatches fixed: 1 (new)
- Missing functions added: 1 (fmt)
- Documented equivalences: 3 (size, base, top)

## Changes
| Function | Action | Justification |
|----------|--------|---------------|
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | FIXED — renamed `from_aligned` → `new` | Original `new(PageAligned<VirtualAddress>) -> Self` is infallible. The verus `from_aligned(PageAlignedAddr) -> Self` is the semantic equivalent. Renamed to match the original function name. |
| `try_new` [try_new_verus.rs](try_new_verus.rs) | DOCUMENTED — renamed from `new` | Original `new` is infallible; the fallible `new(usize) -> Result` was a verification helper with different semantics. Renamed to `try_new` to avoid name collision and clarify it is an extra helper. |
| `size` [size.diff](size.diff) [size_source.rs](size_source.rs) [size_verus.rs](size_verus.rs) | DOCUMENTED — equivalent | Exec body is identical (`USER_STACK_SIZE`). AST mismatch is due to Verus named return syntax `(result: usize)` and requires/ensures annotations (ghost code stripped by AST comparison). |
| `base` [base.diff](base.diff) [base_source.rs](base_source.rs) [base_verus.rs](base_verus.rs) | DOCUMENTED — equivalent | Original returns `self.base` (`PageAligned<VirtualAddress>`). Verus returns `PageAlignedAddr::from_raw_unchecked(self.base_addr)`. Equivalent because `PageAlignedAddr` mirrors `PageAligned<VirtualAddress>` and the struct stores the raw address directly. Alignment is proven by the invariant. |
| `top` [top.diff](top.diff) [top_source.rs](top_source.rs) [top_verus.rs](top_verus.rs) | DOCUMENTED — equivalent | Original: `PageAligned::from_raw_value(self.base.into_raw_value() + self.size()).unwrap()`. Verus: `PageAlignedAddr::from_raw_unchecked(self.base_addr + USER_STACK_SIZE)`. (1) `from_raw_unchecked` is equivalent to `from_raw_value().unwrap()` because alignment is machine-checked by Verus, eliminating the need for runtime check + unwrap. (2) `self.base.into_raw_value()` ≡ `self.base_addr` (field type abstraction). (3) `self.size()` ≡ `USER_STACK_SIZE` (constant). |
| `fmt` [fmt.diff](fmt.diff) [fmt_source.rs](fmt_source.rs) [fmt_verus.rs](fmt_verus.rs) | ADDED | Custom `fmt::Debug` implementation added outside `verus!` block to match original kernel format `UserStack { base: ..., top: ..., size=... }`. Replaced `#[derive(Debug)]` which produced a different format. Verus does not support trait impls inside `verus!` blocks, so placed outside. |
| `UserStack` struct | DOCUMENTED — equivalent | Original field: `base: PageAligned<VirtualAddress>`. Verus field: `base_addr: usize`. Necessary because Verus cannot import kernel wrapper types. The `PageAlignedAddr` wrapper and `inv()` postconditions provide equivalent alignment guarantees. |
| `PageAlignedAddr` [struct_PageAlignedAddr_verus.rs](struct_PageAlignedAddr_verus.rs) | DOCUMENTED — justified extra | Mirrors `PageAligned<VirtualAddress>` from the kernel. Required because Verus modules cannot import kernel types directly. Provides type-level alignment guarantee via `inv()` spec. |
| `base_raw` [base_raw_verus.rs](base_raw_verus.rs) | DOCUMENTED — justified extra | Raw accessor returning `usize`. Verification helper used by other methods (`initial_sp`). Not in original API. |
| `top_raw` [top_raw_verus.rs](top_raw_verus.rs) | DOCUMENTED — justified extra | Raw accessor returning `usize`. Verification helper used by other methods (`initial_sp`). Not in original API. |
| `from_raw` [from_raw_verus.rs](from_raw_verus.rs) | DOCUMENTED — justified extra | Constructor for `PageAlignedAddr` from raw `usize` (fallible). Verification helper for the type abstraction layer. |
| `from_raw_unchecked` [from_raw_unchecked_verus.rs](from_raw_unchecked_verus.rs) | DOCUMENTED — justified extra | Constructor for `PageAlignedAddr` from raw `usize` (infallible with precondition). Used by `base()` and `top()` to construct return values with proven alignment. |
| `into_raw` [into_raw_verus.rs](into_raw_verus.rs) | DOCUMENTED — justified extra | Accessor for `PageAlignedAddr` returning raw `usize`. Used by `new()` to extract the address from the typed wrapper. |
| `contains` [contains_verus.rs](contains_verus.rs) | DOCUMENTED — justified extra | Extension method checking address bounds. Demonstrates additional verified property. Clearly marked as extension in module docs. |
| `page_index` [page_index_verus.rs](page_index_verus.rs) | DOCUMENTED — justified extra | Extension method computing page index. Demonstrates additional verified property. Clearly marked as extension in module docs. |
| `initial_sp` [initial_sp_verus.rs](initial_sp_verus.rs) | DOCUMENTED — justified extra | Extension method returning initial stack pointer. Demonstrates additional verified property. Clearly marked as extension in module docs. |
| `has_room` [has_room_verus.rs](has_room_verus.rs) | DOCUMENTED — justified extra | Extension method checking stack growth room. Demonstrates additional verified property. Clearly marked as extension in module docs. |

## Verification: PASS

```
verification results:: 26 verified, 0 errors (partial verification with `--verify-*`)
```

No `assume`, `admit`, or unjustified `external_body` used.
