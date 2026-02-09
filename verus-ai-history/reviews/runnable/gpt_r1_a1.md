# Review: runnable (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `RunnableProcess::run` (exec) in `verus/split/kernel/pm/process/state/runnable.rs`
  **Description:** The exec model takes `selected_idx` as an oracle parameter and assumes it is the earliest admission-time thread. The original code computes this index with a loop, so the core scheduling selection logic is not verified and equivalence holds only under a caller-supplied assumption.
  **Suggested Fix:** Implement the selection loop in Verus (or add a verified helper that computes the minimum index from `ready_admission_times`) so `run()` derives the chosen index rather than taking it as input.

- **Location:** `RunnableProcess::wakeup` (exec) in `verus/split/kernel/pm/process/state/runnable.rs`
  **Description:** The exec model uses `found`/`found_idx` oracle parameters instead of verifying the search/removal logic from the sleeping list. This shifts correctness of the search path to the caller and weakens equivalence to the original implementation.
  **Suggested Fix:** Encode the search and removal logic in the exec function (or a verified helper) so the proof establishes the same behavior as `NonEmptyVecDeque::remove_if` without relying on oracle inputs.

- **Location:** `RunnableProcess::terminate` (exec) in `verus/split/kernel/pm/process/state/runnable.rs`
  **Description:** Branching is driven by the `has_interrupted` oracle parameter rather than derived from the actual interrupted/sleeping lists. The original code computes this by inspecting the lists, so the verified version does not prove the real decision logic.
  **Suggested Fix:** Remove the oracle parameter and compute the branch from `spec_interrupted_count()` and `spec_sleeping_count()` inside the exec function.

- **Location:** `RunnableProcess::state`, `state_mut`, `find_thread`, `find_thread_mut`, `earliest_admission_time` (original) vs. Verus split files
  **Description:** These functions do not have exec-level verified counterparts. `find_thread` is only modeled as a spec function; `find_thread_mut`, `state`, `state_mut`, and `earliest_admission_time` have no verified exec model at all, violating the coverage requirement.
  **Suggested Fix:** Add verified exec wrappers (even if modeled as trivial accessors) and link them to specs; for `find_thread{_mut}` and `earliest_admission_time`, either implement verifiable exec logic or provide a verified refinement to the spec-only models.

### Medium
- **Location:** `RunnableProcess::wf` in `verus/split/kernel/pm/process/state/runnable.spec.rs`
  **Description:** The well-formedness invariant does not enforce disjointness/uniqueness of thread IDs across lists. This is a key safety property (a thread should not be in multiple states) and is only documented as a trust assumption, weakening safety proofs.
  **Suggested Fix:** Extend `wf()` with pairwise-disjointness invariants for thread ID sets, and prove preservation across run/terminate/wakeup/add_thread.

- **Location:** Thread list modeling in `runnable.spec.rs`
  **Description:** `Option<NonEmptyVecDeque<T>>` is modeled as `Seq<int>` without representing the None vs. Some(non-empty) distinction. This permits empty sequences where the concrete type would be `None`, which can admit behaviors not representable in the real code.
  **Suggested Fix:** Model optional lists as `Option<Seq<int>>` with a non-empty invariant on `Some`, or add a predicate tying empty sequences to `None`-like behavior to preserve equivalence.

### Low
- **Location:** `RunnableProcess::run` (exec) in `verus/split/kernel/pm/process/state/runnable.rs`
  **Description:** The interrupt reason, context pointer, and user TDA returned by the original `run()` are omitted from the verified model. This leaves side effects of the scheduling decision outside the proof surface.
  **Suggested Fix:** Add abstract return values (or ghost fields) with minimal but explicit postconditions linking them to the selected thread, or document why these outputs are out-of-scope for this module’s correctness goals.

- **Location:** Boundary model `RunningProcess::wf` in `runnable.spec.rs`
  **Description:** The boundary wf() is trivial (`true`), so the proof does not enforce any structural invariants on the RunningProcess output. This weakens end-to-end safety until the RunningProcess module is linked.
  **Suggested Fix:** Strengthen the boundary wf() with minimal invariants (e.g., running thread is not in ready list; PID consistency), or add cross-module refinement checks once RunningProcess is verified.

## Positive Observations
- The split cleanly separates exec, spec, and proof files and documents the abstraction model and trust boundaries.
- Key collection-level behaviors (PID preservation, list content preservation, size changes) are specified precisely for run/terminate/wakeup/add_thread.
- The proof file includes helpful lemmas for sequence manipulation and minimum existence, supporting reasoning about admission times.

## Summary
The verification captures several important invariants and transitions, but core scheduling/search logic is left to oracle parameters and several original functions lack exec-level counterparts. Strengthening coverage and removing oracles would materially improve equivalence and soundness, while adding disjointness invariants would close a key safety gap.
