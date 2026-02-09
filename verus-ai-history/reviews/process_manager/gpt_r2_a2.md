# Review: process_manager (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Capability semantics still unmodeled** (exec: `process_manager.rs` `capctl`, `capctl_error_noop`): The fix adds an error-path stub but still models successful `capctl` as a pure no-op with no abstract capability state. This is weaker than the real behavior (capability bits change), leaving a security-relevant operation unverified. **Suggested Fix:** Add an abstract capability predicate/state (ghost or spec) and specify the set/clear effect and its interaction with `has_capability`.
- **Thread-state branch selection still unconstrained** (exec: `process_manager.rs` `sleep_thread_running`, `exit_thread_running`, `exit_thread_to_suspended`, `exit_thread_to_zombie`, wrappers): The branch choice remains unconstrained by any predicate tying it to the actual thread state (runnable vs sleeping vs zombie), so the model can pick any transition. **Suggested Fix:** Introduce abstract thread-state predicates (e.g., `spec_has_runnable_threads(pid)` / `spec_all_sleeping(pid)` / `spec_all_zombie(pid)`) and require them on the corresponding transitions.

### Low
- **Outer post_message error path still not modeled** (exec: `process_manager.rs` `outer_post_message`): The new outer stub requires `spec_process_exists` and only models the success path; receiver-not-found and borrow-failure paths are still only documented, not represented as a distinct stub or disjunction. **Suggested Fix:** Add `outer_post_message_not_found` (and optionally `outer_post_message_borrow_fail`) or relax the precondition and specify disjunctive postconditions.
- **Scheduling order/fairness remains abstracted** (spec/exec): `Set<int>` plus unconstrained `chosen_next` still omits earliest-admission behavior. **Suggested Fix:** If fairness is a required property, model ready as `Seq<int>` or add an order predicate.

## Positive Observations
- The previously missing outer wrappers for `post_message` and `number_buffered_messages` are now present.
- `post_message` now requires `spec_process_exists` and has a `post_message_not_found` stub, addressing the earlier overly-strong increment behavior.
- No `assume` or `external_body` constructs were introduced, and verification still passes cleanly.

## Summary
Several earlier gaps were addressed, but key semantic coverage is still missing for capability updates and thread-state-dependent transitions. The verification remains sound for queue invariants, yet it does not fully capture security-critical capability behavior or the concrete branch selection logic for sleep/exit.
