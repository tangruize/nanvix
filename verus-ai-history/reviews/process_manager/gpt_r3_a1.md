# Review: process_manager (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Missing verified stubs for several original functions (coverage gap).**  
  **Location:** `create_thread`, `try_add_thread`, `wakeup`, `try_wakeup`, `try_borrow`,
  `try_borrow_mut` in `mod.rs` (no corresponding exec stubs in
  `verus/split/kernel/pm/process/manager/process_manager.rs`).  
  **Description:** The verified module does not provide exec/spec stubs for these
  original functions, so the coverage requirement (“all functions, public and private”)
  is not met. The current model only covers fragments (`create_thread_in_ready`,
  `create_thread_from_suspended`, and wakeup no-op variants), which does not account for
  the full control flow and error paths.  
  **Suggested Fix:** Add wrapper specs for each missing function that capture their
  success/error paths and queue-level effects (or explicitly mark them as trusted
  boundary functions with a clear rationale and audit trail).

- **Wrapper specs are too weak to be equivalent to the original API behavior.**  
  **Location:** `sleep_wrapper`, `exit_wrapper`, `exit_thread_wrapper`,
  `check_alarm_wrapper`, `harvest_zombies_wrapper` in
  `process_manager.rs` (exec).  
  **Description:** These wrappers only preserve `wf()` and do not specify the possible
  queue transitions or postconditions. This makes the specifications for the original
  `sleep`, `exit`, `exit_thread`, `check_alarm`, and `harvest_zombies` effectively
  no-ops, which is too weak for equivalence and correctness claims.  
  **Suggested Fix:** Strengthen the wrappers with disjunctive postconditions that
  connect them to the verified transition functions (`sleep_running`,
  `sleep_thread_running`, `exit_*`, `alarm_interrupt`, `harvest_zombie`), or model the
  full transitions directly in the wrapper bodies.

### Medium
- **Interrupt reason semantics are not modeled.**  
  **Location:** `interrupt_reason` in `mod.rs`; `take_interrupt_reason` in
  `process_manager.rs` (exec).  
  **Description:** The original clears `interrupt_reason`, but the verified version
  ignores this state entirely. This drops observable behavior and could allow clients
  to assume stronger properties than the implementation provides.  
  **Suggested Fix:** Add a ghost field for interrupt reason in the view/invariant and
  specify that `take_interrupt_reason` returns the prior value and clears it.

- **Scheduler ordering/fairness is abstracted away.**  
  **Location:** `take_earliest_ready`/`schedule`/`full_schedule` in
  `process_manager.rs` (exec); FIFO semantics in `mod.rs`.  
  **Description:** The verified model uses `Set<int>` and a free `chosen_next`
  parameter, which loses FIFO ordering and admission-time fairness properties present
  in the original `LinkedList`-based scheduler. This weakens liveness/fairness claims.  
  **Suggested Fix:** If fairness is a required property, model the ready queue as
  `Seq<int>` (or a multiset with timestamps) and specify selection of the earliest
  element.

- **Capability state changes are unverified.**  
  **Location:** `capctl` in `mod.rs` vs. `capctl`/`capctl_error_noop` in exec.  
  **Description:** The verified model treats capability updates as no-ops, so it
  does not verify security-relevant state changes or error conditions (already set /
  not set).  
  **Suggested Fix:** Add a ghost capability map keyed by PID and update it in the
  `capctl` spec, with error-path specs mirroring the original conditions.

### Low
- **Message counter tracking lacks a concrete linkage to per-process queues.**  
  **Location:** `post_message`/`recv_message` in `process_manager.rs` (exec).  
  **Description:** The model only bounds the counter and updates it, but does not
  connect it to actual per-process message queues; correctness depends on trust
  boundary T4.  
  **Suggested Fix:** If message accounting is critical, add a ghost model of per-PID
  mailbox sizes and relate it to `number_buffered_messages`.

## Positive Observations
- Strong queue invariants (`wf`) capture disjointness, kernel safety, PID bounds, and
  count consistency.
- Core queue transitions (create/schedule/sleep/exit/terminate/wakeup/harvest) are
  specified and proven to preserve invariants.
- No `assume`/`external_body` usage detected in the core module; proof/exec/spec split
  is clean and well documented.

## Summary
The verification provides a solid queue-level safety model, but coverage and spec
strength are insufficient for several original functions and wrappers. Closing the
gaps in wrapper semantics and adding explicit stubs for missing functions would
substantially improve equivalence and completeness.
