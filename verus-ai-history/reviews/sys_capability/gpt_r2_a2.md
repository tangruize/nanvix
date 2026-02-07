# Review: sys_capability (gpt-5.2-codex)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- None.

### Low
- **Location:** `Capability` exec module header and enum impl in `verus/split/kernel/pm/sys/capability.rs` (lines 17-23, 58-138).
  **Description:** The previous review flagged API additions (`to_u32`, `PartialEq`/`Eq` derives) that are not in the original source. The prover added documentation stating these are verification-only additions (lines 17-23), which is an improvement, but the items remain public and are still compiled unconditionally. If strict drop-in equivalence is required, the issue remains partially fixed.
  **Suggested Fix:** Gate these items behind `#[cfg(verus)]`/feature flag or wrap them in a verification-only module so the exec artifact does not expose extra APIs in production builds.

## Positive Observations
- The added “Verification Additions” section documents the extra API surface explicitly, which addresses the previous “no documentation” concern (capability.rs lines 17-23).
- Exec conversion logic remains exact and explicit; no reliance on enum layout (`#[repr]`) was introduced (capability.rs lines 107-113, 132-137).
- Spec and proof files are cleanly separated and introduce no `external_body` or `assume` shortcuts (capability.spec.rs and capability.proof.rs).

## Summary
The prover did add explicit documentation noting the verification-only additions, so the prior issue is acknowledged and partially addressed. However, the extra public APIs (`to_u32`, `PartialEq`/`Eq` derives) are still present without gating, so strict drop-in equivalence is not fully restored. Verification appears sound otherwise; no new correctness issues were introduced.
