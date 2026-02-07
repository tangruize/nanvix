# Review: sys_capability (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- None.

### Low
- **Location:** `Capability` exec module in `verus/split/kernel/pm/sys/capability.rs` (lines 17-23, 58-138).
  **Description:** The prior low-severity issue remains: `to_u32` and `PartialEq`/`Eq` derives are still present as public API surface in the exec module and are compiled unconditionally. The latest update does not change these items, so strict drop-in equivalence with the original source is still violated.
  **Suggested Fix:** Gate these additions behind a verification-only feature (`#[cfg(verus)]` or similar), or move them to a verification-only wrapper module to keep production API unchanged.

## Positive Observations
- No new changes were introduced in the exec/spec/proof files; the verification logic remains stable and sound.
- Conversion behavior and error reporting remain explicit and fully specified without relying on enum layout.
- No `external_body` or `assume` shortcuts are introduced in the spec/proof.

## Summary
No substantive updates were made in the target files, so the previous low-severity API-gating issue remains unresolved. Verification still appears sound, but strict drop-in equivalence is not yet restored.
