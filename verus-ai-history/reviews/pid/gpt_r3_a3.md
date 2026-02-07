# Review: pid (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `ProcessIdentifier` definition and constants (exec: `verus/split/kernel/pm/sys/pid.rs`).
  **Description:** The field is still `pub` and `PARSE_ERROR_MESSAGE` remains `pub`. This is still an API/encapsulation mismatch with the original (`pub struct ProcessIdentifier(i32);` and a private `const PARSE_ERROR_MESSAGE`). The explanatory comment does not restore the original visibility. The issue is not fixed.
  **Evidence:** `pub struct ProcessIdentifier { pub value: i32 }` and `pub const PARSE_ERROR_MESSAGE` in `verus/split/kernel/pm/sys/pid.rs`.
  **Suggested Fix:** Make the field private or `pub(crate)` and make the error string private, matching the original API. Use `spec_value()`/`view()` to support verification without widening the runtime API.

### Low
- **Location:** External trait implementations (exec: `verus/split/kernel/pm/sys/pid.rs`, outside `verus!`).
  **Description:** Trait impls remain unverified Rust code. The comment says they are “marked external,” but there are no `#[verifier::external_body]` annotations or proof wrappers. This remains a trusted boundary that is not explicitly captured in the proof artifacts.
  **Evidence:** Trait impls from `impl Default` onward have no verifier annotations.
  **Suggested Fix:** Add explicit `#[verifier::external_body]` or proof wrappers that document the trusted behavior.

## Positive Observations
- The spec/proof structure and axioms remain clean and unchanged.
- Core conversion functions still carry precise ensures clauses.

## Summary
The previous issues were not actually fixed: the public field and public error constant remain, and trait impls are still unverified. No new regressions were introduced, but verification is still incomplete relative to the original interface and trust boundary documentation.
