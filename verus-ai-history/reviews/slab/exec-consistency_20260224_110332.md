# Review: slab Exec Consistency (claude-opus-4.6)

## Grade: A

## Verification Result

```
verification results:: 83 verified, 0 errors
```

All 83 verification conditions pass. Module verified as `libs::slab`.

## Issues Found

### Critical

- None.

### Minor

1. **Redundant defensive check**: `from_raw_parts` at line 261 checks `if block_size == 0` again, but this was already checked at line 207. Harmless but dead code.

2. **`from_raw_parts_at_offset` not in original source**: The report documents this as `EXTRA_JUSTIFIED` for use by `Kheap`. This is acceptable — it delegates to `from_raw_parts` and adds no independent exec logic — but the justification could be stronger by referencing the specific `Kheap` caller site.

3. **`is_power_of_two` iterative vs bitwise**: The original uses `block_size & (block_size - 1) != 0` (constant-time bitwise). The Verus version uses iterative division (O(log n) loop). The report correctly documents this as a Verus limitation. Semantically equivalent for all positive inputs. No functional concern; minor performance difference is irrelevant for verification context.

4. **Verify script module resolution**: `./verus-ai/scripts/verify.sh slab` fails because the script cannot locate `libs/slab/lib.rs` (it searches for `slab.rs`). The direct command `verus --verify-module libs::slab` works. This is an infrastructure issue, not a code issue.

## Criterion 1: Were all MISMATCH functions properly restored or equivalence documented?

**Yes.** The report identifies 3 functions (`from_raw_parts`, `allocate`, `deallocate`) as `DOCUMENTED_EQUIVALENCE` with 0 mismatches. All exec-level changes are Verus-necessary adaptations:

| Function | Key Adaptations | Sound? |
|----------|----------------|--------|
| `from_raw_parts` | `*mut u8` → `usize`, wrapping check → precondition, bitwise → `is_power_of_two()`, `is_multiple_of` → `%`, `for` → `while`, `RawArray::from_raw_parts` → `from_raw_addr`, extra fields `base_addr`/`total_len` | ✅ |
| `allocate` | Return `*mut u8` → `usize`, `?` → explicit `match`, pointer arithmetic → integer arithmetic | ✅ |
| `deallocate` | Param `*const u8` → `usize`, `offset_from_unsigned` → subtraction, `?` → explicit `match` | ✅ |

Each adaptation is forced by Verus limitations (no raw pointers, no `?` with proof blocks, no `for` loops, no `is_multiple_of`).

## Criterion 2: Were MISSING functions added with proper verification?

**N/A.** The report states 0 missing functions. All 3 original functions have Verus counterparts. The extra functions (`is_power_of_two`, `from_raw_parts_at_offset`, `num_data_blocks`, `block_size`, test harnesses) are additive and justified.

## Criterion 3: Are equivalence justifications sound?

**Yes.** The detailed equivalence tables in the report are accurate. I verified each claim:

- **`from_raw_parts`**: All 9 documented changes checked against source lines 86–159 and Verus lines 155–428. The wrapping check replacement (`addr.wrapping_add(len) < addr` → precondition `addr + len <= usize::MAX`) is semantically equivalent — both prevent address-space wraparound. The extra defensive checks (lines 231–256, 261–278) reject edge cases that the original would not encounter given valid inputs, so they are strictly conservative.

- **`allocate`**: Source lines 171–179 vs Verus lines 570–756. Return type change and `match` expansion are mechanically necessary. Address computation `self.data_addr.add((block - self.num_index_blocks) * self.block_size)` → `self.data_addr + (block - self.num_index_blocks) * self.block_size` is exact integer equivalent of pointer arithmetic. Overflow safety is proven in the proof block (lines 685–701).

- **`deallocate`**: Source lines 200–223 vs Verus lines 779–1045. All 5 equivalence rows in the report table verified correct. The bounds checks are semantically identical (`ptr < self.data_addr` → `addr < self.data_addr`, etc.). The `offset_from_unsigned` → subtraction conversion is correct since `addr >= self.data_addr` is guaranteed by the bounds check.

## Criterion 4: Does the exec code faithfully represent the original source?

**Yes.** The Verus exec code preserves all original control flow:
1. Input validation checks in `from_raw_parts` (length, block size, power-of-two, alignment, block count divisibility).
2. Layout computation (total blocks, index length, index blocks, data blocks, data address).
3. Index initialization loop marking index blocks as allocated.
4. `allocate`: bitmap alloc → offset computation → address return.
5. `deallocate`: bounds check → index computation → free check → bitmap clear.

The structural additions (`base_addr`, `total_len` fields; `SlabView` abstraction; proof blocks) are purely for verification and do not alter runtime behavior.

## Criterion 5: Does verification still pass?

**Yes.** `83 verified, 0 errors` confirmed by running `verus --crate-type lib lib.rs --verify-module libs::slab` from the `verus/split/` directory.

## Summary

The slab exec consistency fix is thorough and sound. All three original functions (`from_raw_parts`, `allocate`, `deallocate`) have Verus counterparts that are semantically equivalent modulo well-documented Verus limitations (no raw pointers, no `?` operator in proof contexts, no `for` loops). The equivalence justifications in the report are detailed, accurate, and include both high-level rationale and line-by-line comparison tables. Extra functions are genuinely needed for verification infrastructure. The verification suite passes completely with 83 conditions verified. The only notable gap is the verify script's inability to resolve the `slab` shorthand to `libs::slab`, which is an infrastructure convenience issue. Grade A (not A+ due to the redundant dead-code check and the verify script issue).
