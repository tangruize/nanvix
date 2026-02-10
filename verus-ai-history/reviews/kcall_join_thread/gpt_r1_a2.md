# Review: kcall_join_thread (gpt-5.2-codex)

## Grade: A-

## Issues Found

### Critical
- _None._

### High
- _None._

### Medium
- **Location:** `process_manager_join_thread` external_body (exec)
- **Description:** The model still excludes `SleepError::Interrupted(TimedOut)` by assumption; there is no formal proof that `join_cond.wait(None)` cannot time out. This remains a soundness gap if PM behavior changes or `wait(None)` can still return `TimedOut`.
- **Suggested Fix:** Either extend the model with a `TimedOut` variant and propagate it, or import/prove a lemma from the PM/condvar model that `wait(None)` cannot return `TimedOut`.

### Low
- _None._

## Positive Observations
- The success-path memory write is now captured: `join_thread_model` ensures `spec_user_mem_written` with the join exit status via `spec_join_exit_status`.
- Safety preconditions now include `spec_no_resources_held`, and PM/MM predicates explicitly subsume synchronization.
- The TID validity axiom is documented as a deliberate trust boundary with a discharge reference.

## Summary
Most prior issues are fixed, and the verification now captures the key functional postcondition about writing the joined thread’s exit status to user memory. The remaining concern is the unproven TimedOut exclusion in T2, which is still a trust assumption and should be formalized if full soundness is desired.
