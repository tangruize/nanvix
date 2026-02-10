# Review: kcall_wait_cond (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Resource invariants between get/put operations remain unmodeled**
  - **Location:** External-body specs in `wait_cond.rs`, spec predicates in `wait_cond.spec.rs`.
  - **Description:** The model still treats `get_cond` and `put_cond` as independent outcomes, and `put_cond` success directly establishes `spec_cond_ref_released` even if `get_cond` failed (no reference acquired). Similarly, there is no ownership/state invariant connecting `take_mutex_guard`, `mutex_lock`, and `put_mutex_guard`. This allows executions that violate reference-count/guard ownership rules, so resource-safety properties are not actually proven.
  - **Suggested Fix:** Introduce an abstract state (e.g., `cond_ref_held`, `guard_held`) and strengthen external-body contracts so `put_cond` success requires a prior successful `get_cond`, and `put_mutex_guard` success requires successful `lock` following a prior `take_mutex_guard`. Prove state preservation across the pipeline.

- **Liveness/timing semantics are still not modeled**
  - **Location:** `cond_wait_model` in `wait_cond.rs` and spec scope notes.
  - **Description:** The model still only constrains `TimedOut` to occur when an alarm is present; it does not model when timeouts occur or eventual wakeups. Key liveness properties (e.g., eventual TimedOut with finite alarm, or no TimedOut before the alarm) remain unproven.
  - **Suggested Fix:** Connect to a verified condvar/clock model with temporal guarantees or add a logical-time abstraction and lift those properties into `wait_cond`.

### Low
- **Architecture assumption remains implicit**
  - **Location:** `wait_cond.rs` (uses `u32` parameters and `USIZE_MAX_X86_32`).
  - **Description:** Equivalence to the original `usize`-based API still relies on the x86-32 assumption without an explicit precondition or compile-time guard in `wait_cond_model`.
  - **Suggested Fix:** Add an explicit architecture precondition (or compile-time guard) to prevent misuse on non-32-bit targets.

## Positive Observations
- The previous high-priority issue is fixed: the exec postcondition now asserts mutex/condvar protocol predicates for stored-result returns via `spec_is_stored_result_return`.
- The stored-result predicate and supporting lemma (`lemma_stored_result_implies_continuation_ok`) correctly capture the continuation-success requirement.
- Control-flow equivalence and error overriding semantics remain precisely modeled.

## Summary
The update correctly strengthens the mutex/condvar postcondition for stored-result returns, addressing the earlier high-priority gap. However, resource-state invariants and liveness/timing semantics are still missing, and the architecture assumption remains implicit. The verification is improved but not fully complete or sound with respect to resource ownership and liveness guarantees.
