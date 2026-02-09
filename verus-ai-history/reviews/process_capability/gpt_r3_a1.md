# Review: process_capability (gpt-5.2-codex)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Priority:** Medium  
  **Location:** `Capabilities` struct (`capability.rs`, exec) and `wf()` invariant (`capability.spec.rs`, spec)  
  **Description:** The verified executable type exposes `pub bits`, which makes the API strictly more permissive than the original tuple struct with a private field. This allows external code to construct or mutate non-`wf` values directly, while the proofs only guarantee correctness for API-reachable, `wf()`-preserving states (via `old(self).wf() ==> self.wf()`). This is a semantic mismatch in encapsulation and reachable-state assumptions.  
  **Suggested Fix:** Preserve encapsulation by hiding raw storage: keep `bits` private and expose spec access through a wrapper (e.g., a private inner struct with `pub` bits used only for specs, or `pub(crate)` visibility plus re-exported APIs). Alternatively, make `Capabilities` non-constructible externally and provide a sealed module boundary so only verified APIs can create instances, while still offering `pub open spec fn` via a `View` or `opaque` spec accessor.

### Low
- None.

## Positive Observations
- All original methods (`set`, `clear`, `has`) are covered with precise postconditions matching the bitwise behavior.
- The explicit mask mapping is proven equivalent to the original `1 << discriminant` formula, avoiding reliance on enum layout.
- No `assume` or `external_body` is used; proofs establish idempotence, round-trips, and `wf()` preservation.
- Spec/proof are cleanly separated from executable code with clear documentation of verification additions and trust boundaries.

## Summary
The verification is strong and well-structured, with solid equivalence proofs and invariant reasoning. The main gap is the public `bits` field, which relaxes encapsulation versus the original and weakens the reachable-state model. Addressing that would bring the verification closer to full semantic equivalence.
