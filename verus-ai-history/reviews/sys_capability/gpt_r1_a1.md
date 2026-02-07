# Review: sys_capability (gpt-5.2-codex)

## Grade: A-

## Issues Found

### Medium
- **Location:** `TryFrom<u32> for Capability::try_from` (exec, `capability.rs`)
  - **Description:** The trait implementation lives outside the `verus!` block and is therefore
    unverified; coverage is effectively delegated to `try_from_u32`, but the wrapper itself is not
    checked by Verus. This leaves a small but real coverage gap relative to the original function.
  - **Suggested Fix:** Move the `TryFrom<u32>` implementation into the `verus!` block with an
    `ensures` clause equivalent to `try_from_u32`, or add a verified wrapper function that the trait
    implementation calls and prove a lemma that they are equivalent.

### Low
- **Location:** `Capability::spec_from_discriminant` (spec, `capability.spec.rs`)
  - **Description:** The spec uses `recommends` instead of `requires`, meaning it is defined for
    invalid discriminants and defaults to `ProcessManagement`. This can be used in proofs on
    invalid inputs, which does not match executable behavior (which returns `Err`).
  - **Suggested Fix:** Strengthen the spec with `requires Self::spec_is_valid_discriminant(value)`
    (or return an `Option<Capability>`), and update any dependent lemmas accordingly.

## Positive Observations
- All enum variants and discriminant mappings are explicitly modeled and match the original
  implementation.
- The error path is fully specified, including error code and message string, which tightly
  matches the original behavior.
- No `assume` or `external_body` is used in this module, and verification passes cleanly.
- Exec/spec/proof are well separated, and proof lemmas cover key discriminant properties.

## Summary
The verification is strong and mostly faithful to the original code, with only minor gaps.
Addressing the unverified `TryFrom` wrapper and tightening `spec_from_discriminant` would make the
coverage and spec alignment complete.
