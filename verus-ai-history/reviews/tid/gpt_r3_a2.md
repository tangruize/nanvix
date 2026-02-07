# Review: tid (gpt-5.2-codex)

## Grade: B

## Issues Found

### Medium
- **Location:** `verus/split/kernel/pm/sys/tid.rs` (exec), trait impls around lines 568-705.
  **Description:** The public trait implementations (`From`, `TryFrom`, `PartialEq`, `Ord`, `Debug`) still live outside the `verus!` block and are not verified. The comments claim they are “marked external,” but no `#[verifier::external_body]` is applied and they remain unverified glue. This is the same issue as the previous review and remains unresolved.
  **Suggested Fix:** Move trait implementations inside a `verus!` block with verified wrappers if supported, or explicitly mark them `#[verifier::external_body]` and document them as trusted boundaries.

- **Location:** `verus/split/kernel/pm/sys/tid.spec.rs` lines 58-72; `verus/split/kernel/pm/sys/tid.proof.rs` axioms around lines 73-129.
  **Description:** The byte-serialization spec is still uninterpreted and only constrained by round-trip axioms. The update added `axiom_from_ne_bytes_in_range`, but the spec still does not characterize native-endian layout beyond round-trip fidelity. This remains too weak for ABI/serialization compatibility proofs.
  **Suggested Fix:** Strengthen the spec to relate bytes to the integer value (bitvector or explicit byte layout axiom), and use that in proofs where byte layout matters.

### Low
- None. The module-level `assert_tid_layout` proof (tid.proof.rs lines 268-278) now invokes layout lemmas, which fixes the previous layout-assertion gap.

## Positive Observations
- The module-level layout assertion is now present and invoked (`assert_tid_layout`), addressing the prior “unused layout proof” issue.
- Trust boundaries for byte serialization and layout are documented more clearly.

## Summary
The update fixes the layout assertion issue, but the two medium issues from the prior review remain: unverified public trait impls and a weak byte-serialization specification. Verification is not yet complete or fully sound at the public API boundary.
