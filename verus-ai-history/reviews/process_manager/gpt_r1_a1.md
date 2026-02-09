# Review: process_manager (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** Trust boundary list (process_manager.spec.rs, T2/T3 sections) / missing exec models.
  **Description:** Coverage is incomplete: many functions in `src/kernel/src/pm/process/manager/mod.rs` have no verified counterparts (e.g., `create_thread`, `try_add_thread`, `set_thread_data_area`, `get_thread_data_area`, `sleep`, `wakeup`, `try_wakeup`, `exit`, `exit_thread`, `check_alarm`, `interrupt_reason`, `harvest_zombies`, `try_join_thread`, `get_mutex`, `get_cond`, `put_cond`, `put_mutex_guard`, `take_mutex_guard`, `take_earliest_ready`, `find_*`, and the entire outer `ProcessManager` API). The spec explicitly places these behind trust boundaries, which fails the "all functions verified" coverage requirement.
  **Suggested Fix:** Add verified wrappers/specs for the remaining functions (even if they only state no-queue-change), or split the module further so every original function has a modeled/verified counterpart. At minimum, provide verified stubs with pre/postconditions for all public APIs, including the outer `ProcessManager` methods.

- **Location:** `schedule` (process_manager.rs exec) vs `schedule`/`check_alarm` (mod.rs).
  **Description:** The verified `schedule` only swaps running/ready PIDs. In the original, `schedule` also runs `check_alarm()` (suspended→interrupted) and then resumes *all* interrupted processes back to ready before selecting the next runnable process. These state transitions are not modeled or composed, so the verified semantics diverge from the actual scheduler.
  **Suggested Fix:** Extend the verified model so `schedule` includes alarm and interrupt transitions (or verify a wrapper that sequences `alarm_interrupt` + `resume_all_interrupted` to match the real behavior). Also model the `interrupt_reason` update if it is considered observable state.

### Medium
- **Location:** Ready queue model (process_manager.spec.rs + `schedule` exec).
  **Description:** The ready queue is abstracted as a `Set<int>`, and `schedule` accepts an arbitrary `chosen_next` in `ready_with_running`. This loses FIFO/earliest-ready semantics (`take_earliest_ready`) and any fairness guarantees from the original linked-list ordering.
  **Suggested Fix:** Model the ready queue as a `Seq<int>` with ordering, or add a spec predicate capturing earliest-ready selection and prove schedule respects it.

- **Location:** Wakeup paths (process_manager.rs exec vs `wakeup`/`try_wakeup` in mod.rs).
  **Description:** The verified model only covers the suspended→ready transition (`wakeup_to_ready`) and assumes the wake succeeds. The original can target running/ready processes (no queue change), or fail to find the thread and return an error while preserving state. These cases are not modeled.
  **Suggested Fix:** Add verified functions for wakeup no-op and error paths (returning Result), or expand the model to cover all `wakeup`/`try_wakeup` outcomes with appropriate preconditions.

- **Location:** Termination path (process_manager.rs `terminate_ready_stays_ready`).
  **Description:** `terminate_ready_stays_ready` uses `&self` and proves only `wf()`; it does not model the internal thread state changes or the `terminate() → resume()` flow. This weakens the spec relative to the original termination logic for ready processes.
  **Suggested Fix:** Either (a) model thread-level effects explicitly, or (b) add a ghost state tracking per-process thread liveness and relate it to termination outcomes.

### Low
- **Location:** Message tracking invariants (process_manager.spec.rs `number_buffered_messages`).
  **Description:** The invariant only bounds the counter and does not relate it to actual per-process message queues, so correctness of message accounting is not captured.
  **Suggested Fix:** Introduce a ghost model of per-process message counts and relate it to `number_buffered_messages` if IPC correctness is in scope.

## Positive Observations
- The `wf()` invariant cleanly captures disjointness, PID bounds, kernel liveness, and count consistency.
- PID freshness and kernel-alive lemmas are explicitly stated and used.
- Spec/proof/exec are well separated, and there are no `assume`/`external_body` shortcuts.

## Summary
The verification provides a solid queue-level invariant model and proves key safety facts, but it abstracts away a large portion of the module and omits scheduler interrupt/alarm behavior. To meet the stated coverage and equivalence criteria, the unmodeled functions and the full scheduling semantics need verified counterparts or stronger wrappers.
