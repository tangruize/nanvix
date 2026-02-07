# Review: sleeping_thread (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Location:** `SleepingThread::join_cond` omission (exec/spec/proof) — **Coverage/Equivalence**. The verified split still omits `join_cond()` entirely; there is no verified stub or trusted external that preserves the Condvar identity. The new docs acknowledge the gap, but callers can still depend on `Condvar` identity without any verified counterpart. **Status:** Not fixed (documented only).
- **Location:** `SleepingThread::thread_state_mut` (exec, external) — **Soundness**. The method remains `#[verifier::external]` with no enforceable postconditions, so mutation can violate `wf`, `spec_id`, and `spec_alarm`. The added trust-boundary comments do not restore soundness. **Status:** Not fixed (documented only).

### Medium
- None.

### Low
- **Location:** Interrupt reason modeling (`InterruptReason` -> `int`) — **Specification clarity**. Constants `INTERRUPT_REASON_*` are now defined, but the link to the real enum values is still a TODO with no proof or checked conversion function. **Status:** Partially fixed (better documentation), but still unproven.

## Positive Observations
- The module now explicitly documents the liveness/timeout and admission-time omissions as out of scope, which addresses the prior “silent gap” concerns.
- Trust boundaries are clearer and more specific, especially around `thread_state_mut` and omitted sync primitives.

## Summary
The update improved documentation and clarified trust boundaries, and it appropriately scoped scheduling and admission-time behavior out of this module. However, the two highest-risk gaps remain: `join_cond()` is still unmodeled, and `thread_state_mut()` is still an unchecked external escape hatch. Verification is not fully sound until those are either modeled or fenced with a trusted external spec; a minor low-severity mapping gap for interrupt reasons also remains.
