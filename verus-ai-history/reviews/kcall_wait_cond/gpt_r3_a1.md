# Review: kcall_wait_cond (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `get_cond_model` / `put_cond_model` (exec, `wait_cond.rs`) and `spec_wait_cond_result` (spec, `wait_cond.spec.rs`).
  **Description:** The model treats `get_cond` and `put_cond` outcomes as independent and does not track a condition-variable reference token. This permits success of `put_cond` without a proven prior acquisition and allows `GetCondError` returns even when the real PM path would likely force a `PutCondError` (e.g., refcount/ownership coupling). This weakens equivalence and the refcount/resource invariants.
  **Suggested Fix:** Thread a ghost “condvar reference token” (or an explicit predicate) from `get_cond_model` to `put_cond_model` and require it for `put_cond` success, or add a spec invariant coupling `get_cond` failure to `put_cond` failure based on PM semantics.

- **Location:** `cond_wait_model` (exec, `wait_cond.rs`) and timeout semantics (spec, `wait_cond.spec.rs`).
  **Description:** The external-body contract only constrains `TimedOut ⇒ has_alarm` and does not relate outcomes to alarm expiration or progress. As a result, key timeout/liveness properties of `wait_cond` (e.g., alarm-driven wakeup, eventual return under fair scheduling) are unproven in this module.
  **Suggested Fix:** Strengthen the `cond_wait_model` contract with a spec-level predicate capturing alarm-expiration behavior (or explicitly link to a proven condvar spec), and document/assume liveness conditions if they are intentionally out of scope.

### Low
- **Location:** `take_mutex_guard_model` / `mutex_lock_model` / `put_mutex_guard_model` (exec, `wait_cond.rs`).
  **Description:** The model does not track a guard/ownership token across release and reacquisition, so it cannot express that the guard returned by `mutex.lock(None)` is the one stored by `put_mutex_guard`, or that the stored guard is associated with the same mutex and thread.
  **Suggested Fix:** Introduce a ghost guard token (similar to other kcall models) that is produced by `mutex_lock_model` and consumed by `put_mutex_guard_model` to enforce guard identity/ownership invariants.

## Positive Observations
- The exec model mirrors the original control flow precisely, including the stored-result semantics and continuation-error override behavior.
- The spec/proof split is clean, with detailed spec functions and focused proofs of short-circuiting and error propagation.
- The model explicitly guards the x86-32 `usize` width and validates the invalid-timeout error code mapping.

## Summary
The verification robustly captures the control-flow and error-propagation semantics of `wait_cond`, but the spec is weakened by uncoupled PM refcount semantics and minimal `cond.wait` guarantees, leaving timeout/liveness and ownership details under-specified. Tightening the external-body contracts with reference/guard tokens and clearer timeout predicates would improve equivalence and strengthen key safety properties.
