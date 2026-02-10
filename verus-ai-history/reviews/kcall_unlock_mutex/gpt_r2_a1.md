# Review: kcall_unlock_mutex (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `take_mutex_guard_model` (exec) / `spec_guard_dropped_and_mutex_unlocked` (spec)
  - **Description:** The error-path mutex state is underspecified: the model only states that when
    `pm_internally_dropped_guard` is true the mutex is unlocked, but it never asserts the converse
    (that when the flag is false the mutex remains locked). This leaves the spec too weak to rule
    out unintended unlocks on error paths and does not let clients reason about error side effects.
  - **Suggested Fix:** Strengthen the PM contract with a predicate for the “mutex remains locked”
    case (or an explicit `spec_mutex_unlocked_on_error` output) and assert
    `!pm_internally_dropped_guard ==> !spec_guard_dropped_and_mutex_unlocked(mutex_addr)`.

### Low
- **Location:** `drop_guard_model` (exec) / `spec_guard_dropped_and_mutex_unlocked` (spec)
  - **Description:** The original code comment notes that dropping the guard notifies waiting threads,
    but the verification does not specify or prove any wakeup/notification (liveness) property.
    This leaves an essential behavioral aspect of mutex unlocks unverified at the kcall boundary.
  - **Suggested Fix:** Add a spec predicate for waiter notification (or link to a verified mutex
    module lemma) and include it in the postcondition of `drop_guard_model`.

## Positive Observations
- Full functional coverage: the only function in the source file (`unlock_mutex`) is modeled and
  verified.
- The exec/spec/proof split is clean, with explicit trust boundaries and documented assumptions.
- Error propagation and the guard-drop-on-success behavior are explicitly modeled and proven.

## Summary
The verification captures the control-flow and error-propagation behavior well, but it is still
underspecified for error-path mutex state and does not cover wakeup/notification semantics.
Strengthening the PM error-path contract and adding a waiter-notification predicate would bring
this closer to a complete correctness story for the kcall.
