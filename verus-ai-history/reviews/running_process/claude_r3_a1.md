# Review: running_process (claude-opus-4.6)

## Grade: A-

## Verification Result

43 verified, 0 errors. All proof obligations discharged by Verus.

## Issues Found

### Critical

None.

### High

- **Location:** `wakeup()` and `try_join_thread()` (exec file)
- **Description:** Oracle parameters (`found: bool`, `tag: u8`) are required because ghost `Seq::contains()` cannot be evaluated at exec time. While the preconditions (`found == spec_seq_contains(...)`, `tag == spec_try_join_thread(tid)`) are verified at call sites, the pattern introduces an unusual proof obligation on callers. If a caller is itself `external_body` or uses `assume`, the oracle contract could be silently violated. This is a structural trust gap that should be tracked.
- **Suggested Fix:** Document at the module level that all callers of `wakeup()` and `try_join_thread()` must be verified (not `external_body`) for the oracle contracts to hold. Consider adding a cross-module integration test or proof that exercises these call sites.

### Medium

- **Location:** `state_mut()`, `running_mut()` (exec file, lines 359–400)
- **Description:** Both are `external_body` with frame condition postconditions asserting that all modeled fields (PID, thread IDs, counts) are unchanged after the call. These postconditions are trusted, not proven. If the original code mutates ProcessState in a way that changes the PID, or RunningThread in a way that changes the thread ID, the frame condition would be silently violated.
- **Suggested Fix:** When ProcessState and RunningThread are independently verified, add cross-module proof obligations that their mutable accessors preserve identity fields. The `mutation_frame_preserved()` spec in the spec file is the correct hook for this — it should eventually be required as a postcondition on the downstream types.

- **Location:** `find_thread()`, `find_thread_mut()` (exec file, lines 517–554)
- **Description:** Both are modeled as pure spec-only computations returning `Ghost<Option<int>>`. The original performs an exec-level linear search across five thread collections with specific priority ordering (running → ready → interrupted → sleeping → zombie). The verification proves the spec function `spec_find_thread()` has the correct search order, but the correspondence between the original's iterator-based search and the spec's existential quantifier is a trust assumption. Additionally, `find_thread_mut()` returns `&mut ThreadRefMut` in the original, permitting mutation of the found thread — this mutation capability is completely unmodeled.
- **Suggested Fix:** Document that exec-level search correctness is a trust assumption until Verus supports reference-returning functions. For `find_thread_mut()`, consider adding a frame condition lemma that callers must satisfy after mutating through the returned reference.

- **Location:** `interrupted_resume()` (exec file, line 226)
- **Description:** This `external_body` function models `InterruptedProcess::resume()` from a sibling module. The postconditions are detailed (PID preservation, sleeping/zombie pass-through, exactly one ready from front of interrupted, tail becomes remaining interrupted, thread count conservation). However, these postconditions are assumed, not proven in this module. If the sibling module's `resume()` implementation changes, these assumptions could silently become invalid.
- **Suggested Fix:** When the `interrupted` module is verified, add a cross-module proof that its `resume()` satisfies these postconditions. Consider adding a comment with the specific source file and function that must be verified to discharge this assumption.

### Low

- **Location:** `wakeup()` precondition (exec file, line 977)
- **Description:** The precondition `self.sleeping_count > 0 || !found` is documented as a "solver hint" that is redundant given `wf()` and `found == spec_seq_contains(...)`. Redundant preconditions can be confusing to callers and may mask weaknesses in the proof.
- **Suggested Fix:** Add a proof lemma `lemma_wf_and_found_implies_sleeping_positive()` that derives `sleeping_count > 0` from `wf()` and `found`, then remove the redundant precondition.

- **Location:** `wakeup()` precondition (exec file, line 974)
- **Description:** The precondition `self.ready_count < u64::MAX` prevents overflow on `ready_count + 1`. While practically reasonable (a process cannot have 2^64 threads), the original code has no such explicit bound — the `NonEmptyVecDeque::push_back` doesn't check for overflow at the type level. This is a minor specification strengthening.
- **Suggested Fix:** Acceptable as-is. Could document this as a modeling assumption that real systems never approach this limit.

- **Location:** Condvar and ContextInformation elision (all state-transition functions)
- **Description:** All state transitions in the original return `*mut ContextInformation` (HAL boundary) and some return `Condvar` (sync boundary). These are completely elided in the verification model. The verification does not capture that the correct context pointer or condvar is propagated to the caller.
- **Suggested Fix:** Acceptable for process state machine verification. Document as out-of-scope. If HAL or sync correctness is ever verified, these elisions must be revisited.

- **Location:** `wf()` spec (spec file, line 335)
- **Description:** Well-formedness does not enforce thread ID uniqueness across lists. A `RunningProcess` with the same thread ID in both ready and sleeping lists would pass `wf()`. This is explicitly documented as a trust assumption from Rust's ownership model, and `wf_strict()` is provided for downstream proofs that need uniqueness.
- **Suggested Fix:** Acceptable as-is. The `wf_strict()` predicate is the correct mitigation. Consider adding a proof lemma showing that `wf_strict()` is preserved by all operations (currently only `wf()` preservation is proven).

## Positive Observations

- **Complete function coverage:** All 13 public methods in the original (`new`, `state`, `state_mut`, `running_mut`, `schedule`, `sleep`, `exit`, `exit_thread`, `get_tid`, `wakeup`, `try_join_thread`, `find_thread`, `find_thread_mut`) have verified counterparts.
- **Content-level specifications:** Postconditions specify exact sequence content (e.g., `result.ready_thread_ids@ == self.ready_thread_ids@.push(self.running_thread_id@)`), not just counts. This is significantly stronger than count-only specifications.
- **Bug discovery:** Verification identified a real bug in `exit_thread()` where `self.zombie.take()` at the interrupted branch was always `None` because the zombie list had already been consumed. The original source was patched.
- **Thread count conservation:** Proven via lemmas for `schedule()` and `wakeup()` — total thread count is invariant across these operations.
- **PID immutability:** Proven across all operations via postconditions and the `mutation_frame_preserved()` spec.
- **Excellent documentation:** Trust boundaries, modeling decisions, oracle parameters, and elisions are all thoroughly documented in file headers and inline comments. The Verification Model and Trust Assumptions sections in the spec file are particularly well-done.
- **Clean spec/proof/exec split:** Specifications (view types, spec functions, well-formedness) are cleanly separated from proofs (lemmas) and exec code (implementations). The `include!()` pattern keeps related code together while maintaining logical separation.
- **Branch-aware postconditions:** `sleep()`, `exit()`, and `exit_thread()` have postconditions that specify different guarantees for each branch, matching the original's control flow. The interrupted branch in `sleep()` and `exit()` correctly threads sleeping/zombie lists through `interrupted_resume()`.
- **Precise boundary modeling:** The `InterruptedProcess`, `RunnableProcess`, `SleepingProcess`, `ZombieProcess` boundary types faithfully capture the fields needed for cross-module reasoning, including `InterruptedProcess.sleeping_thread_ids` which is critical for the `sleep()` interrupted path.

## Summary

This is a high-quality verification of a complex kernel process state machine. All 13 original methods are covered with precise, content-level specifications that go well beyond simple count tracking. The verification successfully identified a real bug in `exit_thread()`. Trust boundaries are minimal (4 `external_body` annotations) and thoroughly documented. The main areas for improvement are: (1) proving `wf_strict()` preservation across operations, (2) documenting that oracle parameter callers must themselves be verified, and (3) eventually discharging the `interrupted_resume()` and frame condition trust assumptions when sibling modules are verified. The spec/proof/exec split is clean and the documentation is exemplary.
