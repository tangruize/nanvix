# Review: process_manager (claude-opus-4.6)

## Grade: B+

## Issues Found

### Critical

_None._

### High

- **Location**: `terminate_ready_to_interrupted` (exec, line 573)
  - **Description**: Semantic mismatch with original `terminate()` for ready processes. In the original (mod.rs:1051–1065), when a ready process with surviving threads is terminated, it goes through `process.terminate()` → `Ok(interrupted_process)` → `interrupted_process.resume()` → pushed back to `self.ready`. The net effect at the queue level is the process **stays in ready**. However, the verified `terminate_ready_to_interrupted` moves the process from ready to the interrupted queue, which is a state that the original never produces for this code path. This is a semantic equivalence gap — the verified model allows a transient state (process in interrupted queue after ready-terminate) that cannot occur in the original.
  - **Suggested Fix**: Replace `terminate_ready_to_interrupted` with a `terminate_ready_noop` function that models the Ok path as a no-op at the queue level (process remains in ready, internal thread state changes are abstracted away). This correctly captures that the process stays in the ready queue when it has surviving threads after termination.

### Medium

- **Location**: `exit_thread_to_suspended` and `exit_thread_to_zombie` (exec, lines 427–484)
  - **Description**: Missing `number_buffered_messages` frame postcondition. All other state-transition functions explicitly ensure `self.number_buffered_messages == old(self).number_buffered_messages`, but these two functions omit it. While `wf()` preservation implies the field is not corrupted, a caller composing proofs cannot directly establish that message count is preserved without the explicit postcondition.
  - **Suggested Fix**: Add `self.number_buffered_messages == old(self).number_buffered_messages` to the `ensures` clauses of both `exit_thread_to_suspended` and `exit_thread_to_zombie`.

- **Location**: Coverage gap — thread-level operations
  - **Description**: Approximately 15 functions from the original `ProcessManagerInner` are unverified, including `create_thread`, `try_add_thread`, `wakeup` (full version with search), `try_wakeup`, `exit_thread` (internal branching), `set_thread_data_area`, `get_thread_data_area`, `try_join_thread`, and all synchronization primitives (`get_mutex`, `get_cond`, `put_cond`, `put_mutex_guard`, `take_mutex_guard`). These are documented as trust boundary T3 (thread-level details), but they represent substantial logic that affects which queue a process lands in.
  - **Suggested Fix**: Document the trust boundary more precisely in the spec file. For the most critical functions (`wakeup`, `exit_thread`), consider adding at minimum a spec-level model that captures the branching outcomes (e.g., wakeup can move suspended→ready or be a no-op on running/ready processes).

- **Location**: Coverage gap — outer `ProcessManager` wrapper
  - **Description**: The outer `ProcessManager` (mod.rs:1529–1982) which wraps `ProcessManagerInner` in `Rc<RefCell<_>>` is entirely unverified. While documented as trust boundary T2, this wrapper includes ~30 public API methods that are the actual kernel interface. The `try_borrow`/`try_borrow_mut` pattern could fail, and several methods compose multiple inner operations (e.g., `harvest_zombies` does memory cleanup + thread harvesting + queue removal).
  - **Suggested Fix**: At minimum, add a note in the spec file enumerating which outer methods map to which verified inner functions, to aid manual audit.

- **Location**: Coverage gap — error paths
  - **Description**: Original functions return `Result<T, Error>` with multiple failure modes (process not found, kernel process rejection, running process rejection in `terminate`). The verified functions model only the success paths via preconditions (e.g., `terminate_ready` requires `ghost_ready@.contains(pid)`). While this is standard for verification (preconditions eliminate error cases), it means the error-handling logic is entirely unverified.
  - **Suggested Fix**: Consider adding verified error-returning versions of key functions (at least `terminate` and `create_process`) that return a Result type, proving that error paths don't corrupt state.

### Low

- **Location**: `recv_message` (exec, line 667)
  - **Description**: The verified model has a `recv_message` function that decrements `number_buffered_messages`, but the original `ProcessManagerInner` has no corresponding method. The decrement happens indirectly through process state methods. While the abstraction is reasonable, it models behavior that doesn't directly exist in the source type.
  - **Suggested Fix**: Add a comment documenting that `recv_message` models the implicit decrement that occurs when messages are consumed through `ProcessState` methods.

- **Location**: `spec_counts_bounded` (spec, line 128)
  - **Description**: The bound `number_buffered_messages < usize::MAX` is weaker than ideal. The original code only increments this counter in `post_message`, and messages are tied to processes. A tighter bound relating message count to total process count would be more informative, though the current bound is sufficient for overflow prevention.
  - **Suggested Fix**: Consider strengthening to `number_buffered_messages <= some_function_of(next_pid)` if a tighter relationship can be established, or document why the loose bound is acceptable.

- **Location**: Queue ordering not modeled
  - **Description**: The original uses `LinkedList` with `push_back`/`pop_front` (FIFO) and `take_earliest_ready` selects by earliest admission time. The verified model uses `Set<int>` which loses ordering information. This means scheduling fairness/ordering properties cannot be proven.
  - **Suggested Fix**: This is an acceptable abstraction for the current verification goals. If scheduling order matters in the future, consider using `Seq<int>` instead of `Set<int>` for the ready queue.

- **Location**: `interrupt_capable` immutability
  - **Description**: In the original code, `interrupt_capable` is set once at construction and never modified. The verified model doesn't enforce this as an invariant — it's implicitly preserved because no function modifies it, but there's no spec-level guarantee.
  - **Suggested Fix**: Add `self.interrupt_capable == old(self).interrupt_capable` to all mutation function postconditions, or add it to `wf()` as a ghost-tracked invariant.

## Positive Observations

- **Zero assume/external_body**: The verification is entirely self-contained with no soundness escape hatches. All 32 verification conditions pass cleanly.
- **Strong well-formedness invariant**: The `wf()` predicate is comprehensive, covering finiteness, count consistency, queue disjointness, running exclusivity, kernel safety, PID bounds, and overflow prevention. This is a well-designed invariant.
- **Kernel liveness proven**: The critical safety property that PID 0 (kernel) is always alive (running or ready) and never enters suspended/interrupted/zombie states is formally proven and maintained across all transitions.
- **PID freshness chain**: The monotonic PID allocation scheme is cleanly verified — `lemma_next_pid_is_fresh` proves that new PIDs never collide, and `create_process` uses this to maintain disjointness.
- **Clean decomposition of complex transitions**: The original `sleep`, `exit`, and `exit_thread` functions have multiple outcome branches. The verified code decomposes these into separate functions (`sleep_running` vs `sleep_thread_running`, `exit_running` vs `exit_thread_running` vs `exit_thread_to_suspended` vs `exit_thread_to_zombie`), each fully specified. This is a good verification strategy.
- **Well-documented trust boundaries**: The three trust boundaries (T1: scheduler choice, T2: RefCell borrow, T3: thread-level details) are explicitly documented with rationale.
- **Excellent spec/proof separation**: The three-file split cleanly separates concerns — spec for abstract model, proof for lemmas, exec for implementation. Proof lemmas are targeted and invoked only where needed.
- **Proof of `resume_all_interrupted`**: The `lemma_union_disjoint_len` proof with structural induction is a non-trivial proof that is cleanly executed.

## Summary

This is a solid verification of the process manager's core state machine. The verification successfully proves the most important safety properties: process queue partitioning (each process in exactly one state), kernel liveness (PID 0 always alive), PID uniqueness, and overflow safety. The abstraction from concrete kernel types to ghost PID sets is appropriate and well-executed.

The main weakness is a semantic equivalence gap in `terminate_ready_to_interrupted`, which models a queue transition that doesn't occur in the original code (the original atomically terminates and resumes, leaving the process in ready). Two `exit_thread_*` functions also have incomplete frame postconditions for `number_buffered_messages`.

Coverage is focused on the state machine transitions (~24 verified functions out of ~50 in the original), with thread-level operations, memory management, synchronization primitives, and the outer `ProcessManager` wrapper left as trust boundaries. This is a reasonable scope for the verification effort, as the state machine is the most critical aspect of the process manager. The trust boundaries are well-documented and the rationale is sound.

**Recommendations (priority order)**:
1. Fix `terminate_ready_to_interrupted` semantic mismatch (High).
2. Add missing `number_buffered_messages` postconditions to `exit_thread_to_suspended`/`exit_thread_to_zombie` (Medium).
3. Document outer `ProcessManager` → inner function mapping (Medium).
4. Consider adding error-path verification for `terminate` and `create_process` (Medium).
