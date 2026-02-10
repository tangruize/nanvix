# Review: process_manager_unsafe (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

- None.

### High

- **Location:** `exit()` and `exit_thread()` (exec file, lines 524-597)
  **Description:** The original `exit()` and `exit_thread()` return `Result<!, Error>` — they diverge on the success path (never return to the caller). The verified model captures the state transition but the functions return normally, allowing callers to reason about post-exit state as if the exiting process continues. While documented as T10, this means the postcondition `self.current_pid == chosen_next_pid` is provable but misleading: it describes the *new* process's view, yet nothing prevents a caller from chaining code after `exit()` in the verified model. A `requires false`-guarded continuation or a comment-level caller obligation is not machine-checked.
  **Suggested Fix:** Add a ghost `ensures` that marks the post-state as "exited" (e.g., a boolean flag `self.exited == true`) so that any subsequent operation's `requires !self.exited` would fail, providing a machine-checked divergence boundary. Alternatively, document explicitly that integration-level harnesses must not call further methods after exit/exit_thread.

- **Location:** `exit()` postconditions (exec file, line 545)
  **Description:** `exit()` ensures `self.current_pid == chosen_next_pid` unconditionally. However, `switch()` only updates `current_pid` when `next_pid != old(self).current_pid` AND `next_tid != old(self).current_tid`. If both the PID and TID happen to stay the same (soft switch — theoretically impossible for exit but not excluded by preconditions), the postcondition would need to hold via the unchanged path. The ensures is correct only because exit always changes the running process (the exiting process leaves), but this reasoning is implicit — the precondition `old(self).current_pid != KERNEL_PID_RAW` is necessary but not sufficient to guarantee a hard switch. The actual invariant (exit always produces a different next_tid) relies on the inner module's exit semantics, which are trusted via `new_inner`.
  **Suggested Fix:** Add an explicit precondition `chosen_next_tid != old(self).current_tid` to `exit()` and `exit_thread()`, making the hard-switch assumption explicit and machine-checked. This matches the real semantics: exit always switches away from the current thread.

### Medium

- **Location:** Delegation functions `get_mutex`, `put_mutex_guard`, `get_cond`, `put_cond`, `take_mutex_guard` (exec file, lines 616-701)
  **Description:** These are modeled as pure identity functions returning a `succeeds` boolean parameter. They take `&self` (immutable) rather than `&mut self`, which means they cannot model any state changes that the inner ProcessManagerInner might perform. While the documentation says "no queue-level state change," the original `get_mutex()` may create a new mutex in the inner table (a mutation), and `put_cond()` removes a condvar. The verification proves nothing about the actual behavior of these functions beyond wf() preservation.
  **Suggested Fix:** Either (a) change these to `&mut self` and accept a `new_inner: ProcessManagerInner` parameter (similar to `wakeup`) to model potential inner mutations, or (b) add explicit trust boundary annotations (T12 already exists) and accept the current abstraction. The current approach is acceptable given that sync object correctness is deferred to the inner module, but it's worth noting.

- **Location:** `join_thread_harvest()` (exec file, line 767)
  **Description:** The original `join_thread()` (unsafe.rs:341-408) contains significant logic: it calls `try_join_thread`, harvests zombie threads by unmapping user stack pages via `VirtMemoryManager::get_mut().unmap_upage()`, and iterates over pages. The verified model is a no-op (`join_thread_harvest` has an empty body). While memory management is correctly identified as external to the queue model, the loop structure and error handling (the `warn!` on failed unmap) represent non-trivial control flow that is not modeled at all.
  **Suggested Fix:** Consider adding a ghost parameter `harvested_pages: nat` to model the number of pages processed, with a postcondition that it equals `(top - base) / PAGE_SIZE`. This would verify the loop bounds without modeling the actual memory operations.

- **Location:** `sleep()` spec vs original (exec file, lines 454-476 vs unsafe.rs:438-469)
  **Description:** The original `sleep()` takes an `alarm: Option<SystemTime>` parameter that determines whether the sleep is timed. The verified model's `sleep()` does not model the alarm parameter at all — it is completely abstracted away. The alarm affects whether the thread is placed on a timed-sleep queue vs. indefinite-sleep queue, which could have different wakeup semantics.
  **Suggested Fix:** Add a ghost `alarm: Option<int>` parameter to `sleep()` and propagate it as a precondition on `new_inner`, requiring that timed vs. untimed sleep is correctly reflected in the inner state.

- **Location:** `giveup()` branching condition (exec file, line 437)
  **Description:** The original checks `remaining_ticks > 1` (unsafe.rs:497), and the model checks `self.remaining_quantum > 1` (line 437). This matches. However, the original reads `remaining_ticks` from an atomic load and then stores `remaining_ticks - 1`, while the model operates directly on the struct field. The atomicity gap (between load and store in the original) is not a real issue on single-core, but the model does not capture that the no-switch path returns `Ok(())` — it is a void function. The original returns `Result<(), Error>`, and the giveup model has no error-path modeling. The `try_borrow_mut()?` call in the switch path can fail with `ResourceBusy`, which is not modeled.
  **Suggested Fix:** Add an error path variant `giveup_error()` similar to `join_thread_error()` to model the `try_borrow_mut()` failure case, or document it as part of the T2 (RefCell borrow) trust boundary.

### Low

- **Location:** `switch()` — `user_tda` parameter (exec file, line 296 vs unsafe.rs:763)
  **Description:** The original `switch()` takes 5 parameters including `user_tda: Option<VirtualAddress>` which is passed to `ContextInformation::switch(from, to, user_tda)`. The verified model's `switch()` takes only 3 parameters (new_inner, next_pid, next_tid), omitting `from`, `to`, and `user_tda`. The raw pointer parameters are correctly abstracted away (T5), but `user_tda` represents the user-space thread data area and could affect the correctness of the address space setup for the next thread. This is acceptable as a trust boundary but worth noting.
  **Suggested Fix:** No change needed; the T5 trust boundary correctly covers this. Consider adding a brief note in the T5 documentation mentioning `user_tda`.

- **Location:** `try_recv_some()` — inner struct update (exec file, lines 740-743)
  **Description:** The implementation creates a new `ProcessManagerInner` using struct update syntax `..self.inner`. This copies all fields except `number_buffered_messages`. If `ProcessManagerInner` gains new fields in the future, this would silently copy them. The pattern is correct today but fragile under evolution.
  **Suggested Fix:** No immediate change needed; this is a standard Verus pattern. The existing proof (`lemma_recv_message_preserves_wf`) would catch any issues if new fields affected wf().

- **Location:** Spec file — `spec_tid_valid` (spec file, line 117-119)
  **Description:** The spec only requires `current_tid >= 0i32`, but does not bound it above. In the original, TIDs are `AtomicI32` values derived from `ThreadIdentifier`, which presumably has a bounded range. An unbounded TID could theoretically overflow if used in arithmetic elsewhere. However, since TIDs are only compared (not used arithmetically) in this module, this is low risk.
  **Suggested Fix:** Consider adding an upper bound like `current_tid < i32::MAX` for defense in depth.

- **Location:** Performance counter modeling (exec file, not present)
  **Description:** The original code increments several performance counters (`PERF_SCHED_EXIT_CONTEXT_SWITCHES`, `PERF_SCHED_HARD_CONTEXT_SWITCHES`, etc.) via `fetch_add(1, ORDER)`. These are completely omitted from the verified model. While performance counters don't affect correctness, their atomic operations do have ordering effects that could theoretically matter in a multi-core setting.
  **Suggested Fix:** No change needed for single-core Nanvix. Document as intentionally omitted.

## Positive Observations

- **Zero assumes/external_body:** The module has no `assume` or `external_body` annotations. All 32 verified functions pass cleanly. This is excellent for a module dealing with unsafe global state.

- **Excellent trust boundary documentation:** T5 through T13 are clearly enumerated with precise descriptions of what is trusted, why, and what would be needed to remove the trust boundary. The T10 (divergence) documentation is particularly thorough.

- **Faithful `switch()` modeling:** The verified `switch()` correctly captures the subtle semantics of the original: inner state is mutated *before* switch is called, but switch reads the *old* PID/TID from atomics. The comparison `next_pid != self.current_pid` at line 341 (where `self.current_pid` is still the old value because `self.inner` was just updated but atomics weren't) precisely matches the original `CURRENT_PID.load()` behavior.

- **Complete function coverage:** All 17 original functions have verified counterparts. Complex functions (join_thread, try_recv) are split into multiple verified variants covering different execution paths.

- **Well-structured spec/proof/exec separation:** The spec file contains pure specifications and View types, the proof file contains lemmas, and the exec file contains executable verified code. No spec leaks into exec or vice versa.

- **Strong invariant preservation:** Every state-mutating function proves `self.wf()` in its postcondition, ensuring the global invariant is maintained across all transitions.

- **Quantum management correctness:** The proof that quantum stays in `[1, scheduler_freq]` is tight — the lower bound of 1 (not 0) is correctly justified by the branching condition `remaining_quantum > 1`.

## Summary

This is a high-quality verification of a complex unsafe kernel module. The verification correctly models the global singleton pattern, atomic state management, and context switch semantics including the subtle stale-atomic PID comparison. All 32 functions verify with zero errors, zero assumes, and zero external_body annotations.

The main gap is the lack of machine-checked divergence for `exit()`/`exit_thread()` (T10), which is a known Verus limitation. The unconditional `self.current_pid == chosen_next_pid` postcondition on `exit()` implicitly relies on the hard-switch guarantee that could be made explicit. Delegation functions are thin abstractions that correctly defer to the inner module but don't capture mutation semantics.

The trust boundaries are well-documented and appropriately scoped. The module verifies everything it owns (atomics, quantum, singleton lifecycle) and cleanly delegates cross-cutting concerns (thread membership, sync objects, memory management) to other modules. Recommended improvements are minor: explicit hard-switch preconditions on exit functions, and optional alarm/error-path modeling for sleep/giveup.
