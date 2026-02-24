# Exec Consistency Fix: bitmap

## Summary
- Mismatches fixed: 2
- Missing functions added: 0
- Documented equivalences: 8

## Changes
| Function | Action | Justification |
|----------|--------|---------------|
| `new` | FIXED [diff](new.diff) [src](new_source.rs) [verus](new_verus.rs) | Restored zeroing loop (`while i < array.len() { array.set(i, 0u8); }`) matching original's `for byte in array.iter_mut() { *byte = 0; }`. Original explicitly zeroes the array after allocation; verus had removed this relying on `RawArray::new` zero-initialization. The `while` loop replaces `for` (Verus limitation: no iterator support). `is_multiple_of()` → `%` is equivalent. |
| `from_raw_array` | FIXED [diff](from_raw_array.diff) [src](from_raw_array_source.rs) [verus](from_raw_array_verus.rs) | Restored zeroing loop and `mut` parameter. Original takes `mut array` and zeroes all bytes; verus had removed the loop and required a precondition that the array was already zeroed (`forall\|i\| array@[i] == 0`). This precondition has been removed—the function now zeroes the array itself, matching the original contract. |
| `number_of_bits` | EQUIVALENT [diff](number_of_bits.diff) [src](number_of_bits_source.rs) [verus](number_of_bits_verus.rs) | Identical exec logic: returns `self.number_of_bits`. Only difference is added requires/ensures annotations. |
| `alloc` | EQUIVALENT [diff](alloc.diff) [src](alloc_source.rs) [verus](alloc_verus.rs) | Identical exec logic: calls `self.alloc_range(1)`. Only difference is proof block before the call. |
| `alloc_range` | EQUIVALENT [diff](alloc_range.diff) [src](alloc_range_source.rs) [verus](alloc_range_verus.rs) | Exec logic preserved. Structural differences: (1) `for` → `while` (Verus has no `for`/iterator support), (2) `self.bits[w] \|= x` → `self.bits.set(w, self.bits[w] \| x)` (Verus has no `\|=` on array index), (3) `self.usage += size` → incremental `self.usage = self.usage + 1` per iteration (needed for loop invariant tracking), (4) added unreachable bounds check `idx >= self.number_of_bits` (needed for Verus to prove `test_unchecked` precondition; unreachable since outer loop guarantees `start + offset < number_of_bits`), (5) `start += offset + 1` → `start = idx + 1` where `idx = start + offset` (algebraically identical). |
| `set` | EQUIVALENT [diff](set.diff) [src](set_source.rs) [verus](set_verus.rs) | Identical exec logic. `self.bits[word] \|= 1 << bit` → `self.bits.set(word, self.bits[word] \| (1 << bit))` (Verus limitation: no compound assignment on array index). `self.usage += 1` → `self.usage = self.usage + 1` (Verus syntax). |
| `clear` | EQUIVALENT [diff](clear.diff) [src](clear_source.rs) [verus](clear_verus.rs) | Same pattern as `set`. `self.bits[word] &= !(1 << bit)` → `self.bits.set(word, self.bits[word] & !(1 << bit))`. `self.usage -= 1` → `self.usage = self.usage - 1`. |
| `test` | EQUIVALENT [diff](test.diff) [src](test_source.rs) [verus](test_verus.rs) | Identical exec logic. Intermediate values bound to named variables (`byte_val`, `result_val`) for verification clarity; semantically equivalent to original's single expression. |
| `index` | EQUIVALENT [diff](index.diff) [src](index_source.rs) [verus](index_verus.rs) | Identical exec logic. Parameter renamed from `index` to `bit_index` to avoid shadowing with the method name; no semantic change. |
| `index_unchecked` | EQUIVALENT [diff](index_unchecked.diff) [src](index_unchecked_source.rs) [verus](index_unchecked_verus.rs) | Identical exec logic. Parameter renamed from `index` to `bit_index` (same as `index`). |

## Extra Functions in Verus (EXTRA_IN_VERUS)
| Function | Action | Justification |
|----------|--------|---------------|
| `new_managed` | KEPT [verus](new_managed_verus.rs) | Thin alias for `new()` with stronger preconditions. Used by slab allocator API. No exec logic beyond `Self::new(number_of_bits)`. |
| `usage` | KEPT [verus](usage_verus.rs) | Simple getter returning `self.usage`. Needed by verification callers to query bitmap usage count. |
| `clear_range` | KEPT [verus](clear_range_verus.rs) | Extension for clearing a contiguous range of bits. Used by slab/frame allocators for bulk deallocation. Exec logic follows same pattern as `set`/`clear`. |
| `test_unchecked` | KEPT [verus](test_unchecked_verus.rs) | Helper extracted from `alloc_range` for verification. Combines `index_unchecked` + bit test. Needed because Verus requires explicit precondition checking. |
| `test_*_verified` (10 functions) | KEPT | Verification test functions that prove properties of the bitmap. These are proof-level tests, not executable tests. |

## Verification: PASS
- Module `libs::bitmap`: 85 verified, 0 errors
- Module `libs::slab` (caller): 83 verified, 0 errors
- Module `kernel::mm::phys::frame` (caller): 26 verified, 0 errors
