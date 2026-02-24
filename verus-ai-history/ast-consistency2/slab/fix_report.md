# Exec Consistency Fix: slab

## Summary
- Mismatches fixed: 0 (all 3 are documented equivalences)
- Missing functions added: 0
- Documented equivalences: 3 functions + 1 struct

## Struct: `Slab` [struct_Slab.diff](struct_Slab.diff) | [struct_Slab_source.rs](struct_Slab_source.rs) | [struct_Slab_verus.rs](struct_Slab_verus.rs)

| Field | Source | Verus | Justification |
|-------|--------|-------|---------------|
| `data_addr` | `*mut u8` | `usize` | Verus limitation: raw pointers not supported. Semantically equivalent — stores same address value. |
| `base_addr` | absent | `usize` | Added for verification: tracks buffer base for bounds-checking invariants (Issue 6 fix). Not present in original because original uses `unsafe` without formal bounds proof. |
| `total_len` | absent | `usize` | Added for verification: tracks buffer length for bounds-checking invariants (Issue 6 fix). Same justification as `base_addr`. |

## Changes

| Function | Action | Justification |
|----------|--------|---------------|
| `from_raw_parts` [from_raw_parts.diff](from_raw_parts.diff) | [from_raw_parts_source.rs](from_raw_parts_source.rs) | [from_raw_parts_verus.rs](from_raw_parts_verus.rs) | DOCUMENTED_EQUIVALENCE | All exec changes are Verus-necessary adaptations (see details below). |
| `allocate` [allocate.diff](allocate.diff) | [allocate_source.rs](allocate_source.rs) | [allocate_verus.rs](allocate_verus.rs) | DOCUMENTED_EQUIVALENCE | Return type `*mut u8` → `usize` (Verus limitation: no raw pointers). Pointer arithmetic `data_addr.add(...)` → integer arithmetic `data_addr + ...`. `?` operator → explicit `match` (required for proof blocks on error path). Core logic identical: `index.alloc()` → compute offset → return address. |
| `deallocate` [deallocate.diff](deallocate.diff) | [deallocate_source.rs](deallocate_source.rs) | [deallocate_verus.rs](deallocate_verus.rs) | DOCUMENTED_EQUIVALENCE | Parameter type `*const u8` → `usize` (Verus limitation: no raw pointers). `ptr.offset_from_unsigned(self.data_addr)` → `(addr - self.data_addr)` (integer equivalent). `index.clear(index)?` → explicit `match` (required for proof blocks on both Ok/Err paths). Bounds checks preserved with identical semantics. |
| `is_power_of_two` [is_power_of_two_verus.rs](is_power_of_two_verus.rs) | EXTRA_JUSTIFIED | Exec helper extracted for verification. Replaces bitwise check `block_size & (block_size - 1) != 0` with iterative division, which is provably equivalent to `spec_is_power_of_two`. Required because Verus needs a verified exec function that connects to the spec. |
| `from_raw_parts_at_offset` [from_raw_parts_at_offset_verus.rs](from_raw_parts_at_offset_verus.rs) | EXTRA_JUSTIFIED | New helper for creating slabs at offsets within larger memory regions. Used by `Kheap` module. Not present in original slab source but needed by verified kernel heap. |
| `num_data_blocks` [num_data_blocks_verus.rs](num_data_blocks_verus.rs) | EXTRA_JUSTIFIED | Getter method needed for verified callers to access field through ensures postconditions. |
| `block_size` [block_size_verus.rs](block_size_verus.rs) | EXTRA_JUSTIFIED | Getter method needed for verified callers to access field through ensures postconditions. |
| `test_slab_from_raw_parts_verified` [test_slab_from_raw_parts_verified_verus.rs](test_slab_from_raw_parts_verified_verus.rs) | EXTRA_JUSTIFIED | Verification test harness — proof-only, no runtime effect. |
| `test_slab_from_raw_parts_allocate_verified` [test_slab_from_raw_parts_allocate_verified_verus.rs](test_slab_from_raw_parts_allocate_verified_verus.rs) | EXTRA_JUSTIFIED | Verification test harness — proof-only, no runtime effect. |
| `test_slab_creation_verified` [test_slab_creation_verified_verus.rs](test_slab_creation_verified_verus.rs) | EXTRA_JUSTIFIED | Verification test harness — proof-only, no runtime effect. |
| `test_allocate_deallocate_verified` [test_allocate_deallocate_verified_verus.rs](test_allocate_deallocate_verified_verus.rs) | EXTRA_JUSTIFIED | Verification test harness — proof-only, no runtime effect. |
| `test_double_deallocate_verified` [test_double_deallocate_verified_verus.rs](test_double_deallocate_verified_verus.rs) | EXTRA_JUSTIFIED | Verification test harness — proof-only, no runtime effect. |
| `test_allocate_out_of_bounds_verified` [test_allocate_out_of_bounds_verified_verus.rs](test_allocate_out_of_bounds_verified_verus.rs) | EXTRA_JUSTIFIED | Verification test harness — proof-only, no runtime effect. |
| `test_multiple_allocations_verified` [test_multiple_allocations_verified_verus.rs](test_multiple_allocations_verified_verus.rs) | EXTRA_JUSTIFIED | Verification test harness — proof-only, no runtime effect. |
| `test_address_computation_verified` [test_address_computation_verified_verus.rs](test_address_computation_verified_verus.rs) | EXTRA_JUSTIFIED | Verification test harness — proof-only, no runtime effect. |
| `test_allocation_reuse_verified` [test_allocation_reuse_verified_verus.rs](test_allocation_reuse_verified_verus.rs) | EXTRA_JUSTIFIED | Verification test harness — proof-only, no runtime effect. |
| `test_memory_block_alignment_verified` [test_memory_block_alignment_verified_verus.rs](test_memory_block_alignment_verified_verus.rs) | EXTRA_JUSTIFIED | Verification test harness — proof-only, no runtime effect. |
| `test_no_data_corruption_verified` [test_no_data_corruption_verified_verus.rs](test_no_data_corruption_verified_verus.rs) | EXTRA_JUSTIFIED | Verification test harness — proof-only, no runtime effect. |
| `test_fresh_slab_all_free_verified` [test_fresh_slab_all_free_verified_verus.rs](test_fresh_slab_all_free_verified_verus.rs) | EXTRA_JUSTIFIED | Verification test harness — proof-only, no runtime effect. |
| `test_index_blocks_always_used_verified` [test_index_blocks_always_used_verified_verus.rs](test_index_blocks_always_used_verified_verus.rs) | EXTRA_JUSTIFIED | Verification test harness — proof-only, no runtime effect. |

## Detailed `from_raw_parts` [from_raw_parts.diff](from_raw_parts.diff) | [from_raw_parts_source.rs](from_raw_parts_source.rs) | [from_raw_parts_verus.rs](from_raw_parts_verus.rs) Equivalence

The verus version makes the following exec-level changes, all necessary for verification:

1. **Parameter type**: `addr: *mut u8` → `addr: usize` — Verus limitation (no raw pointers).
2. **Wrapping check removed**: `addr.wrapping_add(len) < addr` replaced by precondition `(addr as int) + (len as int) <= (usize::MAX as int)` — equivalent: both prevent address space wraparound.
3. **Power-of-two check**: `block_size & (block_size - 1) != 0` → `!Self::is_power_of_two(block_size)` — semantically equivalent for non-zero `block_size` [block_size_verus.rs](block_size_verus.rs) (already validated).
4. **Alignment check**: `!(addr as usize).is_multiple_of(block_size)` → `addr % block_size != 0` — identical semantics; `is_multiple_of` is not available in Verus.
5. **Extra defensive checks added**: `total_num_blocks < 8`, `num_index_blocks == 0`, `num_index_blocks >= total_num_blocks`, `num_data_blocks == 0`, overflow checks with `checked_mul` — these reject additional edge cases that the original would handle via undefined behavior or later panics. They do not change behavior for inputs that the original accepts.
6. **`RawArray::from_raw_parts` → `RawArray::from_raw_addr`**: Verus-verified API variant that zeroes memory and provides postconditions.
7. **`for i in 0..n` → `while i < n`**: Verus limitation (for-range loops not supported).
8. **Pointer arithmetic → integer arithmetic**: `addr.add(num_index_blocks * block_size)` → `addr + index_region_size` — equivalent with `usize` types.
9. **Extra struct fields**: `base_addr: addr, total_len: len` — verification-only metadata for buffer bounds invariant.

**Semantic equivalence**: For any input `(addr, len, block_size)` accepted by the original function (all preconditions met, no UB), the verus version produces a `Slab` [struct_Slab.diff](struct_Slab.diff) | [struct_Slab_source.rs](struct_Slab_source.rs) | [struct_Slab_verus.rs](struct_Slab_verus.rs) with identical `index`, `num_index_blocks`, `num_data_blocks` [num_data_blocks_verus.rs](num_data_blocks_verus.rs), `block_size` [block_size_verus.rs](block_size_verus.rs), and `data_addr` (as integer value). The extra fields `base_addr` and `total_len` are purely additive and do not affect allocator behavior.

## Detailed `allocate` [allocate.diff](allocate.diff) | [allocate_source.rs](allocate_source.rs) | [allocate_verus.rs](allocate_verus.rs) Equivalence

| Aspect | Source | Verus | Equivalent? |
|--------|--------|-------|-------------|
| Return type | `Result<*mut u8, Error>` | `Result<usize, Error>` | Yes (Verus limitation) |
| Alloc call | `self.index.alloc()?` | `match self.index.alloc() { Ok(b) => b, Err(e) => { ... return Err(e) } }` | Yes (explicit match for proof blocks) |
| Address computation | `self.data_addr.add((block - self.num_index_blocks) * self.block_size)` | `self.data_addr + (block - self.num_index_blocks) * self.block_size` | Yes (pointer vs integer arithmetic on same values) |

## Detailed `deallocate` [deallocate.diff](deallocate.diff) | [deallocate_source.rs](deallocate_source.rs) | [deallocate_verus.rs](deallocate_verus.rs) Equivalence

| Aspect | Source | Verus | Equivalent? |
|--------|--------|-------|-------------|
| Parameter | `ptr: *const u8` | `addr: usize` | Yes (Verus limitation) |
| Lower bound check | `ptr < self.data_addr` | `addr < self.data_addr` | Yes |
| Upper bound check | `ptr >= self.data_addr.add(self.num_data_blocks * self.block_size)` | `addr >= self.data_addr + self.num_data_blocks * self.block_size` | Yes |
| Index computation | `self.num_index_blocks + ptr.offset_from_unsigned(self.data_addr) / self.block_size` | `self.num_index_blocks + (addr - self.data_addr) / self.block_size` | Yes |
| Free check | `!self.index.test(index)?` | `!self.index.test(index)?` | Yes |
| Clear call | `self.index.clear(index)?` | `match self.index.clear(index) { Ok(()) => { ... Ok(()) }, Err(e) => { ... Err(e) } }` | Yes (explicit match for proof blocks) |

## Verification: PASS

```
verification results:: 83 verified, 0 errors
```
