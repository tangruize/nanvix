# Review: bitmap Exec Consistency (claude-opus-4.6)

## Grade: A

## Verification Result

- **Command**: `verus --crate-type lib lib.rs --verify-module libs::bitmap`
- **Result**: 83 verified, 0 errors
- **Cheating**: None (`assume()`: 0, `external_body`: 0; one `#[verifier::external]` on test-only `Deref` impl — acceptable)

## Issues Found

### Critical

- None.

### Minor

1. **`from_raw_array` precondition is stricter than source**: The source's `from_raw_array` has no preconditions — it accepts any `RawArray<u8>` and returns `Result<Self, Error>`, using `checked_mul` for overflow detection. The Verus version adds `requires array@.len() > 0` and `array@.len() * u8::BITS < u32::MAX` and `forall|i: int| 0 <= i < array@.len() ==> array@[i] == 0`. These preconditions are needed to establish `inv()` in the postcondition and are satisfied by all callers (since `RawArray` guarantees zero-initialization), but they shift error handling from runtime to compile-time verification. The `ensures result is Ok` clause means the error path is provably unreachable — this is sound but diverges from source semantics where the error path is reachable at the type level. **Impact**: Low. Callers that don't satisfy preconditions will get verification errors instead of runtime errors; the exec behavior for valid callers is identical.

2. **`new` uses `is_multiple_of` correctly**: The old Verus code used `number_of_bits % (u8::BITS as usize) != 0`; the fix restored `is_multiple_of` matching the source. Verified correct.

3. **Verify script module resolution**: Running `./verus-ai/scripts/verify.sh bitmap` fails because the script cannot resolve `bitmap` to `libs::bitmap` when the entry file is `lib.rs` (not `bitmap.rs` or `mod.rs`). Must use `./verus-ai/scripts/verify.sh libs::bitmap`. This is a tooling issue, not a code issue.

### Informational

1. **`new_managed`**: Extra function not in source. Alias for `new()` with tighter preconditions for slab/frame callers. No exec divergence since it delegates entirely to `new()`. Justified.

2. **`usage()`**: Extra getter not in source. Source accesses `self.usage` directly in tests via `Deref`; the Verus version exposes a getter for verification postconditions. Justified.

3. **`test_unchecked`**: Extra unchecked test helper for proof use. Same bit-test logic as `test()` without bounds check. Justified.

4. **`clear_range`**: Extra function not in source. Complements `alloc_range` for range deallocation. Has full verification (loop invariants, frame conditions, set-based postconditions). Justified for slab/frame integration.

5. **12 `test_*_verified` functions**: Verification-only tests proving key bitmap properties. Not exec code — serve as proof-carrying tests. Justified.

## Function-by-Function Consistency Analysis

| Function | Source Signature | Verus Signature | Exec Match | Notes |
|----------|-----------------|-----------------|------------|-------|
| `new` | `fn new(usize) -> Result<Self, Error>` | Same | ✅ | `Ok(Self{...})` → `let result = Self{...}; Ok(result)` for proof block. |
| `from_raw_array` | `fn from_raw_array(RawArray<u8>) -> Result<Self, Error>` | Same return type (restored from old `-> Self`) | ✅ | `checked_mul`/`ok_or_else` → manual `if array.len() > usize::MAX / 8`. Equivalent overflow detection. |
| `number_of_bits` | `fn number_of_bits(&self) -> usize` | Same | ✅ | Identical. |
| `alloc` | `fn alloc(&mut self) -> Result<usize, Error>` | Same | ✅ | Delegates to `alloc_range(1)`. |
| `alloc_range` | `fn alloc_range(&mut self, usize) -> Result<usize, Error>` | Same | ✅ | `for` → `while`; `bits[w] \|= ...` → `bits.set(w, ...)`;`self.usage += size` → `self.usage = self.usage + size`; `debug_assert_eq!` guarded with `#[cfg(not(verus_keep_ghost))]`. All equivalent. |
| `set` | `fn set(&mut self, usize) -> Result<(), Error>` | Same | ✅ | `bits[word] \|= ...` → `bits.set(word, ...)`; `self.usage += 1` → `self.usage = self.usage + 1`. |
| `clear` | `fn clear(&mut self, usize) -> Result<(), Error>` | Same | ✅ | `bits[word] &= !(...)`→ `bits.set(word, ... & !(...))`;`self.usage -= 1` → `self.usage = self.usage - 1`. |
| `test` | `fn test(&self, usize) -> Result<bool, Error>` | Same | ✅ | Intermediate variables `byte_val`, `result_val` extracted; same computation. |
| `index` | `fn index(&self, usize) -> Result<(usize, usize), Error>` | Param renamed `index` → `bit_index` | ✅ | Rename only; same logic. |
| `index_unchecked` | `fn index_unchecked(&self, usize) -> (usize, usize)` | Param renamed `index` → `bit_index` | ✅ | Rename only; same logic. |

## Verus Limitations Documented

All 6 documented Verus limitations are valid and well-known:
1. `for` loops with ranges → `while` ✅
2. Mutable indexing with compound assignment → `bits.set()` ✅
3. Compound assignment on struct fields → explicit assignment ✅
4. `debug_assert_eq!` → `#[cfg(not(verus_keep_ghost))]` guard ✅
5. `checked_mul` / closures → manual overflow check ✅
6. Direct `Ok(Self{...})` return → variable binding for proof blocks ✅

## Summary

The exec consistency fix correctly restores `from_raw_array` to return `Result<Self, Error>` matching the source signature, with an equivalent manual overflow check replacing the unsupported `checked_mul`/`ok_or_else` pattern. All other documented equivalences are sound — each syntactic transformation (`for` → `while`, `|=` → `.set()`, `+=` → explicit assignment, parameter renames) preserves the original semantics exactly. The extra functions (`new_managed`, `usage`, `test_unchecked`, `clear_range`) are well-justified for verification and downstream integration. Verification passes cleanly with 83 verified, 0 errors, and zero cheating patterns. The only notable divergence is that `from_raw_array`'s preconditions make the error path provably unreachable, which is stricter than source but sound for all known callers.
