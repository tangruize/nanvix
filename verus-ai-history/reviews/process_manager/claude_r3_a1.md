# Review: process_manager (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

*None.*

### High

- **Location:** `exit_running` (exec: process_manager.rs:369-398) vs. original `exit()` (mod.rs:895-946)
  - **Description:** The original `exit()` function has two outcomes: `Ok` → process to ready (remaining runnable threads) or `Err` → process to zombie. The verified model correctly splits this into `exit_running` (→zombie) and `exit_thread_running` (→ready). However, the original `exit()` comment on line 924 says "only sleeping threads left → zombies," but the type system shows it returns `ZombieProcess`, not `SleepingProcess`. The original `exit` has no path to suspended — only ready or zombie. Yet `exit_thread` (mod.rs:974-1034) has a *three-way* branch: ready, suspended, or zombie. The verified model covers all three paths for `exit_thread` (`exit_thread_running`, `exit_thread_to_suspended`, `exit_thread_to_zombie`) but there is no `exit_to_suspended` variant for `exit()` itself. This is actually correct since the original `exit` only has two branches, but the code comment in the original is misleading. **No actual bug — verified model matches the original.**

- **Location:** `recv_message` (exec: process_manager.rs:728-744) vs. original
  - **Description:** The verified model includes a `recv_message` function that decrements the buffered message count, but there is no direct `recv_message` method in `ProcessManagerInner` in the original. The decrement actually happens in the `unsafe` submodule (`unsafe.rs`), which is outside the scope of this module's verification. The model correctly identifies this in its doc comment (line 726-727), but it means the send/receive message count tracking has a cross-module trust boundary that isn't explicitly listed in the T1/T2/T3 boundary documentation.
  - **Suggested Fix:** Add a T4 trust boundary note for cross-module operations (unsafe submodule), or verify the decrement in the unsafe module's verification.

### Medium

- **Location:** Spec: `spec_kernel_safe` (spec: process_manager.spec.rs:232-237)
  - **Description:** The invariant states that PID 0 is always either running or ready, and never suspended/interrupted/zombie. The original enforces this via `panic!` guards in `sleep()` (line 739) and `exit()` (line 914). The verified model correctly captures this via preconditions (`running_pid != 0` on `sleep_running`, `exit_running`, etc.). However, the `terminate` function in the original (line 1038-1041) also rejects kernel PID termination. The verified model covers this via the `pid != 0` precondition on `terminate_ready` and implicitly in `outer_terminate_error`. This is adequately modeled.

- **Location:** `terminate_suspended` (exec: process_manager.rs:629-650) vs. original `terminate` (mod.rs:1067-1073)
  - **Description:** In the original, terminating a suspended process moves it to `interrupted` (via `process.terminate()` which returns an `InterruptedProcess`). The verified model correctly maps this as `suspended→interrupted`. However, the verified model does not include a `terminate_suspended_to_zombie` variant. In the original, the suspended-path terminate always produces an `InterruptedProcess` (the type system enforces this), so this is not a gap — the model is correct. But it means that for a suspended process, terminate always interrupts rather than zombifies, which is a semantic difference from the ready-path terminate (which can zombify). This asymmetry is inherent in the original design and correctly captured.

- **Location:** No `assume` or `external_body` in any file
  - **Description:** This is a **positive** finding. There are zero `assume` statements, zero `external_body` annotations, and zero `trusted` markers in the entire verification. All 86 verified items are proven from first principles.

- **Location:** `LinkedList<Process>` → `Set<int>` abstraction (spec: process_manager.spec.rs:122-138)
  - **Description:** The abstraction from ordered `LinkedList` to unordered `Set<int>` is well-justified for current verification goals (process partitioning, kernel liveness, PID uniqueness). However, this means scheduling fairness (FIFO ordering, earliest-admission-time selection) is not verified. The `chosen_next` parameter abstracts the scheduler choice (T1 boundary), which is appropriate for the current scope. If fairness properties become important, the ready queue should use `Seq<int>`.
  - **Suggested Fix:** No fix needed now; document as future work if fairness verification is desired.

- **Location:** Outer `ProcessManager` wrapper (exec: process_manager.rs:1328-1670)
  - **Description:** The outer `ProcessManager` methods are modeled as pass-through stubs to the inner methods. The `Rc<RefCell<_>>` borrow checking is explicitly placed at trust boundary T2. This is a reasonable modeling choice for a single-threaded kernel. However, the outer stubs (e.g., `outer_vmcopy_from_user`, `outer_mmap`) don't verify that the borrow pattern is *correct* — they just assert wf() preservation as trivial no-ops. The value is in documenting the mapping, not in proving deep properties.
  - **Suggested Fix:** Acceptable for current scope. Consider adding a comment that T2 verification would require modeling RefCell borrow state.

### Low

- **Location:** `forge_user_context` (exec: process_manager.rs:1163-1171)
  - **Description:** The verified stub has precondition `ghost_ready@.contains(pid)`, but in the original (mod.rs:203-244), `forge_user_context` is a static method called during `create_process` and `create_thread` *before* the process is necessarily in the ready queue (it's called to set up the context, then the process is added to ready). The precondition is overly strong — the original function doesn't require the process to be in any queue since it's a context setup utility.
  - **Suggested Fix:** Remove the `ghost_ready@.contains(pid)` precondition or document why it's modeled this way (perhaps the stub is only relevant post-creation).

- **Location:** `spec_can_create_process` (spec: process_manager.spec.rs:318-319)
  - **Description:** The spec bounds PID space at `i32::MAX`. The original uses `ProcessIdentifier::from(i32::from(pid) + 1)` which could overflow. The verified model correctly identifies this limitation and prevents it via the precondition `spec_can_create_process()` (which checks `next_pid < i32::MAX`). The documentation note about PID recycling as future work is helpful.

- **Location:** `harvest_zombies_wrapper` vs. `harvest_zombie` (exec: process_manager.rs:1319-1326, 657-676)
  - **Description:** The `harvest_zombie` function models removing a *single* zombie by PID. The original `harvest_zombies` (mod.rs:1191-1208) pops the *front* of the zombie queue (not by PID). The verified model abstracts this correctly since `Set` membership doesn't depend on ordering. The `harvest_zombie` postcondition `!self.spec_process_exists(pid)` is a strong and useful guarantee: after harvesting, the PID is truly gone from all queues.

## Positive Observations

- **Zero assume/external_body/trusted:** The entire verification (86 items) is proven without any axioms or trust assumptions beyond the three explicitly documented trust boundaries (T1: scheduler choice, T2: RefCell borrow, T3: thread-level logic). This is excellent for a kernel component.
- **Comprehensive invariant (`wf()`):** The well-formedness predicate captures seven distinct sub-invariants (finiteness, cardinality matching, disjointness, running exclusivity, kernel safety, PID bounds, count bounds) that together provide strong structural guarantees.
- **Kernel liveness proven:** The most important safety property — that PID 0 (kernel) is always alive (running or ready, never suspended/interrupted/zombie) — is proven to hold across all state transitions.
- **PID uniqueness proven:** The freshness lemmas (`lemma_next_pid_is_fresh`, `lemma_fresh_pid_not_in_sets`) prove that newly allocated PIDs never collide with existing processes.
- **Complete coverage of state transitions:** All five process queues (running, ready, suspended, interrupted, zombie) and all transitions between them are modeled: create (→ready), schedule (running↔ready), sleep (running→suspended or running→ready), exit (running→zombie or running→ready), wakeup (suspended→ready), alarm (suspended→interrupted), resume (interrupted→ready), terminate (ready→zombie or suspended→interrupted), harvest (zombie→removed).
- **Well-structured trust boundaries:** The T1/T2/T3 decomposition is clearly documented with specific code line references, making it easy to audit what is and isn't verified.
- **Excellent documentation:** The spec file header (151 lines of structured comments) provides a thorough verification model description, trust boundary documentation, error path analysis, and abstraction justifications. This is above-average documentation quality for formal verification.
- **Proof structure:** The proof file is lean (232 lines) with focused lemmas. The `lemma_union_disjoint_len` proof uses clean structural induction. The `lemma_kernel_alive_after_schedule` covers all three cases (kernel running + chosen, kernel running + not chosen, kernel ready).
- **Overflow safety:** Arithmetic overflow is addressed by `spec_counts_bounded`, which proves that total process count ≤ next_pid (an i32), so all pairwise sums of usize counts fit within bounds.

## Summary

This is a high-quality verification of an OS kernel process manager. The verification successfully proves the most important safety properties: process queue partitioning (each process in exactly one state), kernel liveness (PID 0 always alive), PID uniqueness, and overflow safety — all without any `assume` or `trusted` annotations.

The abstraction from concrete kernel types (`LinkedList<RunningProcess>`, etc.) to ghost `Set<int>` is well-justified and clearly documented. The trust boundary decomposition (scheduler choice, RefCell borrow, thread-level state) is appropriate for the verification scope.

The main limitation is that the verification operates at the queue-level state machine, not at the per-process or per-thread level. This is a reasonable scoping decision for a first verification pass. The ~78 verified functions (exec) cover all original functions from both `ProcessManagerInner` (34 functions) and `ProcessManager` outer wrapper (24 functions), with the addition of decomposed transition variants to handle the various branches in complex functions like `exit_thread` and `terminate`.

**Recommendations:**
1. Document the cross-module trust boundary for `recv_message` (unsafe submodule).
2. Fix the overly strong precondition on `forge_user_context` stub.
3. Consider future work on scheduling fairness (Seq-based ready queue) and capability state verification.
