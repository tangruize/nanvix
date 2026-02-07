# Review: sleeping_thread (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `SleepingThread::join_cond` omission (exec/spec/proof) — **Coverage/Equivalence**. The method is still omitted; no verified stub or trusted external models `Condvar` identity. The new comment claims identity is “trivially preserved,” but there is no modeled field or proof, and `thread_state_mut()` can mutate state outside verification, so the claim is unsubstantiated. **Status:** Not fixed.
- **Location:** `SleepingThread::thread_state_mut` (exec, external) — **Soundness**. Still `#[verifier::external]` with no enforceable postconditions, allowing unchecked mutation that can violate `wf`, `spec_id`, and `spec_alarm`. Documentation does not restore soundness. **Status:** Not fixed.

### Medium
- None.

### Low
- **Location:** Interrupt reason modeling (`InterruptReason` -> `int`) — **Specification clarity**. The mapping to `INTERRUPT_REASON_*` remains a TODO with no proof or checked conversion. **Status:** Not fixed.

## Positive Observations
- The trust-boundary rationale for condvar elision is clearer, though it remains a documented assumption rather than a verified fact.

## Summary
The update is largely explanatory; it does not close the two high-severity gaps from the prior review. `join_cond()` remains unmodeled, and `thread_state_mut()` remains an unchecked escape hatch, so verification is still not fully sound. A minor low-severity mapping gap for interrupt reasons also persists.
