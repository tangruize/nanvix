# Review: pid (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `ProcessIdentifier` definition and constants (exec: `verus/split/kernel/pm/sys/pid.rs`).
  **Description:** The field is still `pub` and `PARSE_ERROR_MESSAGE` is still `pub`. This remains an API/encapsulation mismatch with the original (`pub struct ProcessIdentifier(i32);` and a private `const PARSE_ERROR_MESSAGE`). The comment explains the choice but does not restore the original visibility, so the issue is not fixed. This keeps the verified API strictly more permissive than the original.
  **Evidence:** `pub struct ProcessIdentifier { pub value: i32 }` and `pub const PARSE_ERROR_MESSAGE` in `verus/split/kernel/pm/sys/pid.rs`.
  **Suggested Fix:** Make the field private or at least `pub(crate)` and make the error string private, matching the original API. If Verus needs visibility, provide a `spec_value()` accessor and keep the runtime visibility minimal.

### Low
- **Location:** External trait implementations (exec: `verus/split/kernel/pm/sys/pid.rs`, outside `verus!`).
  **Description:** The trait impls are still unverified Rust code. The comment claims they are “marked external,” but there are no `#[verifier::external_body]` annotations or proof wrappers. This is still a trusted boundary that is not explicitly acknowledged in the proof artifacts.
  **Evidence:** Trait impls from `impl Default` onward have no verifier annotations.
  **Suggested Fix:** Add explicit `#[verifier::external_body]` where appropriate or add proof wrappers documenting these trait impls as trusted.

## Positive Observations
- The proof/spec split remains clean, with explicit axioms for byte serialization and layout.
- Spec-level predicates and ensures clauses still precisely capture conversion behavior and error codes.

## Summary
The previous issues were not actually fixed: the public field and public error constant remain, and trait impls are still unverified despite a clarifying comment. No new correctness regressions were introduced in the core logic, but the API-compatibility and verification-coverage gaps persist. Verification is therefore not fully complete or sound with respect to the original interface.
