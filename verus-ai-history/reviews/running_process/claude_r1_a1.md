# Review: running_process (claude-opus-4.6)

## Grade: B+

## Issues Found

### Critical

- **Location:** `sleep()` in exec file (running.rs:376–381), interrupted branch
  **Description:** When the original `sleep()` takes the interrupted branch, it constructs an `InterruptedProcess::from_sleeping()` with `Some(sleeping_threads)` (the newly-sleeping thread plus any existing sleeping threads). The Verus model constructs `InterruptedProcess` with only `interrupted_thread_ids` and `zombie_thread_ids`, completely dropping the sleeping thread IDs. The sleeping threads are *not* passed through to `interrupted_resume()`, so they are silently lost. In the original code, `InterruptedProcess::from_sleeping` receives the sleeping threads and preserves them through `resume()` into the resulting `RunnableProcess`. This means the Verus model may verify properties about the result that are only valid because it ignores that sleeping threads should still be present in the output.
  **Suggested Fix:** Extend the `InterruptedProcess` boundary model to include a `sleeping_thread_ids` field, and thread it through `interrupted_resume()` so the resulting `RunnableProcess` carries the sleeping threads. Update the `SleepResult::Runnable` ensures clause to assert the sleeping threads are preserved.

- **Location:** `exit()` in exec file (running.rs:461–466), InterruptedProcess construction
  **Description:** Same structural issue as `sleep()`. The original `exit()` at line 217–222 calls `InterruptedProcess::from_sleeping(self.state, self.sleeping_threads.take(), interrupted_threads, Some(zombie_threads))`. Note `self.sleeping_threads.take()` — by this point, sleeping threads have already been taken earlier (line 208), so this is always `None`. The Verus model correctly doesn't pass sleeping threads here, but the `InterruptedProcess` boundary model still lacks a sleeping thread field, making this modeling gap invisible and fragile for sibling module verification.
  **Suggested Fix:** Add the sleeping threads field to `InterruptedProcess` even if it is `None`/empty in this call site, for structural completeness.

### High

- **Location:** `exit_thread()` in exec file (running.rs:549–555), interrupted branch
  **Description:** In the original `exit_thread()` (source line 281–289), when the interrupted branch is taken, `InterruptedProcess::from_sleeping` is called with `self.zombie.take()` instead of the newly-constructed `zombie_threads` (which includes the just-exited running thread). The Verus model correctly passes `new_zombie_ids` (line 552). This means the Verus model and the original code may diverge: the original passes `self.zombie.take()` which at that point is `None` (the zombie was already consumed into `zombie_threads` at line 261), so the exited thread's zombie state would be lost in the original code. The Verus model correctly constructs this, so it is actually *more correct* than the original. This is a potential bug in the original source that the verification model silently fixes rather than documenting.
  **Suggested Fix:** Document this as a known divergence or potential original-code bug. The verification should either faithfully model the (possibly buggy) original or explicitly note the fix.

- **Location:** Functions `state()`, `state_mut()`, `running_mut()` — missing from exec file
  **Description:** The original source has three accessor methods (`state()`, `state_mut()`, `running_mut()`) that return references to internal fields. These are completely unverified. While `state()`/`state_mut()` are noted as "elided (ProcessState access modeled via PID)", `running_mut()` returns a mutable reference to the running thread, which could be used to modify thread state in arbitrary ways. This is a coverage gap.
  **Suggested Fix:** Add at minimum a spec-only model for `running_mut()` documenting that it permits mutation of the running thread. If the running thread's invariants matter (e.g., thread ID stability), this should be specified.

- **Location:** `try_join_thread()` — missing from exec file
  **Description:** `try_join_thread()` (original lines 339–395) is a non-trivial function with complex control flow (searches multiple thread lists, returns different error types). It is listed as "modeled spec-only" in the header but there is only a `spec_find_thread()` function that doesn't model the `try_join_thread` return type semantics (Error for running thread, Ok(ZombieThread) for zombie, Err(Ok(Condvar)) for live threads, Err(Err(Error)) for not found). The function mutates `self.zombie` via `take()` — this side effect is not modeled at all.
  **Suggested Fix:** Add at least a spec-level model that captures the key property: joining a running thread returns an error, joining a zombie thread removes it from the zombie list, and joining a live thread returns a condvar.

### Medium

- **Location:** `sleep()` spec (running.rs:333–350), Runnable branch via interrupted
  **Description:** When `sleep()` returns `SleepResult::Runnable` via the interrupted path, the ensures clause does not specify the sleeping thread content of the resulting `RunnableProcess`. The original code passes `Some(sleeping_threads)` to `InterruptedProcess::from_sleeping()` which eventually flows into the `RunnableProcess`. The spec only asserts PID preservation and well-formedness but says nothing about the sleeping thread list in the result. This is a weak specification.
  **Suggested Fix:** Add postconditions about the sleeping thread content when the interrupted path is taken, or at minimum assert the sleeping count in the result.

- **Location:** `exit()` spec (running.rs:414–433), Runnable branch
  **Description:** When `exit()` returns `ExitResult::Runnable`, the spec only asserts PID preservation, well-formedness, and that interrupted or sleeping threads existed. It doesn't assert anything about the zombie list in the resulting `RunnableProcess` (which should contain the running thread + all ready threads + original zombies). This is masked by the `external_body` on `interrupted_resume()`, but makes the spec weaker than it needs to be.
  **Suggested Fix:** Thread the zombie count through `interrupted_resume()` in the boundary model so that the exit result's zombie content can be specified.

- **Location:** `wakeup()` in exec file (running.rs:586), oracle parameter `found`
  **Description:** The `wakeup()` function takes a `found: bool` oracle parameter that tells it whether the thread exists in the sleeping list. In the original code, this is determined by `sleeping_threads.remove_if()` — an actual runtime search. The oracle parameter is a modeling compromise that shifts the responsibility of thread lookup outside the verified boundary. The precondition `found == Self::spec_seq_contains(...)` ties it to the spec, which is correct, but this means the caller must be trusted to provide the correct value.
  **Suggested Fix:** Document this oracle parameter as a trust assumption in the spec file header. Consider whether the caller's correctness is verified elsewhere.

- **Location:** `find_thread()` and `find_thread_mut()` — missing from exec file
  **Description:** Both `find_thread()` and `find_thread_mut()` are non-trivial functions that search all thread lists. They are modeled only as spec functions (`spec_find_thread()`). While the spec model captures the search semantics correctly, there is no exec-level verification that the implementation matches.
  **Suggested Fix:** This is acceptable for reference-returning functions that can't be modeled in Verus's ownership system, but should be explicitly documented as a trust boundary.

- **Location:** `sleep()` exec (running.rs:329), alarm parameter dropped
  **Description:** The original `sleep()` takes an `alarm: Option<SystemTime>` parameter that controls the sleep duration. The Verus model drops this parameter entirely. While this is a HAL-boundary simplification, the alarm parameter affects the `SleepingThread` state and could influence wakeup behavior. This is a modeling gap that could matter for temporal correctness properties.
  **Suggested Fix:** Document the alarm elision explicitly. If alarm-based wakeup behavior is ever verified, this will need to be revisited.

### Low

- **Location:** Proof file (running.proof.rs), multiple lemmas
  **Description:** Several lemmas have trivially true ensures clauses (e.g., `lemma_schedule_preserves_total_threads` ensures `self.spec_ready_count() + 1 == self.spec_ready_count() + 1`, and `lemma_schedule_result_has_ready`, `lemma_sleep_with_ready_gives_runnable`, etc. all ensure `true`). These lemmas serve as documentation but provide zero proof value — they don't actually prove anything that the main function specs don't already prove.
  **Suggested Fix:** Either strengthen these lemmas to prove something non-trivial (e.g., actually invoke `schedule()` and reason about the result), or remove them to reduce noise. As-is, they give a false sense of proof coverage.

- **Location:** `wf()` spec (running.spec.rs:251–256)
  **Description:** The well-formedness predicate only checks that exec-level counters match ghost sequence lengths. It does not enforce thread ID uniqueness across lists (acknowledged as a trust assumption from Rust's ownership model). This is reasonable but means the verification cannot detect bugs where a thread accidentally appears in two lists.
  **Suggested Fix:** Consider adding an optional stronger `wf_strict()` that asserts uniqueness, even if it's only used in specific proof contexts.

- **Location:** `EXIT_STATUS_INTERRUPTED` spec constant (running.spec.rs:129)
  **Description:** This constant is defined (`4` = EINTR) but never used anywhere in the spec, proof, or exec files.
  **Suggested Fix:** Remove the unused constant, or use it in the `exit()` spec if it should constrain the exit status of interrupted threads.

- **Location:** Exec file struct fields are all `pub`
  **Description:** All fields in the Verus `RunningProcess` struct are `pub` (noted as "for Verus proof ergonomics"). This is contrary to the Nanvix coding standard that struct fields must be private with getter/setter methods. While justified for verification, it means the verified model has weaker encapsulation than the original.
  **Suggested Fix:** No action needed for verification purposes, but document this deviation from coding standards.

## Positive Observations

- **Clean spec/proof/exec separation.** The three-file split is well-organized with clear responsibilities: specs define the abstract model and invariants, proofs provide supporting lemmas, and exec contains the verified implementations.
- **Strong PID preservation.** Every state transition correctly ensures PID is preserved, which is a critical safety property for process identity.
- **Correct branch condition modeling.** The `schedule()`, `sleep()`, `exit()`, and `exit_thread()` functions all correctly model the branch conditions from the original source (ready > 0, interrupted > 0, sleeping > 0).
- **Well-formedness propagation.** All transition functions require `wf()` as a precondition and ensure `wf()` on the result, establishing an inductive invariant.
- **Content-level specs for `schedule()` and `wakeup()`.** These functions have strong postconditions specifying the exact content of the resulting thread lists (not just counts), which is the gold standard for verification.
- **Thorough view type definitions.** The abstract view types and `View` implementations provide a clean abstraction layer for compositional reasoning.
- **Only one `external_body`.** The `interrupted_resume()` function is the sole trust boundary within the module, and its postconditions (PID preservation + well-formedness) are reasonable and well-documented.
- **Verification passes cleanly.** All 32 verification conditions pass without errors, confirming internal consistency.

## Summary

The verification is solid for the core state transitions (`schedule`, `sleep`, `exit`, `exit_thread`, `wakeup`, `get_tid`, `new`) with good PID preservation proofs and well-formedness invariants. The main weaknesses are: (1) the `InterruptedProcess` boundary model drops sleeping thread information, which could mask data loss in the `sleep()` interrupted branch; (2) three original functions (`try_join_thread`, `find_thread`, `find_thread_mut`) are only partially modeled or missing exec-level verification; (3) some postconditions are weaker than they could be (especially for `exit()` and `sleep()` Runnable results via the interrupted path); and (4) several proof lemmas are trivially true and provide no real proof value. The verification correctly identifies the state machine structure and proves the key branching logic, but would benefit from stronger content-level specifications on the interrupted resume path and exec-level modeling of the thread search functions.
