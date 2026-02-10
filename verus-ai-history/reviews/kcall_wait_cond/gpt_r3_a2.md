# Review: kcall_wait_cond (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `get_cond_model` / `put_cond_model` (exec, `wait_cond.rs`) and `spec_wait_cond_result` (spec, `wait_cond.spec.rs`).
  **Description:** The model still treats `get_cond` and `put_cond` outcomes as independent; there is no reference token or invariant tying `put_cond` success to a prior `get_cond` success. This allows impossible combinations and weakens refcount/resource invariants, so the previous issue is not actually fixed.
  **Suggested Fix:** Thread a ghost condition-variable reference token from `get_cond_model` to `put_cond_model`, or add an explicit spec invariant coupling `GcError` to `PcError` when no reference exists.

- **Location:** `cond_wait_model` (exec, `wait_cond.rs`) and timeout semantics (spec, `wait_cond.spec.rs`).
  **Description:** The external-body contract remains limited to `TimedOut ⇒ has_alarm` with no alarm-expiration or progress/liveness guarantees. This means timeout behavior and eventual return are still unproven; the prior issue remains.
  **Suggested Fix:** Strengthen the postconditions with an alarm-expiration predicate or link the model to a proven condvar spec that captures timeout/liveness behavior.

### Low
- **Location:** `mutex_lock_model` / `put_mutex_guard_model` (exec, `wait_cond.rs`).
  **Description:** There is still no guard token or ownership coupling across lock acquisition and storing the guard, so guard identity and same-mutex/thread invariants are not modeled. This is the same gap as before.
  **Suggested Fix:** Introduce a ghost guard token produced by `mutex_lock_model` and consumed by `put_mutex_guard_model` to enforce guard provenance.

## Positive Observations
- The exec model continues to mirror the original control flow and stored-result override semantics.
- The spec/proof split remains clean with detailed short-circuit and error-propagation lemmas.
- Architecture guard and invalid-timeout mapping remain explicitly verified.

## Summary
No substantive changes were found in the exec/spec/proof files; the prior issues remain unaddressed. Verification is still incomplete for condvar refcount coupling, guard ownership, and timeout/liveness semantics, so the model is not yet fully sound.
