# Review: tid (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `ThreadIdentifier` struct field visibility (exec, `verus/split/kernel/pm/sys/tid.rs`).
  **Description:** The field remains `pub value: i32`; the previous review requested making it private or `pub(crate)` to preserve the original API and prevent bypassing the verified constructors/conversions. The new comment explains the choice but does not fix the API exposure, so semantic equivalence and encapsulation are still weakened.
  **Suggested Fix:** Make `value` private (or `pub(crate)` if needed internally) and rely on `spec_value()`/`view()` and accessor methods for reasoning.

### Low
- **Location:** Byte serialization/layout axioms (proof/exec, `tid.proof.rs`, `tid.rs`).
  **Description:** The module still relies on `#[verifier::external_body]` axioms for byte round-trips and layout. The update adds an additional axiom (`axiom_from_ne_bytes_in_range`), which expands the trusted base rather than eliminating it. Soundness still depends on these assumptions.
  **Suggested Fix:** Isolate these axioms in a designated trusted module or replace with a proven model if available.

## Positive Observations
- Trust-boundary documentation is clearer, and the composed round-trip lemma improves downstream usability.
- Conversion specs and error semantics remain consistent with the original behavior.

## Summary
The previous medium issue (public field exposure) is not fixed, and the proof remains dependent on external-body axioms (now with an additional assumption). Verification appears complete only modulo those trust boundaries, with no other new regressions found.
