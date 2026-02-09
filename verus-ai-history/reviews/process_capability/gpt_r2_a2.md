# Review: process_capability (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Location:** `Capabilities` struct (exec) in `verus/split/kernel/pm/process/capability.rs`.
  **Description:** The executable model still exposes `pub bits: u8`, which changes the public API compared to the original private tuple field. This allows external code to construct or mutate non-`wf()` values directly, which the original module forbids. The proof only guarantees correctness for API-reachable states, so this remains a semantic and invariant-equivalence gap.
  **Suggested Fix:** Restore encapsulation by keeping the exec field private and providing spec access through a `View`/ghost wrapper or `#[verifier::opaque]`/`closed` spec fns. Alternatively, introduce a separate spec-only wrapper type for `pub open spec fn` exposure while keeping the exec field non-public.

### Medium
- None.

### Low
- None.

## Positive Observations
- The `to_mask` contract now explicitly links to the shift-based formula via `spec_pow2_mask`, and the proof discharges this using `lemma_mask_matches_discriminant`.
- All original functions remain covered and the verification run passes with no `assume` or `external_body` usage.
- Bitwise behavior, idempotence, round-trips, and `wf()` preservation are still fully proven.

## Summary
The previous spec-strength issue is fixed, but the public `bits` field remains a real API and equivalence mismatch with the original module. If encapsulation cannot be restored, the verification should acknowledge that the executable interface is strictly more permissive than the source. Otherwise, the proofs are sound and complete for the API-reachable state space.
