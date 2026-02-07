# Review: interrupted (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Location:** `InterruptedThread::thread_state_mut` (exec, `verus/split/kernel/pm/thread/interrupted.rs`)
  **Description:** The method is marked `#[verifier::external]` with no machine-checked postconditions, yet it returns `&mut ThreadState` and allows arbitrary mutation by callers. This creates a trust hole in a core state type: callers can violate `wf()` and identity invariants that the proofs assume, so the verification is only conditionally sound.
  **Suggested Fix:** Refactor to avoid returning `&mut ThreadState` (provide verified setters for specific fields), or introduce a trusted wrapper with explicit `ensures` for identity and `wf` preservation once Verus supports the signature. If keeping it external, restrict and verify all call sites to re-establish `wf()` and identity.

### Medium
- **Location:** `join_cond` (missing; original `src/kernel/src/pm/thread/interrupted.rs`, no exec/spec/proof equivalent)
  **Description:** The original `join_cond()` function is omitted entirely from the verified module. This fails coverage and leaves synchronization behavior involving join conditions outside the verification boundary.
  **Suggested Fix:** Add a boundary model for `Condvar` and a verified (or `external_body`) `join_cond()` spec that at least preserves identity and ties the returned condition variable to the underlying `ThreadState`.

### Low
- **Location:** `ReadyThread` boundary model (exec/spec in `interrupted.rs` / `interrupted.spec.rs`)
  **Description:** The boundary model omits the real `ReadyThread::admission_time` field set by `clock::now()`. Scheduling-related properties or time-based fairness are therefore not represented in this verification.
  **Suggested Fix:** If scheduling correctness is in scope, extend the boundary model to include `admission_time` (or add a separate ReadyThread verification that proves the real `from_state` behavior and composes with this module).

## Positive Observations
- All core state-transition properties for `resume()` are specified: identity preservation, reason stamping, and well-formedness are enforced and verified.
- `InterruptedThread::wf()` cleanly composes the ThreadState invariant with a validated reason tag, matching the intended enum domain.
- The spec/proof split is clean, with explicit lemmas for identity, mutex accounting, drop safety, and stack preservation.

## Summary
Verification passes and captures the main safety properties of `resume()` and construction, but it is incomplete around synchronization (`join_cond`) and relies on an unverified mutable accessor that can break invariants. Addressing the external mutable accessor and adding a minimal `join_cond` boundary spec would materially improve soundness and coverage.
