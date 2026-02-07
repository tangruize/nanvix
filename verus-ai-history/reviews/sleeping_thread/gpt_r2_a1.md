# Review: sleeping_thread (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `join_cond` (missing) in `verus/split/kernel/pm/thread/sleeping.rs` (exec)
  - **Description:** The original `SleepingThread::join_cond()` is not modeled or verified. The exec module explicitly omits `join_cond()` and condvar identity, so coverage and equivalence are incomplete for a public API function.
  - **Suggested Fix:** Add a boundary model for `Condvar` (even as an opaque token) and verify `join_cond()` as a pure pass-through, or explicitly model/track condvar identity in `ThreadStateView` and include `join_cond()` in exec/spec/proof.

- **Location:** `thread_state_mut` in `verus/split/kernel/pm/thread/sleeping.rs` (exec, non-verus impl)
  - **Description:** `thread_state_mut()` is marked `#[verifier::external]` with no enforced postconditions. This is a trust hole in a core module: callers can mutate `ThreadState` and violate `wf()`, identity, or alarm invariants without any proof obligations.
  - **Suggested Fix:** Replace with verified setters or add a verified wrapper once Verus supports `&mut` returns; in the meantime, model a ghost/borrowed-logic API that enforces preservation of `wf`, `spec_id`, and `spec_alarm`.

### Medium
- **Location:** `ReadyThread::from_state` / `InterruptedThread::from_state` boundary models in `verus/split/kernel/pm/thread/sleeping.rs` (exec)
  - **Description:** The module relies on cross-module assumptions that are not yet proven. In particular, `ReadyThread` admission time is only constrained to `>= 0`, so the equivalence to `clock::now()` (monotonic/current time) is not captured; `InterruptedThread` correctness depends on future confirmation of sibling module specs.
  - **Suggested Fix:** When verifying `ready.rs` and `interrupted.rs`, prove that their specs imply the boundary postconditions, and strengthen admission time to reflect the real `clock::now()` contract (e.g., monotonicity or relation to clock state) if required by scheduler logic.

- **Location:** `spec_valid_reason` in `verus/split/kernel/pm/thread/sleeping.spec.rs`
  - **Description:** The mapping of `InterruptReason` to `int` tags (0/1) is assumed via TODOs, not proven against the Rust enum representation. This is a small but real semantic gap in equivalence.
  - **Suggested Fix:** Connect the enum representation to the spec (e.g., via a verified conversion function or a lemma in the `InterruptReason` verification) and discharge the TODOs.

### Low
- **Location:** Module-level properties (spec/proof)
  - **Description:** Liveness properties around `alarm` (e.g., eventual wakeup/timeout) are explicitly out of scope, so the verification only proves storage/retrieval of the alarm, not its effect on scheduling.
  - **Suggested Fix:** If required by the system’s correctness story, add scheduler-level liveness proofs that connect `alarm` to wakeup events.

## Positive Observations
- Specifications cleanly preserve identity, mutex accounting, drop safety, and well-formedness across `from_state`, `wakeup`, and `interrupt`.
- Alarm handling and thread data area setters/getters are specified precisely and proven.
- Exec/spec/proof split is clean and well-documented, with explicit trust boundaries and cross-module TODOs.

## Summary
The verification is solid for the core data-flow properties of SleepingThread, but it omits `join_cond()` entirely and relies on external/assumed boundaries (`thread_state_mut`, `clock_now`, sibling module constructors) that leave gaps in coverage and equivalence. Addressing these trust holes and finishing cross-module checks would significantly strengthen the soundness story. Overall, good structure and documentation, but key public API coverage is missing.
