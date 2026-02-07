# Review: sleeping_thread (gpt-5.2-codex) - Re-review

## Grade: C+

## Issues Found

### High
- **Location:** `SleepingThread::join_cond` (missing in `verus/split/kernel/pm/thread/sleeping.rs`, `sleeping.spec.rs`, `sleeping.proof.rs`).
  **Status:** **Not fixed.** The latest files still explicitly omit `Condvar` and `join_cond()` from the verification model (sleeping.rs:36-49; sleeping.spec.rs:21-23). No boundary model or stub accessor was added to preserve the API surface or condvar identity. This remains an uncovered public API and a coverage gap.

- **Location:** `SleepingThread::thread_state_mut` (`verus/split/kernel/pm/thread/sleeping.rs`).
  **Status:** **Not fixed.** `thread_state_mut()` remains `#[verifier::external]` with no machine-checked postconditions (sleeping.rs:396-433). The trust comments are unchanged and there is still no verified guarantee that `wf`, `spec_id`, or `spec_alarm` are preserved. This continues to be a soundness hole.

### Medium
- **Location:** `spec_valid_reason` / `INTERRUPT_REASON_*` (`verus/split/kernel/pm/thread/sleeping.spec.rs`).
  **Status:** **Not fixed.** The enum-to-int encoding remains a TODO assumption (sleeping.spec.rs:66-76, 141-150). There is still no verified lemma proving the mapping to the real enum values.

### Low
- **Location:** `clock_now()` boundary (`verus/split/kernel/pm/thread/sleeping.rs`).
  **Status:** **Not fixed.** The `external_body` clock model is still duplicated and only ensures `result >= 0` with a TODO to centralize (sleeping.rs:79-87).

- **Location:** `ReadyThread` boundary divergence (`verus/split/kernel/pm/thread/sleeping.rs`).
  **Status:** **Not fixed.** The boundary model mismatch across modules remains explicitly noted (sleeping.rs:56-59). Structural inconsistency persists.

## Resolved Issues
- None. No new fixes were detected relative to the previous review.

## New Issues Introduced
- None identified. The updated files appear unchanged in substance relative to the last re-review.

## Verification Completeness & Soundness
- **Not complete.** The two high-severity issues remain unaddressed, leaving the verification incomplete and not fully sound.

## Summary
The prover did not fix the previously identified high-severity issues. `join_cond()` is still omitted, and `thread_state_mut()` remains an unchecked escape hatch. The enum-tag assumption and boundary model inconsistencies also remain. Verification is still incomplete.
