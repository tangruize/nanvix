# Review: process_capability (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- **Location:** `Capabilities` struct (exec) in `verus/split/kernel/pm/process/capability.rs`.
  **Description:** The verified exec type exposes `pub bits: u8`, while the original uses a private tuple field. This changes the public API and allows external code to construct or mutate non-`wf()` values, which is impossible in the original module. This weakens the equivalence story and undermines the intended invariant that only the `set`/`clear` API can change state.
  **Suggested Fix:** Restore encapsulation by avoiding a public field in the executable model (e.g., keep `bits` private and use `closed` spec functions or a wrapper type for `pub open spec fn` access), or add `requires self.wf()` preconditions to public methods and enforce construction via `new/default` only.

### Medium
- **Location:** `to_mask`/`spec_mask` equivalence (exec/spec) in `capability.rs` and `capability.spec.rs`.
  **Description:** The executable `to_mask` only ensures `result == spec_mask(capability)`. The equivalence to the original shift-based formula is relegated to a standalone lemma (`lemma_mask_matches_discriminant`) and not exposed in the method contract, so callers do not get this equivalence without an extra proof step. This makes the semantic equivalence to the original less explicit in the core API specification.
  **Suggested Fix:** Strengthen `to_mask`’s postcondition to also ensure `result == spec_pow2_mask(capability.spec_discriminant())` (or an equivalent shift-based spec), and reference the lemma in the proof block.

### Low
- None.

## Positive Observations
- All original functions (`set`, `clear`, `has`, plus `Default`) are covered with verified versions, and the verification run passes.
- Specs precisely capture bitwise behavior, and proofs establish idempotence, round-trip behavior, and preservation of other bits.
- No `assume` or `external_body` is used; the module is fully verified.
- The `wf()` invariant is clearly defined and supported by preservation lemmas.

## Summary
The verification is thorough and proves the core bitfield semantics, but the executable model exposes a public field that breaks equivalence with the original module and weakens the invariant story. Tightening encapsulation or adding explicit `wf` preconditions would better match the original design. Overall, the proof structure and split quality are strong, with only a few specification-strength gaps.
