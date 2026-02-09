# Review: running_process (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `try_join_thread`, `find_thread`, `find_thread_mut` (exec: `verus/split/kernel/pm/process/state/running.rs`).
  **Description:** These core functions are `#[verifier::external_body]`, so their behavior is assumed rather than proved. This leaves the core thread-search/join logic unverified even though it directly impacts safety (join semantics, zombie removal, list membership). It also violates the “no unjustified external_body in core module” criterion.
  **Suggested Fix:** Replace `external_body` with executable Verus implementations over the ghost sequences (similar to `wakeup`) and prove the specs, or reduce the surface by verifying wrappers around a modeled `NonEmptyVecDeque` API.

- **Location:** `wakeup` (exec: `verus/split/kernel/pm/process/state/running.rs`).
  **Description:** The verified API adds a `found: bool` oracle parameter not present in the original function, shifting the search responsibility to the caller. Correctness now depends on a trusted precondition tying `found` to ghost state; this is a semantic mismatch if callers can pass the wrong value.
  **Suggested Fix:** Model the search internally (as in the original) using a ghost sequence and `choose` to derive the index, removing the oracle parameter from the exec signature.

### Medium
- **Location:** `wf()` and use sites (spec: `running.spec.rs`; exec/proof: `running.rs`, `running.proof.rs`).
  **Description:** The default invariant only checks count/length equality and does **not** enforce thread ID uniqueness or exclusivity across lists. This allows a thread to appear in multiple queues, weakening safety properties (e.g., join ambiguity, double scheduling) that are guaranteed by Rust ownership in the original.
  **Suggested Fix:** Strengthen `wf()` to include the `wf_strict()` disjointness conditions, or require `wf_strict()` for public methods that assume exclusivity.

- **Location:** `state`, `state_mut`, `running_mut` (exec: `verus/split/kernel/pm/process/state/running.rs`).
  **Description:** These are `external_body` and allow arbitrary mutation while only specifying a frame condition. The preservation of PID and thread identity is assumed rather than enforced, leaving a soundness gap in this module.
  **Suggested Fix:** Provide executable wrappers with explicit post-state reconstruction (or verified adapters) to ensure the frame conditions hold, or discharge the assumptions by verifying `ProcessState` and `RunningThread` and linking them here.

### Low
- **Location:** `interrupted_resume` (exec: `verus/split/kernel/pm/process/state/running.rs`).
  **Description:** The behavior of `InterruptedProcess::resume()` is assumed with a strong spec (exact element preservation). If the sibling module diverges, this module’s proofs become unsound.
  **Suggested Fix:** Prove the corresponding properties in the InterruptedProcess module and replace this `external_body` with a verified call, or reference a verified lemma from that module.

## Positive Observations
- All original functions are represented in the verified split (coverage is complete).
- Specs document key state-machine behaviors and include detailed branch conditions for `schedule`, `sleep`, `exit`, and `exit_thread`.
- The verification identified and documented a real bug in the original `exit_thread()` logic (zombie list loss) and aligns the model with the fixed source.
- Spec/proof separation is clean and well-structured, with explicit view types and lemmas.

## Summary
The verification is well-documented and covers the main state-machine transitions, but several core behaviors remain assumed rather than proved. The strongest gaps are the `external_body` functions for join/search logic and the oracle parameter in `wakeup`, which weaken equivalence and soundness. Strengthening invariants (uniqueness across queues) and discharging trusted bodies would significantly improve the assurance level.
