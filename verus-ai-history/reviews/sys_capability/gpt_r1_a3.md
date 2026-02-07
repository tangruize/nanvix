# Review: sys_capability (gpt-5.2-codex)

## Grade: A-

## Issues Found

### Low
- **Location:** `Capability::spec_from_discriminant` (spec, `capability.spec.rs`)
  - **Description:** The spec still uses `recommends` instead of `requires`, so it remains defined for invalid discriminants and defaults to `ProcessManagement`. This allows proofs over invalid inputs that do not match executable behavior (which returns `Err`).
  - **Suggested Fix:** Strengthen the spec with `requires Self::spec_is_valid_discriminant(value)` (or return an `Option<Capability>`), and update dependent lemmas accordingly.

## Positive Observations
- The `TryFrom<u32>` implementation remains verified inside the `verus!` block with ensures clauses aligned to `try_from_u32`.
- No `external_body` or `assume` is introduced, and exec/spec discriminant mappings are explicit.

## Summary
The prior issue about `spec_from_discriminant` being too permissive has not been fixed. Other previously noted fixes remain in place and no new issues were found.
