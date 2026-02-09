# Review: process_manager (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Coverage still incomplete for several original functions.**  
  **Location:** `create_thread`, `try_add_thread`, `wakeup`, `try_wakeup`,
  `try_borrow`, `try_borrow_mut` in `src/kernel/src/pm/process/manager/mod.rs`; no
  direct verified counterparts in `verus/split/kernel/pm/process/manager/process_manager.rs`.
  **Description:** Dispatch helpers were added, but the original functions themselves
  still have no verified versions, and error paths for thread creation/wakeup remain
  unmodeled. This fails the “all functions have verified versions” coverage criteria.  
  **Suggested Fix:** Add explicit stubs matching each missing function (including
  error/no-op paths) or prove a clear, documented equivalence mapping that the
  dispatch helpers fully cover each original function’s control flow.

- **Exit-thread and iteration wrappers remain too weak to establish equivalence.**  
  **Location:** `exit_thread_dispatch`, `check_alarm_wrapper`, `harvest_zombies_wrapper`
  in `process_manager.rs` (exec).  
  **Description:** `exit_thread_dispatch` does not specify the queue transitions for
  each branch, and the iteration wrappers only preserve `wf()` without relating
  old/new queue contents. This leaves major behavior of `exit_thread`, `check_alarm`,
  and `harvest_zombies` underspecified.  
  **Suggested Fix:** Strengthen `exit_thread_dispatch` with branch-specific postconditions
  (matching `exit_thread_running`/`_to_suspended`/`_to_zombie`) and add a summary
  postcondition for the iteration wrappers (e.g., existence of a subset of moved PIDs).

### Medium
- **Interrupt reason behavior still not modeled.**  
  **Location:** `interrupt_reason` in `mod.rs`; `take_interrupt_reason` in
  `process_manager.rs` (exec).  
  **Description:** The verified model omits the interrupt reason state entirely, so
  the “returns and clears” behavior is unverified.  
  **Suggested Fix:** Introduce a ghost interrupt-reason field and specify that
  `take_interrupt_reason` returns the prior value and clears it.

- **Capability updates remain unverified.**  
  **Location:** `capctl`/`capctl_error_noop` in `process_manager.rs` (exec).  
  **Description:** Capability state is still treated as a no-op, so the spec ignores
  security-relevant state changes and their error conditions.  
  **Suggested Fix:** Model capabilities as a ghost map keyed by PID and update it in
  `capctl`, with error preconditions matching the original logic.

- **Scheduler ordering/fairness not captured.**  
  **Location:** `schedule`/`full_schedule` in `process_manager.rs` (exec).  
  **Description:** Ready queue ordering is still abstracted as a set with free
  `chosen_next`, so FIFO/fairness properties of `LinkedList` are not verified.  
  **Suggested Fix:** Use `Seq<int>` (or timestamps) to model admission order if
  fairness is a required property.

- **Message accounting still only tracked as a global counter.**  
  **Location:** `post_message`/`recv_message` in `process_manager.rs` (exec).  
  **Description:** The model does not relate the counter to per-process mailboxes,
  leaving IPC accounting correctness to trust boundary T4.  
  **Suggested Fix:** Add a ghost mailbox-size map and relate it to
  `number_buffered_messages`.

### Low
- None.

## Positive Observations
- New dispatch functions for `sleep` and `exit` add clearer, branch-aware
  postconditions and are a genuine improvement over the prior wrappers.
- Core queue invariants and transition proofs remain consistent, with no new
  `assume`/`external_body` usage detected.

## Summary
Improvements were made (notably the dispatch functions), but key coverage and
spec-strength gaps remain. The verification is still incomplete for several
original functions and for the iteration/exit-thread behavior, so the model is
not yet fully equivalent or complete.
