# Review: slab Exec Consistency (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical
- None.

### Minor
- **Unused accessor functions**: `num_data_blocks()` (line 572) and `block_size()` (line 585) have **zero callers** anywhere in the verified codebase (`verus/split/`). All proof/spec code accesses the fields directly (e.g., `slab.block_size`, `slab.num_data_blocks`). These should be removed per EXTRA function policy, or callers should be added to justify retention.

### Informational
- **Precondition strengthening**: `from_raw_parts` moves several original runtime checks into `requires` clauses (e.g., `len / block_size >= 8`, `addr > 0`, `addr + len <= usize::MAX`). This means those error paths are unreachable in verified code. Acceptable for verification but changes the API contract semantics vs. the original.
- **12 test functions** (lines 1077–1560) are verification-only proof tests. Harmless but add ~500 lines of bulk.
- `deallocate` preconditions (`is_valid_addr`, `can_deallocate`) make the bounds check (line 849) and "already free" check (line 918) unreachable in verified callers. They are retained for source faithfulness, which is fine.

## Criterion-by-Criterion Assessment

### 1. Were MISMATCH/MISSING functions actually FIXED?
**Yes — genuine code changes, not just documentation.**

| Fix | Detail |
|-----|--------|
| `from_raw_parts` comparison | Restored `num_index_blocks > total_num_blocks` (was `>=`). ✓ Matches original line 131. |
| `from_raw_parts` error message | Restored `"insufficient blocks for index"` (was `"too many index blocks"`). ✓ Matches original line 132. |
| `from_raw_parts` extra checks | Removed 6 defensive checks not in source (`total_num_blocks < 8`, `num_index_blocks == 0`, etc.). ✓ |
| `deallocate` bounds check | Combined back to single `if addr < self.data_addr \|\| addr >= ...` (was two separate checks). ✓ Matches original line 206–208. |
| `deallocate` error message | Restored `"pointer out of bounds"` (was split into two messages). ✓ Matches original line 209. |
| `deallocate` alignment check | Removed extra alignment check not in source. ✓ |

No evidence of "documenting equivalences in lieu of fixing." The prover made real structural changes.

### 2. Were EXTRA functions removed?
**Partially.** Five categories of extras remain:

| Function | Callers | Verdict |
|----------|---------|---------|
| `raw_array_from_addr` | `from_raw_parts` (line 331) | ✓ Keep — Verus cannot cast `usize → *mut T`. |
| `is_power_of_two` | `from_raw_parts` (line 236) | ✓ Keep — Verus cannot reason about bitwise `&`. |
| `from_raw_parts_at_offset` | `kheap.rs` (lines 444–451, 8 calls) | ✓ Keep — used by verified caller. |
| `num_data_blocks()` | **Zero callers** | ✗ Remove or justify. |
| `block_size()` | **Zero callers** | ✗ Remove or justify. |
| 12 test functions | N/A (proof tests) | ✓ Keep — verification-only, no exec impact. |

### 3. Are remaining justifications genuinely necessary?
**Yes — all are legitimate Verus limitations.**

| Justification | Legitimate? |
|---------------|-------------|
| `*mut u8` → `usize` (no raw pointers) | ✓ Fundamental Verus limitation. |
| `for` → `while` (no `for` loops) | ✓ Fundamental Verus limitation. |
| Bitwise `&` → `is_power_of_two()` | ✓ Verus cannot reason about bitwise ops in proofs. |
| `is_multiple_of()` → `%` | ✓ Trivially equivalent; `is_multiple_of` not in Verus stdlib. |
| `?` → explicit `match` | ✓ Required for proof blocks on error paths. |
| `RawArray::from_raw_parts` → `raw_array_from_addr` | ✓ Needed due to `usize → *mut T` cast limitation. |

None of these are lazy — all reflect real Verus constraints that cannot be worked around.

### 4. Does the exec code faithfully represent the original source?
**Yes, with appropriate adaptations.**

- **`from_raw_parts`**: Same validation sequence (length → block size → power-of-two → alignment → layout → data_addr alignment → instantiate → initialize loop). Control flow matches. Extra `base_addr`/`total_len` fields stored for invariant proofs — does not alter behavior.
- **`allocate`**: Same logic (bitmap alloc → compute address → return). Return type difference (`*mut u8` → `usize`) is documented and unavoidable.
- **`deallocate`**: Same logic (bounds check → compute index → free-check → clear). Combined bounds check matches original single-`if` structure.
- **Struct fields**: `data_addr: *mut u8` → `usize` is documented. Extra `base_addr`/`total_len` are verification scaffolding only.

### 5. Does verification pass?
**Yes.** 87 verified, 0 errors. Confirmed by running `./verus-ai/scripts/verify.sh slab`.

## Summary

Solid exec consistency fix. The prover made **genuine code changes** to restore original semantics — comparison operators, error messages, and extraneous checks were all corrected, not merely documented. All six remaining Verus-limitation justifications are legitimate and well-documented. The exec code faithfully mirrors the original `src/libs/slab/src/lib.rs` with only necessary adaptations for Verus's type system constraints. Two minor unused accessor functions (`num_data_blocks()`, `block_size()`) should be removed or their callers identified. Verification passes cleanly at 87/0.
