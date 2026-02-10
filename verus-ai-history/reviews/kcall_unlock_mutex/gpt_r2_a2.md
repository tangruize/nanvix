# Review: kcall_unlock_mutex (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `take_mutex_guard_model` (exec) / `spec_guard_dropped_and_mutex_unlocked` (spec)
  - **Description:** The error-path mutex state is still underspecified. The model only states that
    when `pm_internally_dropped_guard` is true, the mutex is unlocked, but it never asserts the
    converse (that when the flag is false the mutex remains locked). No new predicate or postcondition
    was added to rule out unintended unlocks on error paths, so the original weakness remains.
  - **Suggested Fix:** Add a complementary predicate or postcondition, e.g.
    `!pm_internally_dropped_guard ==> !spec_guard_dropped_and_mutex_unlocked(mutex_addr)` or an
    explicit `spec_mutex_locked_on_error` predicate in the PM contract.

### Low
- **Location:** `drop_guard_model` (exec) / `spec_guard_dropped_and_mutex_unlocked` (spec)
  - **Description:** The verification still omits the wakeup/notification behavior mentioned in the
    original code comment (“threads to be notified”). No spec predicate or lemma captures waiter
    notification or any liveness aspect of unlock.
  - **Suggested Fix:** Add a predicate for waiter notification (or reference a proven mutex-module
    lemma) and include it in `drop_guard_model`’s postcondition.

## Positive Observations
- The exec/spec/proof split remains clean with explicit trust boundaries and clear documentation.
- Error propagation and guard-drop-on-success behavior are still modeled and proven.

## Summary
The prover’s changes do not address the two substantive gaps from the previous review: the error-path
mutex state remains underconstrained, and unlock notification/liveness is still not specified. The
verification is sound for the control-flow skeleton but incomplete for these behavioral properties.
