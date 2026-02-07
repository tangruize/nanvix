# Review: tid (gpt-5.2-codex)

## Grade: B

## Issues Found

### Medium
- **Location:** `verus/split/kernel/pm/sys/tid.rs` (exec), trait impls around lines 568-705.
  **Description:** The public trait implementations (`From`, `TryFrom`, `PartialEq`, `Ord`, `Debug`) still live outside the `verus!` block and are not verified. There is still no `#[verifier::external_body]` marking on these impls, so the verification boundary remains implicit.
  **Suggested Fix:** Move these trait impls into a `verus!` block with verified wrappers if supported, or explicitly mark them `#[verifier::external_body]` and document them as trusted boundaries.

- **Location:** `verus/split/kernel/pm/sys/tid.spec.rs` lines 58-72; `verus/split/kernel/pm/sys/tid.proof.rs` axioms around lines 73-129.
  **Description:** The byte-serialization spec remains uninterpreted and only constrained by round-trip axioms. The added range axiom does not define actual native-endian layout. ABI/serialization compatibility beyond round-trip fidelity is still unproven.
  **Suggested Fix:** Strengthen the spec to relate bytes to the integer value (bitvector or explicit byte layout axiom), and use that in proofs where byte layout matters.

### Low
- None. The module-level `assert_tid_layout` proof remains in place and invoked.

## Positive Observations
- Layout assertion coverage is still present via `assert_tid_layout`.
- Byte serialization trust boundaries are explicitly documented.

## Summary
The update does not resolve the two medium issues from the previous review: unverified public trait impls and a weak byte-serialization specification. Verification remains incomplete at the public API boundary.
