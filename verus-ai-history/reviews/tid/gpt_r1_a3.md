# Review: tid (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Priority:** Medium  
  **Location:** `ThreadIdentifier` struct field (`verus/split/kernel/pm/sys/tid.rs:70-76`)  
  **Description:** The `value` field remains `pub`, so the verified exec type still exposes a writable raw field, unlike the original tuple struct with a private field (`src/libs/sys/src/sys/pm/tid.rs:41`). This is a real API expansion that permits direct construction/mutation outside the intended conversion path.  
  **Suggested Fix:** Make the field private and use `view()`/`spec_value` for spec access. If special access is required in proofs, introduce `pub(crate)` constructors or helper functions within the module instead of a public field.

### Low
- None.

## Positive Observations
- The previously noted extra inherent methods are now `pub(crate)` (e.g., `from_i32`, `into_i32`, `try_from_*`, `eq/ne/lt/le/gt/ge/cmp_ord`), so the public API surface is now much closer to the original.
- No new `Default` impls or new `external_body` blocks were introduced.
- Proof/spec files remain consistent and contain no `unimplemented!`, `assume`, or `admit` placeholders.

## Summary
The prover fixed one of the two prior issues by restricting the extra inherent methods to `pub(crate)`, but the public `value` field is still exposed and keeps the verified API from matching the original. No new soundness issues were introduced, and the proof infrastructure remains intact. Verification is close, but the remaining API mismatch prevents a pass.
