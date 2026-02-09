# Review: process_manager (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** Outer ProcessManager API coverage (process_manager.rs exec).
  **Description:** Several original public methods still lack verified counterparts: `ProcessManager::create_process`, `create_thread`, `set_thread_data_area`, `get_thread_data_area`, `capctl`, `terminate`, `post_message`, `number_buffered_messages`, `handle_fpu_exception`, `harvest_zombies`, and `try_borrow`/`try_borrow_mut`. The new `outer_*` stubs cover only a subset of the outer API, so coverage remains incomplete.
  **Suggested Fix:** Add `outer_*` wrappers for all remaining public methods, including result/error-path behavior (even if modeled as no-ops with preconditions), or explicitly map them to existing inner verified functions.

### Medium
- **Location:** Wrapper stubs (`sleep_wrapper`, `exit_wrapper`, `exit_thread_wrapper`, `check_alarm_wrapper`, `harvest_zombies_wrapper`) in process_manager.rs.
  **Description:** These wrappers only ensure `wf()` and do not constrain post-state, which is too weak to capture the real queue transitions of `sleep`, `exit`, `exit_thread`, `check_alarm`, and `harvest_zombies`.
  **Suggested Fix:** Strengthen these wrappers to specify that the post-state equals one of the corresponding verified transitions (e.g., disjunction of `sleep_running`/`sleep_thread_running` outcomes).

- **Location:** `forge_user_context` stub (process_manager.rs).
  **Description:** The precondition requires the PID to already be in the ready queue, but the original `forge_user_context` is called *before* a new process is inserted into ready in `create_process`. This makes the spec too strong and mismatched with the original call order.
  **Suggested Fix:** Remove the `ghost_ready` requirement or replace it with a weaker precondition (e.g., PID is fresh / equals `next_pid`) that matches the original usage.

- **Location:** `create_thread` / `try_add_thread` modeling (process_manager.rs).
  **Description:** The added `create_thread_in_ready` and `create_thread_from_suspended` stubs cover queue effects, but error paths (running/interrupt/zombie process and thread-not-found) are not modeled, and there is no top-level `create_thread`/`try_add_thread` spec tying these cases together.
  **Suggested Fix:** Add a `create_thread_wrapper` spec with pre/postconditions covering all success/error outcomes and mapping to the relevant queue transitions.

- **Location:** `terminate_ready_stays_ready` (process_manager.rs).
  **Description:** Still a no-op stub with `&self`; it does not model the thread-level termination effects that justify the `terminate() → resume()` flow, so the spec remains too weak for equivalence.
  **Suggested Fix:** Add ghost state for per-process thread liveness or specify the required thread-level postconditions, even if queue membership is unchanged.

### Low
- **Location:** Ready queue ordering (process_manager.spec.rs).
  **Description:** The model still uses `Set<int>`, so FIFO/earliest-ready selection and fairness properties from `take_earliest_ready` are not captured.
  **Suggested Fix:** Use `Seq<int>` or add predicates enforcing earliest-ready selection if liveness/fairness is in scope.

- **Location:** IPC accounting (process_manager.spec.rs `number_buffered_messages`).
  **Description:** The counter is still not linked to per-process message queues, so IPC accounting correctness is not verified.
  **Suggested Fix:** Add a ghost model for per-process message counts and relate it to `number_buffered_messages`.

## Positive Observations
- Coverage improved with new stubs (wakeup error path, create_thread_from_suspended, helper/outer wrapper stubs).
- `full_schedule` now specifies the ready set and count postconditions, improving equivalence with the original scheduler.
- No `assume`/`external_body` shortcuts were introduced.

## Summary
The update fixes several prior gaps (full_schedule postconditions and wakeup error path), but coverage is still incomplete for the outer API and several wrapper specs remain too weak to reflect actual behavior. Addressing the remaining wrapper constraints and missing outer methods is necessary to meet the stated coverage and equivalence criteria.
