# Review: process_capability (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- **Location:** `Capabilities` struct visibility and `set`/`clear` preconditions (exec: `verus/split/kernel/pm/process/capability.rs`).
  **Description:** The verified version makes `bits` public and relies on `wf()` as a precondition for
  `set`/`clear`, but the original type is a private tuple struct. This changes the API and permits
  callers to construct non-`wf` states, while the specs only guarantee behavior for `wf` inputs.
  That weakens equivalence and leaves reachable behaviors (with invalid upper bits) unverified.
  **Suggested Fix:** Keep the runtime field private (e.g., tuple struct as in the original) and
  expose spec access via `spec_bits()`/`View`, or make `bits` `pub(crate)` with a module-level
  invariant `invariant self.wf()` and only construct through verified constructors.

### Medium
- **Location:** `spec_mask`, `spec_pow2_mask`, `wf` (spec: `verus/split/kernel/pm/process/capability.spec.rs`),
  `to_mask` (exec: `verus/split/kernel/pm/process/capability.rs`).
  **Description:** The spec and exec mapping are hardcoded for exactly five capability variants and
  `wf()` forbids any bit >= 5. The original implementation computes `1 << capability as u8`, which
  would naturally extend if the enum gains new variants or explicit discriminants. The verification
  assumes a closed-world enum; future changes could silently diverge from the original semantics.
  **Suggested Fix:** Define masks in terms of `Capability::to_u32()`/`spec_discriminant()` and define
  `wf()` as “only bits corresponding to valid discriminants are set” (e.g., by quantifying over
  `spec_is_valid_discriminant`). Alternatively add a proof-time assertion that the enum is closed
  to the current five variants.

### Low
- None.

## Positive Observations
- All original functions (`set`, `clear`, `has`) are present with verified exec/spec/proof coverage.
- The verification cleanly separates exec/spec/proof, and documents trust boundaries.
- No `assume` or `external_body` is used in this module, and key properties like idempotence,
  roundtrips, and well-formedness preservation are proven.

## Summary
The verification is thorough for the intended bitfield semantics and the core API, but it weakens
API equivalence by making the raw bits publicly mutable and by assuming a fixed, closed set of
capability variants. Restoring encapsulation (or adding a true invariant) and tying masks to the
verified discriminant mapping would close the main gaps.
