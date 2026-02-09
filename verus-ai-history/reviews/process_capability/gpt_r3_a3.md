# Review: process_capability (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Priority:** Medium  
  **Location:** `Capabilities` struct (`capability.rs`, exec) and `wf()` invariant (`capability.spec.rs`, spec)  
  **Description:** The encapsulation mismatch remains: `Capabilities` still exposes `pub bits`, which allows external construction or mutation of non-`wf` states. This is still strictly more permissive than the original private tuple field, and the proofs only cover API-reachable, `wf()`-preserving states. The updated files still document the deviation, indicating the issue was not fixed.  
  **Suggested Fix:** Reintroduce a private storage boundary (e.g., a private inner struct with `pub` bits only for specs, or a sealed wrapper type) so external code cannot bypass the verified API. If Verus visibility constraints require `pub` field access in specs, use an internal representation plus `View`/opaque spec accessors and keep the exec field private to restore semantic equivalence.

### Low
- None.

## Positive Observations
- Coverage for `set`, `clear`, and `has` remains complete with accurate postconditions.
- Mask/discriminant equivalence proof remains valid and no new `assume`/`external_body` usage appears.
- Spec/proof/exec separation is still clean and verification passes.

## Summary
The re-review shows no substantive changes: the earlier encapsulation issue persists and remains the only blocking item for full equivalence. Verification continues to pass, but the public field keeps the exec interface more permissive than the original.
