# Review: sleeping_thread (gpt-5.2-codex) - Re-review

## Grade: C+

## Issues Found

### High
- **Location:** `SleepingThread::join_cond` (missing in `verus/split/kernel/pm/thread/sleeping.rs`, `sleeping.spec.rs`, `sleeping.proof.rs`).
  **Status:** **Not fixed.** The updated files explicitly document that `Condvar` and `join_cond()` are omitted from the verification model, but no boundary model or stub accessor was added to preserve condvar identity or even expose the API surface (sleeping.rs:36-49; sleeping.spec.rs:21-23). This still leaves the public API partially unverified and violates coverage of the real interface.

- **Location:** `SleepingThread::thread_state_mut` (`verus/split/kernel/pm/thread/sleeping.rs`).
  **Status:** **Not fixed.** The function remains `#[verifier::external]` with no machine-checked postconditions and still allows arbitrary mutation of `ThreadState` that can violate `wf`, `spec_id`, and `spec_alarm` (sleeping.rs:396-433). The added comments describe obligations but do not provide verified guarantees, so the soundness hole remains.

### Medium
- **Location:** `spec_valid_reason` / `INTERRUPT_REASON_*` (`verus/split/kernel/pm/thread/sleeping.spec.rs`).
  **Status:** **Not fixed.** The mapping from `InterruptReason` to `int` remains an assumed encoding with TODO notes (sleeping.spec.rs:66-76, 141-150). There is still no verified correspondence to actual enum values, so interrupt reasoning depends on an unproven assumption.

### Low
- **Location:** `clock_now()` boundary (`verus/split/kernel/pm/thread/sleeping.rs`).
  **Status:** **Not fixed.** The `external_body` clock model is still duplicated and only ensures `result >= 0` with a TODO to centralize (sleeping.rs:79-87). This remains a trust boundary duplication issue.

- **Location:** `ReadyThread` boundary divergence (`verus/split/kernel/pm/thread/sleeping.rs`).
  **Status:** **Not fixed.** The updated file still notes the mismatch between boundary models across modules (sleeping.rs:56-59). This structural inconsistency remains and can complicate cross-module composition.

## Resolved Issues
- **Location:** `SleepingThread::wakeup` postconditions (`verus/split/kernel/pm/thread/sleeping.rs`).
  **Status:** **Fixed.** The spec now includes `result.spec_admission_time() >= 0` (sleeping.rs:284-293), matching the ReadyThread boundary’s `clock_now()` contract.

## New Issues Introduced
- None identified. The changes are mostly documentation and spec strengthening for `wakeup()`.

## Verification Completeness & Soundness
- **Not complete.** High-severity gaps remain: missing `join_cond()` modeling and the unrestricted `thread_state_mut()` escape hatch. These are the same critical soundness/coverage issues from the previous review and were not actually fixed.

## Summary
The prover fixed the `wakeup()` admission-time guarantee but did not address the two high-severity issues: `join_cond()` remains unmodeled, and `thread_state_mut()` is still an unchecked escape hatch. The enum-tag assumption is still unproven, and boundary/model inconsistencies remain. Verification is improved but still incomplete and not fully sound.
