# Review: process_capability (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Location:** `Capabilities` struct (exec) in `verus/split/kernel/pm/process/capability.rs`.
  **Description:** The executable model still exposes `pub bits: u8`, which changes the public API compared to the original private tuple field. This allows external code to construct or mutate non-`wf()` values directly, which the original module forbids. The added “Known deviation” note documents the mismatch but does not fix it, so equivalence remains broken.
  **Suggested Fix:** Restore encapsulation by keeping the exec field private and using a spec-only wrapper or `closed` spec functions for public spec access, or introduce a separate verified wrapper type with private state and public API.

### Medium
- None.

### Low
- None.

## Positive Observations
- The `to_mask` contract still explicitly links to the shift-based formula, and verification passes.
- No `assume` or `external_body` is used in this module.
- Bitwise behavior, idempotence, and `wf()` preservation are fully proven for API-reachable states.

## Summary
The only outstanding issue is the unchanged public `bits` field, which keeps the verified exec API more permissive than the original. Documentation acknowledges the deviation but does not resolve the semantic mismatch. Otherwise, the verification is sound and complete for the API-reachable state space.
