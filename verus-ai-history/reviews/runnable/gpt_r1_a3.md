# Review: runnable (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Location:** `RunnableProcess::wakeup` (exec) in `verus/split/kernel/pm/process/state/runnable.rs`
  **Description:** The boolean `found` remains an oracle parameter. The original implementation computes this by searching the sleeping list, but the verified model still relies on caller-provided truth, so equivalence depends on an external assumption.
  **Suggested Fix:** Introduce an exec-level search over an exec representation (or encode a verified loop over ghost data coupled with an exec flag) so `found` is derived internally and the oracle can be removed.

- **Location:** `state`, `state_mut`, `find_thread`, `find_thread_mut`, `earliest_admission_time` (original) vs. Verus split files
  **Description:** These functions still have no exec-level verified counterparts. They are only documented as spec-only or elided, which does not satisfy the coverage requirement.
  **Suggested Fix:** Add exec wrappers (even if trivial) that refine to the existing specs, or provide explicit refinement lemmas showing equivalence to the spec-only models.

### Medium
- **Location:** `RunnableProcess::wf` in `verus/split/kernel/pm/process/state/runnable.spec.rs`
  **Description:** Thread ID disjointness across ready/interrupted/sleeping/zombie lists is still a trust assumption rather than an invariant. This weakens safety guarantees about thread ownership/state exclusivity.
  **Suggested Fix:** Add pairwise-disjointness invariants to `wf()` and prove preservation across run/terminate/wakeup/add_thread.

### Low
- **Location:** `RunnableProcess::run` return values (spec/exec)
  **Description:** The interrupt reason, context pointer, and user TDA from the original `run()` remain omitted. This leaves part of the observable behavior outside the verified surface.
  **Suggested Fix:** Add abstract/ghost return values with minimal postconditions tying them to the selected thread, or explicitly justify why these outputs are irrelevant for this module’s correctness.

- **Location:** Boundary model `RunningProcess::wf` in `runnable.spec.rs`
  **Description:** The boundary wf() is still trivial (`true`), so the proof does not enforce any structural invariants on the RunningProcess output.
  **Suggested Fix:** Strengthen the boundary wf() with minimal invariants (e.g., PID consistency, running thread not in ready list) or add cross-module refinement checks once RunningProcess is verified.

## Positive Observations
- The `terminate()` oracle was removed; the branch now uses exec-level counters tied to ghost lengths via `wf()`.
- `run()` continues to derive the minimum index internally via `lemma_earliest_ready_index_bounds`.
- The spec formalizes an isomorphism between optional non-empty lists and empty/non-empty sequences, clarifying equivalence.

## Summary
The termination decision is now internally derived, which is a clear improvement, but the wakeup boolean oracle and coverage gaps persist. The verification is stronger than before but still incomplete and not fully equivalent to the original implementation.
