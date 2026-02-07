# Review: sys_capability (gpt-5.2-codex)

## Grade: A-

## Issues Found

### Low
- **Location:** `Capability::spec_from_discriminant` (spec, `capability.spec.rs`)
  - **Description:** The spec still uses `recommends` instead of `requires`, so it remains defined for invalid discriminants and defaults to `ProcessManagement`. This enables proofs over invalid inputs that do not match executable behavior (which returns `Err`).
  - **Suggested Fix:** Strengthen the spec with `requires Self::spec_is_valid_discriminant(value)` (or return an `Option<Capability>`), and update dependent lemmas accordingly.

## Positive Observations
- The `TryFrom<u32>` implementation now lives inside the `verus!` block and carries an `ensures` clause consistent with `try_from_u32`, closing the previous verification gap.
- No `external_body` or `assume` is introduced, and the exec/spec alignment of discriminants remains explicit.

## Summary
The previously reported unverified `TryFrom` wrapper is fixed, but the spec for `spec_from_discriminant` remains too permissive. Aside from that low-severity mismatch, the verification appears sound and complete.
