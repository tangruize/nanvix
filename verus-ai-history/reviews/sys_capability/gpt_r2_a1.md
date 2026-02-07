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
- **Location:** `Capability` enum (exec) in `verus/split/kernel/pm/sys/capability.rs`.
  **Description:** The verified exec module adds `to_u32` and derives `PartialEq`/`Eq`, which are not present in the original source. This extends the public API and is not strictly semantically equivalent to the original module if the verified code is used as a drop-in replacement.
  **Suggested Fix:** Gate these additions behind a verification-only feature (e.g., `#[cfg(verus)]`), move them into a verification-only wrapper module, or document that the verified artifact is not intended as a production drop-in.

## Positive Observations
- All original functionality (enum variants and `TryFrom<u32>`) is covered and re-implemented with explicit, verified discriminant mappings.
- Specs precisely capture the intended conversion behavior (success iff in [0,4], exact error code/message on failure) without relying on layout assumptions.
- No `assume` or `external_body` is used in this module; proofs verify directly from definitions.
- Spec/proof are cleanly separated and included from the exec module, improving split quality and modularity.

## Summary
The verification is strong for this small module: conversions are fully specified and proven, invariants are adequate, and no trusted axioms are introduced. The only gap relative to strict equivalence is the additional public API surface in the verified exec code, which should be gated or isolated if drop-in equivalence is required.
