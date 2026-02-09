# Review: runnable (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `RunnableProcess::terminate` (exec) in `verus/split/kernel/pm/process/state/runnable.rs`
  **Description:** The branch decision is still driven by the `has_interrupted` oracle parameter. This means the verified code does not derive the decision from actual list contents as the original does, so equivalence hinges on a caller-supplied boolean.
  **Suggested Fix:** Compute the branch internally by encoding counts at exec level (e.g., store explicit exec counters or use a verified loop over ghost data with an exec flag) and remove the oracle parameter.

- **Location:** `RunnableProcess::wakeup` (exec) in `verus/split/kernel/pm/process/state/runnable.rs`
  **Description:** The index oracle was removed, but `found: bool` remains an oracle parameter. The original function computes this by searching the sleeping list; here the caller must provide the correct value for equivalence to hold.
  **Suggested Fix:** Implement a verified search loop that produces an exec boolean and (if found) an index, eliminating the oracle parameter.

- **Location:** `state`, `state_mut`, `find_thread`, `find_thread_mut`, `earliest_admission_time` (original) vs. Verus split files
  **Description:** These functions still lack exec-level verified counterparts. They are only documented as spec-only or elided, which violates the coverage requirement.
  **Suggested Fix:** Add verified exec wrappers (even if modeled as trivial accessors) that refine to the existing specs, or provide explicit refinement lemmas proving equivalence to the spec-only models.

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
- `run()` now derives the minimum index internally via `lemma_earliest_ready_index_bounds`, removing the previous index oracle and aligning with the original loop’s behavior.
- `wakeup()` no longer takes an index oracle and uses proof-level `choose` to derive a valid index when `found` is true.
- The spec now includes a recursive min-index definition with proof of bounds/minimality, improving rigor for admission-time selection.

## Summary
The prover addressed the `run()` oracle and partially improved `wakeup()` by removing the index oracle, but the boolean oracle in `wakeup()` and the `terminate()` oracle remain, leaving key behavioral decisions outside the verified implementation. Coverage gaps (state accessors, find_thread, earliest_admission_time) and missing disjointness invariants persist, so the verification is still incomplete and not fully equivalent to the original.
