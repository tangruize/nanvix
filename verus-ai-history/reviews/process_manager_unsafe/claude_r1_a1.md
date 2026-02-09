# Review: process_manager_unsafe (claude-opus-4.6)

## Grade: B+

## Issues Found

### Critical

_None._

### High

- **Location:** `get()` / `get_mut()` (exec — process_manager_unsafe.rs)
  - **Description:** The original `unsafe fn get()` and `unsafe fn get_mut()` are public unsafe accessors that return `&ProcessManager` / `&mut ProcessManager` from the `static mut PROCESS_MANAGER`. These functions are entirely absent from the verified model. They are the primary entry point for every other function in the module — every operation calls `Self::get_mut().try_borrow_mut()?` internally. The soundness of the entire module hinges on there being at most one mutable reference at a time, and that the global is initialized before use. Neither property is modeled or proven.
  - **Suggested Fix:** Add spec functions/lemmas modeling the global singleton pattern: (1) `get()` requires `initialized == true` and ensures the returned reference is wf(), (2) `get_mut()` requires the same plus exclusive access (no other borrows). Even if Verus cannot model raw pointer aliasing, the preconditions and postconditions should be explicitly stated.

- **Location:** `switch()` (exec — process_manager_unsafe.rs, line 177–223)
  - **Description:** The verified `switch()` accepts a `new_inner: ProcessManagerInner` parameter that is assumed to already reflect the state transition (e.g., running process has changed). In the original code, `switch()` does **not** modify `ProcessManagerInner` — it only updates atomics (`CURRENT_PID`, `CURRENT_TID`, `REMAINING_QUANTUM`) and calls `ContextInformation::switch()`. The inner state mutation happens **before** `switch()` is called (in `exit()`, `sleep()`, `schedule()`). By accepting `new_inner` as a parameter, the verified model conflates the inner mutation with the context switch, making it impossible to verify that the inner state transition and the atomic update happen in the correct order and are consistent with each other.
  - **Suggested Fix:** Separate the inner state transition from the `switch()` function. The verified `switch()` should only model atomic updates to PID/TID/quantum, and the inner state should be updated before `switch()` is called (matching the original code flow).

### Medium

- **Location:** `giveup_no_switch()` / `giveup_with_switch()` (exec — process_manager_unsafe.rs, lines 239–284)
  - **Description:** The original `giveup()` is a single function with an `if/else` branch. The verified model splits it into two separate functions. While this is acceptable for verification, it means there is no single entry point that models the complete `giveup()` logic including the branch condition. A caller cannot use the verified model to reason about a single call to `giveup()` without manually composing the two cases.
  - **Suggested Fix:** Add a composite `giveup()` function that dispatches to `giveup_no_switch()` or `giveup_with_switch()` based on `remaining_quantum > 1`, with postconditions covering both cases. This would match the original API surface more closely.

- **Location:** `join_thread_harvest()` / `join_thread_error()` (exec — process_manager_unsafe.rs, lines 526–543)
  - **Description:** The original `join_thread()` is a complex loop with three control-flow paths: (1) zombie found → harvest, (2) not yet zombie → wait on condvar, (3) error. The verified model provides two trivial stubs (`join_thread_harvest` and `join_thread_error`) that only assert `wf()` is preserved, but **omit the blocking/condvar-wait path entirely**. The most interesting correctness property — that the thread eventually makes progress and the loop terminates when the target thread exits — is not captured.
  - **Suggested Fix:** At minimum, add a `join_thread_wait()` function modeling the condvar-wait path (which delegates to `sleep()`). Ideally, add a spec-level liveness argument or loop invariant showing that the loop terminates when the target thread becomes a zombie.

- **Location:** `exit()` / `exit_thread()` (exec — process_manager_unsafe.rs, lines 332–389)
  - **Description:** The original `exit()` and `exit_thread()` return `Result<!, Error>` (divergent — they never return on success). The verified model returns normally. While the module header notes this as a T5 boundary, the divergence property is a key correctness property: after `exit()` succeeds, the calling context's stack frame is invalidated. Not modeling divergence means the verified model does not prevent reasoning about code that would execute after `exit()` returns.
  - **Suggested Fix:** Document this as an explicit limitation. Consider adding a ghost postcondition like `ensures false` on the success path to model divergence, or split into success (divergent) and error (returns) paths.

- **Location:** `sleep()` (exec — process_manager_unsafe.rs, lines 296–318)
  - **Description:** The original `sleep()` has important post-switch logic: after the thread is woken up, it checks `interrupt_reason()` and may return `Err(SleepError::Interrupted(reason))`. This post-wakeup check is not modeled in the verified version, which only captures the state transition into sleep and the context switch.
  - **Suggested Fix:** Add a post-switch model that captures the interrupt-reason check. This could be a separate function `sleep_post_wakeup()` that ensures the state is consistent after the thread resumes.

### Low

- **Location:** `spec_quantum_valid` (spec — process_manager_unsafe.spec.rs, line 85–88)
  - **Description:** The quantum validity invariant requires `remaining_quantum >= 1`. However, the original `giveup()` performs a context switch when `remaining_ticks <= 1` (which includes the case `remaining_ticks == 1`). After the switch, the quantum is reset to `SCHEDULER_FREQ`. This means the quantum can momentarily be 1 before a switch, which is valid, but the invariant does not capture the transient state where `remaining_quantum == 0` could theoretically occur if there's a bug. The lower bound of 1 is correct but the reasoning should be made explicit.
  - **Suggested Fix:** Add a comment to the spec explaining why `remaining_quantum >= 1` is the correct lower bound (quantum is decremented only when `> 1`, and reset to `scheduler_freq` otherwise).

- **Location:** Delegation functions (`get_mutex`, `put_mutex_guard`, etc.) (exec — process_manager_unsafe.rs, lines 398–469)
  - **Description:** The delegation functions take no parameters and have no meaningful postconditions beyond `wf()` preservation. They don't model the actual return types (`Result<Mutex, Error>`, `Result<(), Error>`, etc.) or error conditions. While this is acceptable for queue-level verification, it means the verified model provides no guarantees about the success/failure semantics of these operations.
  - **Suggested Fix:** Consider adding at least a `Result`-like return type to capture that these operations can fail, even if the error conditions are left as trust boundaries.

- **Location:** `try_recv_some()` (exec — process_manager_unsafe.rs, lines 479–499)
  - **Description:** The original `try_recv()` accesses `running.state_mut().receive_message(tid)` using a specific `tid` parameter. The verified model does not take a `tid` parameter — it only models the message count decrement. This means the model does not verify that the message is being received by the correct thread.
  - **Suggested Fix:** Add a `tid` parameter (even if it's a ghost parameter) to model thread-specific message reception.

- **Location:** `wf()` predicate (spec — process_manager_unsafe.spec.rs, line 105)
  - **Description:** The `wf()` predicate does not include a constraint tying `current_tid` to a thread within `current_pid`'s process. In the original system, the current TID must belong to the current PID's thread set. Without this, the model permits states where `current_tid` refers to a thread in a different process.
  - **Suggested Fix:** If the inner model tracks thread-to-process mappings, add `spec_tid_belongs_to_pid(current_tid, current_pid)` to `wf()`. If not, document this as a trust boundary.

## Positive Observations

- **Clean spec/proof/exec separation.** The three-file split is well-organized with clear responsibilities: specs define the abstract model, proofs contain lemmas, and exec contains the verified implementation.
- **Comprehensive context-switch modeling.** The `switch()` function correctly captures all three cases (hard switch with PID change, hard switch same PID, soft switch) with precise postconditions for each.
- **Quantum management is well-verified.** The `giveup_no_switch()` and `giveup_with_switch()` functions correctly model quantum decrement and reset, with appropriate lemmas proving bounds.
- **Trust boundaries are explicitly documented.** The module header clearly identifies T5 (raw pointer context switch), T6 (atomic ordering), T7 (RefCell borrow), and T8 (interrupt enable/disable) as trust boundaries.
- **All 30 verification conditions pass** with no errors, indicating the proof is self-consistent.
- **The `wf()` invariant composition is well-designed** — it decomposes into independently checkable sub-predicates (inner wf, PID consistency, TID validity, quantum validity, frequency positivity, FPU owner validity).
- **Inner module integration is clean.** The verified model correctly delegates to `ProcessManagerInner` for state transitions and composes the inner `wf()` with outer invariants.

## Summary

The verification is solid for what it covers: global state invariant preservation across initialization, context switches, quantum management, and delegation operations. The spec/proof/exec separation is clean and the trust boundaries are well-documented.

The main gaps are: (1) the `get()`/`get_mut()` global singleton accessors are not modeled, leaving the fundamental safety property (single-mutable-reference) unverified; (2) `switch()` conflates inner state mutation with atomic updates by accepting `new_inner` as a parameter rather than separating concerns; (3) `join_thread()` blocking path and `sleep()` post-wakeup logic are not modeled; (4) divergence of `exit()`/`exit_thread()` is not captured.

Recommendations: Address the High issues first (especially modeling the singleton access pattern and separating inner mutation from switch). Then add the missing `join_thread` wait path and `sleep` post-wakeup logic. The delegation function stubs are acceptable as-is given the queue-level abstraction, but adding return types would strengthen the model.
