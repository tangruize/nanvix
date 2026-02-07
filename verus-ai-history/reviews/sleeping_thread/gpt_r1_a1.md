# Review: sleeping_thread (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `SleepingThread::join_cond` (exec/spec/proof) — **Coverage/Equivalence**. The original API exposes `join_cond()` but the verified split omits it entirely, so callers can observe or depend on the condition variable identity without any verified counterpart. **Suggested Fix:** Add a verified stub or a modeled opaque token for `Condvar` and specify identity preservation (at least `result` corresponds to `state.join_cond()`), or explicitly gate the omission with a trusted external and document the guarantee it must uphold.
- **Location:** `SleepingThread::thread_state_mut` (exec, external) — **Soundness**. The method is marked `#[verifier::external]` with no checked postconditions, allowing unchecked mutation of `ThreadState` that can violate `wf`, `spec_id`, and `spec_alarm`. This weakens the module’s soundness guarantees. **Suggested Fix:** Replace with a verified method once Verus supports `&mut` returns, or refactor callers to verified setters; until then, provide a wrapper with explicit pre/postconditions and audit all call sites.

### Medium
- **Location:** `ReadyThread` boundary model (exec/spec) — **Equivalence**. The boundary model intentionally omits the `admission_time` field that is set in the real `ReadyThread::from_state`; thus, the verified `wakeup()` does not capture this scheduling effect. **Suggested Fix:** Either extend the model with an abstract admission-time field and postcondition (e.g., `spec_admission_time`), or explicitly state that this property is intentionally out of scope in the module’s correctness contract.
- **Location:** `SleepingThread` specs (`wf`, `alarm`, `interrupt` reasoning) — **Properties/Liveness**. There is no property tying `alarm` to eventual wakeup or interrupt-by-timeout behavior; the model only carries the alarm value. This leaves the key timeout-liveness aspect unproven. **Suggested Fix:** Add a higher-level liveness/scheduler contract (possibly in the scheduler module) that relates `alarm` to wakeup/interrupt transitions, or document that liveness is proven elsewhere.

### Low
- **Location:** `interrupt()` modeling (`InterruptReason` -> `int`) — **Specification clarity**. The mapping of enum variants to integer tags is assumed but not linked to the real `InterruptReason` conversion logic. **Suggested Fix:** Introduce a spec function converting `InterruptReason` to `int` with a proof that all enum values map to `spec_valid_reason`.

## Positive Observations
- Most of the original API surface is modeled with clear pre/postconditions (identity, wf, mutex accounting, drop safety).
- The split cleanly separates spec/proof/exec, and proofs include useful lemmas for transitions and view equality.
- Trust boundaries are documented in the exec file, making assumptions explicit.

## Summary
The verification captures core safety properties around identity preservation and state transitions, but it omits `join_cond()` and relies on an external `thread_state_mut()` that weakens soundness. Important scheduling/liveness effects (alarm-driven timeouts and ready admission time) are not modeled. Addressing these gaps would significantly strengthen coverage and equivalence while keeping the split structure intact.
