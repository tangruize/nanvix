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
  - **Description:** The updated code still treats `get_cond` and `put_cond` as independent outcomes and asserts `spec_cond_ref_released` on `put_cond` success without any precondition that a reference was acquired. There is likewise no ownership/state link between `take_mutex_guard`, `mutex_lock`, and `put_mutex_guard`. This allows executions that violate reference-count/guard ownership rules, so resource-safety properties are not actually proven.
  - **Suggested Fix:** Introduce an abstract state (e.g., `cond_ref_held`, `guard_held`) and strengthen external-body contracts so `put_cond` success requires a prior successful `get_cond`, and `put_mutex_guard` success requires successful `lock` following a prior `take_mutex_guard`. Prove state preservation across the pipeline.

- **Liveness/timing semantics are still not modeled**
  - **Location:** `cond_wait_model` in `wait_cond.rs` and spec scope notes.
  - **Description:** The model still only constrains `TimedOut` to occur when an alarm is present; it does not model actual alarm values or temporal guarantees. Liveness properties such as “finite timeout eventually returns TimedOut if not signaled” or “no TimedOut before the alarm time” remain unproven.
  - **Suggested Fix:** Connect to a verified condvar/clock model with temporal guarantees or add a logical-time abstraction and lift those properties into `wait_cond`.

### Low
- **Architecture assumption remains implicit**
  - **Location:** `wait_cond.rs` (uses `u32` parameters and `USIZE_MAX_X86_32`).
  - **Description:** Equivalence to the original `usize`-based API still relies on the x86-32 assumption without an explicit precondition or compile-time guard in `wait_cond_model`.
  - **Suggested Fix:** Add an explicit architecture precondition (or compile-time guard) to prevent misuse on non-32-bit targets.

## Positive Observations
- The stored-result postcondition remains correctly asserted via `spec_is_stored_result_return`.
- Control-flow equivalence and continuation error overriding semantics remain precise.
- Verification passes for the updated module (`./verus-ai/scripts/verify.sh kcall_wait_cond`).

## Summary
The latest update does not resolve the remaining resource-invariant, liveness/timing, or architecture-precondition gaps. Verification is improved for stored-result returns but still incomplete and potentially unsound with respect to resource ownership and liveness properties.
