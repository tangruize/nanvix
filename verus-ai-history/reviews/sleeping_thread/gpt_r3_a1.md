# Review: sleeping_thread (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `SleepingThread::join_cond` (missing in `verus/split/kernel/pm/thread/sleeping.rs`, `sleeping.spec.rs`, `sleeping.proof.rs`).
  **Description:** The original public API exposes `join_cond()`, but the verified model omits the function and the `Condvar` field entirely. This violates the coverage requirement and leaves the condvar identity/property unverified.
  **Suggested Fix:** Add a boundary model for `Condvar` (opaque token) and a verified `join_cond()` accessor, or at minimum add an `#[verifier::external]` stub with a spec postcondition about identity preservation.
- **Location:** `SleepingThread::thread_state_mut` (`verus/split/kernel/pm/thread/sleeping.rs`, non-verus impl).
  **Description:** The function is marked `#[verifier::external]` with no machine-checked postconditions, allowing arbitrary mutation of `ThreadState` that can break `wf`, `spec_id`, `spec_alarm`, and mutex/drop-safety invariants. This is a significant soundness hole for a public API.
  **Suggested Fix:** Replace with verified setters (or a verified wrapper that only exposes safe mutations), or add a verified specification that explicitly preserves `wf`, `spec_id`, and `spec_alarm`, then refactor callers to adhere to it.

### Medium
- **Location:** `spec_valid_reason` / `INTERRUPT_REASON_*` (`sleeping.spec.rs`).
  **Description:** The int-tag encoding of `InterruptReason` is assumed (0/1) with only a TODO note; there is no proved correspondence to the actual enum values. This makes the interrupt reasoning rely on an unproven assumption.
  **Suggested Fix:** Add a verified conversion lemma in the `InterruptReason` module (or a shared spec) proving the mapping, and use that lemma here to justify `spec_valid_reason`.
- **Location:** `SleepingThread::wakeup` postconditions (`sleeping.rs` exec).
  **Description:** The spec does not propagate `ReadyThread::from_state`'s `spec_admission_time() >= 0` guarantee, leaving admission time unconstrained in this API even though the implementation sets it via `clock_now()`. This makes the spec weaker than the actual behavior and may block downstream proofs.
  **Suggested Fix:** Add `ensures result.spec_admission_time() >= 0` (or a more precise relation to `clock_now()` if required) to `wakeup()`.

### Low
- **Location:** `clock_now()` boundary (`sleeping.rs` exec).
  **Description:** The `external_body` clock model is a trusted assumption with only `result >= 0`, and it is duplicated across modules. While documented, this still creates a thin soundness boundary.
  **Suggested Fix:** Centralize the boundary in a shared module and, if possible, strengthen/align its spec with the verified `ready.rs` contract.
- **Location:** `ReadyThread` boundary model (`sleeping.rs` exec, header comment).
  **Description:** The boundary type here includes `admission_time`, while the boundary in `interrupted.rs` omits it, causing cross-module mismatch. This reduces split quality and makes composition of proofs fragile.
  **Suggested Fix:** Define a shared canonical boundary model for `ReadyThread` used consistently across modules.

## Positive Observations
- Clear documentation of the verification model and explicit trust boundaries.
- Core transitions (`from_state`, `wakeup`, `interrupt`) preserve identity, mutex accounting, drop safety, and well-formedness.
- Thread data area setters/getters and view equality are explicitly proven.

## Summary
The verification is solid for core safety properties and transitions, but it falls short on coverage and soundness due to the missing `join_cond()` and the unverified `thread_state_mut()` escape hatch. Addressing the enum mapping assumption and strengthening `wakeup()`'s admission-time guarantees would improve spec fidelity, while harmonizing boundary models would improve split quality.
