# Review: tid (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `ThreadIdentifier` struct field visibility (exec, `verus/split/kernel/pm/sys/tid.rs`).
  **Description:** The `value` field is still `pub`, despite the prior review requesting private (or `pub(crate)`) visibility to match the original API. The added comment does not prevent external code from bypassing the verified constructors/conversions, so the API exposure remains.
  **Suggested Fix:** Make `value` private (or `pub(crate)` if needed internally) and rely on `spec_value()`/`view()` plus accessor methods for reasoning.

### Low
- **Location:** Byte serialization/layout axioms (proof/exec, `tid.proof.rs`, `tid.rs`).
  **Description:** The module still relies on `#[verifier::external_body]` axioms for byte round-trips and layout, and the added `axiom_from_ne_bytes_in_range` further expands the trusted base. Soundness is still conditional on these assumptions.
  **Suggested Fix:** Isolate these axioms in a designated trusted module or replace with a proven model if available.

## Positive Observations
- No new regressions in conversion specs or error semantics were found.
- The composed round-trip lemma still improves usability for downstream proofs.

## Summary
The prior medium issue (public field exposure) remains unfixed, and the proof remains dependent on external-body axioms. Verification is therefore complete only modulo these trust boundaries, with no additional new issues identified.
