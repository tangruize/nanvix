# Review: process_manager (claude-opus-4.6)

## Grade: A-

## Verification Result

78 verified, 0 errors. No `assume`, `external_body`, or `trusted` annotations found.

## Issues Found

### Critical

- None.

### High

- **No PID exhaustion recovery model.**
  - Location: `create_process` (exec), `spec_pid_bounds` (spec)
  - Description: The `create_process` precondition requires `next_pid < i32::MAX` but there is no model for what happens when PID space is exhausted. In a long-running kernel, `next_pid` monotonically increases (PIDs are never recycled), so eventually `create_process` becomes uncallable. The original code uses `ProcessIdentifier::from(i32::from(pid) + 1)` which would overflow. The verification correctly guards against this with the precondition, but the lack of PID recycling means the system has a hard upper bound of ~2 billion processes over its lifetime.
  - Suggested Fix: Document this as an explicit design limitation in the spec, or model a PID recycling scheme (e.g., reusing PIDs from harvested zombies) with an additional invariant that recycled PIDs are not in any queue. At minimum, add a spec function `spec_can_create_process` that callers can check.

### Medium

- **Thread-level queue transitions are unverified (T3 boundary is broad).**
  - Location: `create_thread_in_ready`, `exit_thread_running`, `exit_thread_to_suspended`, `exit_thread_to_zombie` (exec)
  - Description: The decision of which branch is taken in `exit()` (lines 918-928), `exit_thread()` (lines 1001-1017), and `sleep()` (lines 743-754) depends on thread-level state (whether remaining threads are runnable, sleeping, or zombie). This logic is entirely within trust boundary T3. A bug in the thread-level branching (e.g., process incorrectly routed to zombie instead of ready) would violate the state machine, but the verification cannot catch it because the branch choice is a parameter.
  - Suggested Fix: Consider adding a lightweight thread-count model (e.g., `runnable_thread_count: nat` per process) to verify that the branch preconditions are mutually exclusive and exhaustive. This would narrow T3 without requiring full thread verification.

- **Queue ordering abstraction loses scheduling fairness.**
  - Location: `spec_ready_with_running`, `spec_full_schedule_pool` (spec), `schedule`, `full_schedule` (exec)
  - Description: The original `take_earliest_ready` (mod.rs:1352-1372) selects the process with the earliest admission time (FIFO-like). The verified model uses `Set<int>` which is unordered, so `chosen_next` is an unconstrained parameter. This means the verification cannot prove starvation-freedom or fairness properties. The spec file (lines 97-107) acknowledges this.
  - Suggested Fix: If scheduling fairness is a future verification goal, replace `ghost_ready: Ghost<Set<int>>` with `ghost_ready: Ghost<Seq<int>>` and model `take_earliest_ready` as selecting from the sequence with appropriate ordering constraints.

- **`wakeup` failed-wakeup path is not explicitly modeled.**
  - Location: `try_wakeup` (original mod.rs:819-875)
  - Description: The original `try_wakeup` can find a thread in a suspended process but fail to wake it (the `Err(suspended_process)` branch at line 832). In this case, the process stays suspended and the function returns `None`. This path is implicitly covered by "no state change on error" but is not explicitly modeled as a distinct verified transition. The `wakeup_not_found` stub covers the "not found" case but does not distinguish between "found but failed" and "not found at all".
  - Suggested Fix: Add a `wakeup_suspended_failed_noop` stub that documents this specific error path, similar to `wakeup_running_noop` and `wakeup_ready_noop`.

- **`exit` process-to-ready path shares code with `exit_thread_running` without explicit equivalence lemma.**
  - Location: `exit_running` (exec), `exit_thread_running` (exec)
  - Description: The original `exit()` function (mod.rs:918-922) has an Ok path where the process still has runnable threads and goes to ready. This is modeled by reusing `exit_thread_running`. While the queue-level behavior is identical, the original `exit()` terminates the process's running thread while `exit_thread()` terminates a specific thread. The reuse is semantically correct but a comment or lemma explicitly stating the equivalence would strengthen confidence.
  - Suggested Fix: Add a comment in `exit_thread_running` noting it models both the Ok path of `exit()` (process-level) and the Ok path of `exit_thread()` (thread-level). The spec file (lines 32-34) partially does this but could be more explicit.

### Low

- **Many outer wrapper stubs provide trivial verification value.**
  - Location: `outer_vmcopy_from_user`, `outer_vmcopy_to_user`, `outer_mmap`, `outer_munmap`, `outer_mctrl`, `outer_mmio_alloc`, `outer_mmio_free`, `outer_attach_pmio`, `outer_detach_pmio`, `outer_read_pmio`, `outer_write_pmio`, `outer_add_event`, `outer_remove_event` (exec)
  - Description: These 13 outer wrapper stubs take `&self` and simply assert `wf() → wf()`, which is trivially true. They don't model any state changes, parameters, or return values. While they serve as documentation that the original outer methods delegate without queue changes, the verification value is minimal.
  - Suggested Fix: Consider grouping these under a single documentation section rather than individual stubs, or add meaningful preconditions (e.g., `outer_mmap` could require `spec_process_exists(pid)` to model the `find_process_mut` call).

- **`lemma_view_equality` is weak.**
  - Location: `lemma_view_equality` (proof)
  - Description: The lemma only proves that equal views imply equal `spec_running_pid()`. This is trivially true from the view definition. It does not prove the converse or any deeper structural equality. It appears unused.
  - Suggested Fix: Either remove the lemma or strengthen it to prove that equal views imply `wf()` equivalence, or document its intended future use.

- **`terminate` for running process is an error path but not explicitly modeled.**
  - Location: `terminate` (original mod.rs:1044-1049)
  - Description: The original `terminate` returns an error if the target PID is the running process. The verified model only has `terminate_ready` and `terminate_suspended`, which match the success paths. The error path (running process cannot be terminated) is a precondition violation. This is fine for the verification model, but documenting this explicitly in the spec would be helpful.
  - Suggested Fix: Add a brief note in the spec file's trust boundary documentation that `terminate` for a running process is an error path that preserves state trivially.

- **`spec_counts_bounded` uses a sum inequality but doesn't prove uniqueness across all queues.**
  - Location: `spec_counts_bounded` (spec)
  - Description: The invariant `ready + suspended + interrupted + zombie + 1 <= next_pid` bounds the total but relies on disjointness to guarantee uniqueness. The connection between disjointness, finiteness, and this bound is not proven as a standalone lemma (it's implied by the combined `wf()` invariant).
  - Suggested Fix: Add a lemma proving that under `wf()`, the total process count equals the sum of queue sizes plus one (for running), and this equals the number of distinct PIDs.

## Positive Observations

- **Excellent documentation of trust boundaries.** The spec file (lines 22-119) meticulously documents three trust boundaries (T1: scheduler choice, T2: RefCell, T3: thread-level ops) and maps every original function to its verified counterpart. This is exemplary for a verification project.
- **No unsound shortcuts.** Zero `assume`, `external_body`, or `trusted` annotations. All 78 proof obligations are discharged by the Verus solver. This is the gold standard for verified code.
- **Strong well-formedness invariant.** The `wf()` predicate composes seven sub-invariants covering finiteness, count matching, disjointness, running exclusivity, kernel safety, PID bounds, and overflow prevention. Every state transition function preserves `wf()` both as a precondition and postcondition.
- **Faithful decomposition of branching transitions.** Complex original functions like `exit_thread()` (three branches) and `sleep()` (two branches) are correctly decomposed into separate verified functions with matching queue-level semantics.
- **Kernel liveness is proven across all transitions.** The `spec_kernel_safe` invariant ensures PID 0 is always either running or ready, and the `lemma_kernel_alive_after_schedule` proof handles all cases. This is the most important safety property for an OS kernel.
- **Clean three-file split.** Spec (316 lines), proof (222 lines), and exec (1461 lines) are well-separated. The spec is readable as an independent specification document.
- **Compositional verification.** The `full_schedule` function composes `resume_all_interrupted` + `schedule`, matching the original's three-step scheduling logic. The composition is proven correct with proper cardinality reasoning.

## Summary

This is a high-quality verification of the Nanvix process manager's state machine. The core contribution is proving that all process lifecycle transitions (create, schedule, sleep, exit, terminate, harvest) preserve the well-formedness invariant, which encodes process partitioning, kernel liveness, PID uniqueness, and overflow safety. The verification covers 100% of original functions through a combination of verified state transitions (for queue-changing operations) and documented stubs (for thread-level and query operations). The three trust boundaries (scheduler choice, RefCell borrow, thread-level state) are clearly identified and well-justified. The main limitation is the thread-level trust boundary (T3), which means the branch decisions in `exit`/`exit_thread`/`sleep` are parameters rather than derived from verified thread state. Adding a lightweight thread-count model would be the highest-value improvement. The queue ordering abstraction (Set vs Seq) is acceptable for current goals but precludes future fairness proofs.
