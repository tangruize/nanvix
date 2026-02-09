# Review: process_manager (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **`wakeup`/`try_wakeup` wrappers still model only the suspended-success path.**  
  **Location:** `inner_wakeup`, `inner_try_wakeup`, `inner_try_wakeup_noop` in
  `process_manager.rs` (exec).  
  **Description:** The new named stubs only allow the suspended→ready success case.
  The original functions can also succeed on running/ready processes, fail when the
  thread is not sleeping, or return not-found errors. Those cases are not captured
  in a single verified wrapper or in a branch-parameterized dispatch, so equivalence
  is still incomplete.  
  **Suggested Fix:** Add a branch-parameterized `inner_wakeup_dispatch` (or extend
  `inner_wakeup`) that allows all outcomes and connects to
  `wakeup_running_noop`/`wakeup_ready_noop`/`wakeup_suspended_failed_noop`/`wakeup_not_found`.

- **Iteration wrappers still too weak to capture behavior.**  
  **Location:** `check_alarm_wrapper`, `harvest_zombies_wrapper` in
  `process_manager.rs` (exec).  
  **Description:** These wrappers only preserve `wf()` without any relation between
  pre/post queue contents. This is too weak to claim equivalence to the original
  iteration logic.  
  **Suggested Fix:** Add postconditions describing the resulting sets (e.g., existence
  of subsets moved from suspended→interrupted or removed from zombie) or model the
  iteration as a parameterized batch transition over a ghost set of affected PIDs.

### Medium
- **Interrupt reason semantics still unmodeled.**  
  **Location:** `take_interrupt_reason` in `process_manager.rs` and corresponding
  spec/view in `process_manager.spec.rs`.  
  **Description:** The verified model still omits interrupt-reason state and does not
  specify the “returns and clears” behavior.  
  **Suggested Fix:** Add a ghost interrupt-reason field and specify the clear/return
  semantics in `take_interrupt_reason`.

- **Capability updates remain unverified.**  
  **Location:** `capctl`/`capctl_error_noop` in `process_manager.rs` (exec).  
  **Description:** Capability state is still treated as a no-op, so security-relevant
  state changes and error conditions are not verified.  
  **Suggested Fix:** Model capabilities via a ghost map from PID to capability sets and
  update it in `capctl` with error-path preconditions.

- **Scheduler ordering/fairness still abstracted away.**  
  **Location:** `schedule`/`full_schedule` in `process_manager.rs` (exec).  
  **Description:** The ready queue is still modeled as a set, so FIFO/fairness
  properties of the original `LinkedList` are not captured.  
  **Suggested Fix:** Use an ordered model (`Seq<int>` or timestamps) if fairness is a
  required property.

- **Message accounting still only tracked as a global counter.**  
  **Location:** `post_message`/`recv_message` in `process_manager.rs` (exec).  
  **Description:** There is no modeled relation between per-process mailboxes and the
  global counter, leaving IPC accounting correctness to T4.  
  **Suggested Fix:** Add a ghost mailbox-size map and relate it to
  `number_buffered_messages`.

### Low
- None.

## Positive Observations
- The new named stubs (`inner_create_thread`, `inner_try_add_thread`, `outer_try_borrow`)
  close most of the earlier coverage gaps for missing function names.
- `exit_thread_dispatch` now includes branch-specific postconditions, which is a
  meaningful improvement in equivalence strength.

## Summary
The update improves coverage and strengthens `exit_thread` semantics, but key
behavioral gaps remain (notably wakeup control flow and iteration semantics), and
several core state aspects are still unmodeled. Verification is not yet complete
or fully equivalent to the original implementation.
