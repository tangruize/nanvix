# Review: process_manager (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- **Coverage gap for outer wrappers** (exec: `process_manager.rs`): The original `ProcessManager::post_message` and `ProcessManager::number_buffered_messages` wrappers (and the private `try_borrow`/`try_borrow_mut`) are not modeled with `outer_*` stubs, so the RefCell borrow-error path (`ResourceBusy`) and outer-level result behavior are unverified for these APIs. **Suggested Fix:** Add `outer_post_message` and `outer_number_buffered_messages` stubs (and optionally `outer_try_borrow`/`outer_try_borrow_mut` or a documented lemma) that explicitly model the T2 borrow boundary and map to the inner transition or no-op as appropriate.

### Medium
- **`post_message` spec is too weak** (exec: `process_manager.rs` `post_message`): The model only requires a buffer bound and always increments `number_buffered_messages`, but the original returns `Err` when the receiver PID/TID does not exist and must not increment the counter on failure. **Suggested Fix:** Add a precondition requiring `spec_process_exists` for the resolved receiver (or split into success/failure stubs like `post_message_ok`/`post_message_err`), and explicitly model the no-op/error path.
- **Capability control semantics omitted** (exec: `process_manager.rs` `capctl`): The spec models `capctl` as a pure no-op that only preserves `wf`, omitting capability updates and error cases (capability already set / not set). This is weaker than the intended behavior. **Suggested Fix:** Add ghost capability state (or an abstract predicate) and specify the set/clear effect and corresponding error preconditions.
- **Thread-state branch conditions not captured** (exec: `process_manager.rs` `sleep_thread_running`, `exit_thread_*`, wrappers): The model allows any branch to be chosen without preconditions linking to actual thread state (runnable vs sleeping vs zombie). This weakens equivalence for `sleep`/`exit`/`exit_thread`. **Suggested Fix:** Introduce abstract predicates about per-process thread state (e.g., `spec_has_runnable_threads(pid)`) and use them as preconditions to select the correct transition.

### Low
- **Scheduling order/fairness abstracted away** (spec: `process_manager.spec.rs` / exec: `schedule`): The ready queue is modeled as a `Set<int>` and `chosen_next` is arbitrary, so FIFO/earliest-admission semantics and fairness are not captured. **Suggested Fix:** If fairness is a required property, replace `Set` with `Seq` (or add an order predicate) and specify selection of the earliest element.

## Positive Observations
- Strong core invariants (`wf`) cover disjointness, kernel safety, PID bounds, and overflow safety, and are preserved by all state-changing transitions.
- Trust boundaries (scheduler choice, RefCell borrow, thread-level effects) are explicitly documented, which clarifies what is and is not verified.
- Proofs show PID freshness, kernel liveness, and count preservation, and the spec/proof/exec split is clean and easy to audit.
- No `assume`/`external_body` found in the core module; verification passes cleanly.

## Summary
The verification captures the main process-queue safety properties and preserves key invariants, but several outer API wrappers and operation-specific semantics are either missing or under-specified. Strengthening `post_message`, `capctl`, and thread-level branch conditions, and adding missing outer stubs would significantly improve equivalence and coverage without changing the core invariants.
