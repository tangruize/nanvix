# Exec Consistency Fix: bitmap

## Summary
- Mismatches fixed: 2
- Missing functions added: 0
- Documented equivalences: 8

## Changes
| Function | Action | Justification |
|----------|--------|---------------|
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | FIXED | Restored zeroing loop (`while i < array.len() { array.set(i, 0u8); }`) matching original's `for byte in array.iter_mut() { *byte = 0; }`. Original explicitly zeroes the array after allocation; verus had removed this relying on `RawArray::new` zero-initialization. The `while` loop replaces `for` (Verus limitation: no iterator support). `is_multiple_of()` → `%` is equivalent. |
| `from_raw_array` [from_raw_array.diff](from_raw_array.diff) [from_raw_array_source.rs](from_raw_array_source.rs) [from_raw_array_verus.rs](from_raw_array_verus.rs) | FIXED | Restored zeroing loop and `mut` parameter. Original takes `mut array` and zeroes all bytes; verus had removed the loop and required a precondition that the array was already zeroed (`forall\|i\| array@[i] == 0`). This precondition has been removed—the function now zeroes the array itself, matching the original contract. |
| `number_of_bits` [number_of_bits.diff](number_of_bits.diff) [number_of_bits_source.rs](number_of_bits_source.rs) [number_of_bits_verus.rs](number_of_bits_verus.rs) | EQUIVALENT | Identical exec logic: returns `self.number_of_bits`. Only difference is added requires/ensures annotations. |
| `alloc` [alloc.diff](alloc.diff) [alloc_source.rs](alloc_source.rs) [alloc_verus.rs](alloc_verus.rs) | EQUIVALENT | Identical exec logic: calls `self.alloc_range(1)`. Only difference is proof block before the call. |
| `alloc_range` [alloc_range.diff](alloc_range.diff) [alloc_range_source.rs](alloc_range_source.rs) [alloc_range_verus.rs](alloc_range_verus.rs) | EQUIVALENT | Exec logic preserved. Structural differences: (1) `for` → `while` (Verus has no `for`/iterator support), (2) `self.bits[w] \|= x` → `self.bits.set(w, self.bits[w] \| x)` (Verus has no `\|=` on array index), (3) `self.usage += size` → incremental `self.usage = self.usage + 1` per iteration (needed for loop invariant tracking), (4) added unreachable bounds check `idx >= self.number_of_bits` (needed for Verus to prove `test_unchecked` precondition; unreachable since outer loop guarantees `start + offset < number_of_bits`), (5) `start += offset + 1` → `start = idx + 1` where `idx = start + offset` (algebraically identical). |
| `set` [set.diff](set.diff) [set_source.rs](set_source.rs) [set_verus.rs](set_verus.rs) | EQUIVALENT | Identical exec logic. `self.bits[word] \|= 1 << bit` → `self.bits.set(word, self.bits[word] \| (1 << bit))` (Verus limitation: no compound assignment on array index). `self.usage += 1` → `self.usage = self.usage + 1` (Verus syntax). |
| `clear` [clear.diff](clear.diff) [clear_source.rs](clear_source.rs) [clear_verus.rs](clear_verus.rs) | EQUIVALENT | Same pattern as `set`. `self.bits[word] &= !(1 << bit)` → `self.bits.set(word, self.bits[word] & !(1 << bit))`. `self.usage -= 1` → `self.usage = self.usage - 1`. |
| `test` [test.diff](test.diff) [test_source.rs](test_source.rs) [test_verus.rs](test_verus.rs) | EQUIVALENT | Identical exec logic. Intermediate values bound to named variables (`byte_val`, `result_val`) for verification clarity; semantically equivalent to original's single expression. |
| `index` [index.diff](index.diff) [index_source.rs](index_source.rs) [index_verus.rs](index_verus.rs) | EQUIVALENT | Identical exec logic. Parameter renamed from `index` to `bit_index` to avoid shadowing with the method name; no semantic change. |
| `index_unchecked` [index_unchecked.diff](index_unchecked.diff) [index_unchecked_source.rs](index_unchecked_source.rs) [index_unchecked_verus.rs](index_unchecked_verus.rs) | EQUIVALENT | Identical exec logic. Parameter renamed from `index` to `bit_index` (same as `index`). |

## Extra Functions in Verus (EXTRA_IN_VERUS)
| Function | Action | Justification |
|----------|--------|---------------|
| `new_managed` [new_managed_verus.rs](new_managed_verus.rs) | KEPT | Thin alias for `new()` with stronger preconditions. Used by slab allocator API. No exec logic beyond `Self::new(number_of_bits)`. |
| `usage` [usage_verus.rs](usage_verus.rs) | KEPT | Simple getter returning `self.usage`. Needed by verification callers to query bitmap usage count. |
| `clear_range` [clear_range_verus.rs](clear_range_verus.rs) | KEPT | Extension for clearing a contiguous range of bits. Used by slab/frame allocators for bulk deallocation. Exec logic follows same pattern as `set`/`clear`. |
| `test_unchecked` [test_unchecked_verus.rs](test_unchecked_verus.rs) | KEPT | Helper extracted from `alloc_range` for verification. Combines `index_unchecked` + bit test. Needed because Verus requires explicit precondition checking. |
| `test_*_verified` (10 functions) | KEPT | Verification test functions that prove properties of the bitmap. These are proof-level tests, not executable tests. |

## Verification: PASS
- Module `libs::bitmap`: 85 verified, 0 errors
- Module `libs::slab` (caller): 83 verified, 0 errors
- Module `kernel::mm::phys::frame` (caller): 26 verified, 0 errors
