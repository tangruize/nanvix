# Review: kredzone Exec Consistency (claude-opus-4.6)

## Grade: A

## Issues Found

### Critical
- None.

### Minor

1. **Axiom soundness caveat (pre-existing, not introduced by this fix):** `axiom_volatile_read_consistency` (proof.rs:254–266) is `external_body` and asserts that any `actual_value` returned by volatile read equals `spec_load_result(ghost_view, index)`. This axiom is sound *only* under trust assumptions T2 (volatile semantics) and T5 (ghost uniqueness). Both are well-documented, but the axiom is universally quantified over `actual_value`, meaning a caller could pass an arbitrary value and have it "proved" equal to the ghost state. This is an inherent limitation of the approach, not a defect in the consistency fix, and is already flagged in the module documentation. No action required.

2. **`init_kredzone` postcondition is trivially `true`** (kredzone.rs:513): The postcondition `ensures true` provides no verified guarantee that all entries are zeroed. The comment "Effect: all kredzone entries are set to 0" is informal. This is acceptable because `store` is `external_body` for the volatile write, so Verus cannot prove the memory effect. Still, a comment explaining why a stronger postcondition is impossible would be helpful.

3. **Error message string difference (cosmetic):** The original uses a `let reason: &str = "index out of bounds"` binding before passing to both `error!()` and `Error::new()`. The Verus version inlines the string literal directly in `Error::new()`. Functionally identical since `error!()` is omitted. No impact.

## Criterion Assessment

### 1. Were all MISMATCH functions properly restored or equivalence documented?
**Yes.** The consistency report identifies `store` and `load` as having documented equivalences rather than mismatches. Both functions have the same control flow structure: bounds check → error return or memory operation → Ok. The three divergence patterns (NUM_ENTRIES vs `size_of`, logging omission, unsafe extraction) are all well-documented with sound justifications.

### 2. Were MISSING functions added with proper verification?
**N/A.** The consistency report states 0 missing functions were added. The original module has exactly two functions (`store`, `load`), both present in the Verus version. The additional functions (`raw_store`, `raw_load`, `store_with_ghost`, `load_with_ghost`, `init_kredzone`, `create_initial_ghost`) are justified extras for verification infrastructure.

### 3. Are equivalence justifications sound?
**Yes.** All three divergence patterns are well-justified:
- **NUM_ENTRIES equivalence:** `KREDZONE_SIZE / ENTRY_SIZE` where `ENTRY_SIZE` is conditionally compiled to match `size_of::<usize>()`. This is verified by `lemma_entry_size_matches_target()` which proves `ENTRY_SIZE == 8` on 64-bit and `ENTRY_SIZE == 4` on 32-bit. The equivalence is machine-checked.
- **Logging omission:** `error!()` is a side-effect-only macro that does not affect control flow or return values. Omission is standard practice for formal verification.
- **Unsafe extraction:** Moving `ptr.write_volatile(value)` / `ptr.read_volatile()` into separate `external_body` functions (`raw_store`/`raw_load`) is a standard Verus pattern. The bounds check remains verified; only the volatile operation is trusted. The exec behavior is identical.

### 4. Does the exec code faithfully represent the original source?
**Yes.** The two original functions (`store`, `load`) are faithfully represented:
- Same function signatures (parameter types, return types).
- Same bounds check condition (`index >= NUM_ENTRIES` ≡ `index >= KREDZONE_SIZE / mem::size_of::<usize>()`).
- Same error code (`ErrorCode::InvalidArgument`) and error message (`"index out of bounds"`).
- Same success path (volatile write/read then Ok).
- The `extern "C" { static mut kredzone: usize; }` declaration is handled within the `raw_store`/`raw_load` stubs, which document the actual implementation.

### 5. Does verification still pass?
**Yes.** Confirmed: 36 verified, 0 errors via `./verus-ai/scripts/verify.sh kredzone`.

## Summary

The exec consistency fixes for kredzone are well-executed. The original module is small (two public functions, one constant), and the Verus version faithfully reproduces both functions with only three categories of intentional divergence, all of which are sound and well-documented. The additional verification infrastructure (ghost state wrappers, algebraic lemmas, `external_body` axiom) is cleanly separated from the core exec code and does not alter the original API. The `lemma_entry_size_matches_target` proof provides machine-checked evidence for the `NUM_ENTRIES` equivalence. The only notable concern is the `axiom_volatile_read_consistency` being a pre-existing `external_body` axiom, but this is inherent to verifying volatile memory operations in Verus and is thoroughly documented with trust assumptions T1–T5.
