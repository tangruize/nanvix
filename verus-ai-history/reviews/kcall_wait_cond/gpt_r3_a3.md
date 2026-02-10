# Review: kcall_wait_cond (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `get_cond_model` / `put_cond_model` (exec, `wait_cond.rs`) and `spec_wait_cond_result` (spec, `wait_cond.spec.rs`).
  **Description:** There is still no reference token or invariant that couples `get_cond` success to `put_cond` success. The model continues to allow impossible combinations, weakening refcount/resource invariants and equivalence. No evidence of a fix is present in the updated files.
  **Suggested Fix:** Thread a ghost condition-variable reference token from `get_cond_model` to `put_cond_model`, or add an explicit spec invariant coupling `GcError` to `PcError` when no reference exists.

- **Location:** `cond_wait_model` (exec, `wait_cond.rs`) and timeout semantics (spec, `wait_cond.spec.rs`).
  **Description:** The external-body contract still only states `TimedOut ⇒ has_alarm` and does not capture alarm-expiration or progress/liveness guarantees. Timeout behavior and eventual return remain unproven, so the previous issue is not addressed.
  **Suggested Fix:** Strengthen postconditions with an alarm-expiration predicate or link the model to a verified condvar spec that captures timeout/liveness behavior.

### Low
- **Location:** `mutex_lock_model` / `put_mutex_guard_model` (exec, `wait_cond.rs`).
  **Description:** There is still no guard token/ownership coupling across lock acquisition and `put_mutex_guard`, so guard provenance and same-mutex/thread invariants are not modeled.
  **Suggested Fix:** Introduce a ghost guard token produced by `mutex_lock_model` and consumed by `put_mutex_guard_model` to enforce guard provenance.

## Positive Observations
- The exec model continues to mirror the original control flow and stored-result override semantics.
- The spec/proof split remains clean with detailed short-circuit and error-propagation lemmas.
- Architecture guard and invalid-timeout mapping remain explicitly verified.

## Summary
I found no substantive changes in the updated exec/spec/proof files that address the prior gaps. The verification remains incomplete for refcount coupling, guard provenance, and timeout/liveness semantics, so it is still not fully sound.
