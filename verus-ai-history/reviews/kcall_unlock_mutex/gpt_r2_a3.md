# Review: kcall_unlock_mutex (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `take_mutex_guard_model` (exec) / `spec_guard_dropped_and_mutex_unlocked` (spec)
  - **Description:** The error-path mutex state remains underspecified. There is still no assertion
    that `pm_internally_dropped_guard == false` implies the mutex remains locked; only the
    one-way implication (flag true ⇒ unlocked) exists. The updated files do not add a
    complementary postcondition or predicate, so the original weakness persists.
  - **Suggested Fix:** Add a complementary predicate or postcondition, e.g.
    `!pm_internally_dropped_guard ==> !spec_guard_dropped_and_mutex_unlocked(mutex_addr)` or an
    explicit `spec_mutex_locked_on_error` predicate in the PM contract.

### Low
- **Location:** `drop_guard_model` (exec) / `spec_guard_dropped_and_mutex_unlocked` (spec)
  - **Description:** The verification still omits the wakeup/notification behavior noted in the
    original code comment (“threads to be notified”). No predicate or lemma was added to capture
    waiter notification or liveness on unlock.
  - **Suggested Fix:** Add a predicate for waiter notification (or reference a proven mutex-module
    lemma) and include it in `drop_guard_model`’s postcondition.

## Positive Observations
- The exec/spec/proof split remains clean with explicit trust boundaries and clear documentation.
- Error propagation and guard-drop-on-success behavior are still modeled and proven.

## Summary
No substantive changes were made to address the prior gaps. The verification remains sound for the
control-flow skeleton but incomplete for error-path mutex state and unlock notification properties.
