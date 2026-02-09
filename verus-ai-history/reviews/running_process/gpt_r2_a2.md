# Review: running_process (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `try_join_thread` (exec: `verus/split/kernel/pm/process/state/running.rs`).
  **Description:** The previous external_body was removed, but the function now takes an oracle parameter `tag` and simply returns it. This shifts the core join logic to the caller and is not semantically equivalent to the original API (which computes the result by searching thread lists and returns either a zombie thread or condvar/error). The verification still relies on a trusted precondition rather than proving the search/decision logic.
  **Suggested Fix:** Implement the search over ghost sequences inside `try_join_thread` (as was done for `wakeup`) and compute the tag internally, or provide a verified model of `NonEmptyVecDeque::remove_if()` and use it to derive the outcome without caller-provided oracles.

- **Location:** `find_thread`, `find_thread_mut` (exec: `verus/split/kernel/pm/process/state/running.rs`).
  **Description:** These functions now accept an oracle `found_in: Option<u8>` and return that value wrapped, instead of computing the search result. This is not equivalent to the original API and still leaves the search correctness unverified.
  **Suggested Fix:** Compute the list variant internally from the ghost sequences (using `spec_find_thread` logic in exec form) and eliminate the oracle parameter.

- **Location:** `wakeup` (exec: `verus/split/kernel/pm/process/state/running.rs`).
  **Description:** The oracle parameter `found: bool` remains, so correctness depends on callers supplying the right value. This is a semantic mismatch with the original code, which performs the search internally.
  **Suggested Fix:** Remove the oracle parameter and derive the presence/absence of the thread internally from the ghost sequence.

### Medium
- **Location:** `wf()` (spec: `running.spec.rs`).
  **Description:** The invariant still omits thread-ID uniqueness/disjointness across lists, leaving safety properties (no double-queuing, unambiguous join target) unproven. The `wf_strict()` predicate exists but is not required by public methods.
  **Suggested Fix:** Strengthen `wf()` to include `wf_strict()` or require `wf_strict()` for all public operations that depend on exclusive list membership.

- **Location:** `state`, `state_mut`, `running_mut` (exec: `verus/split/kernel/pm/process/state/running.rs`).
  **Description:** These remain `external_body`, so the frame conditions and PID/thread-ID preservation are assumed rather than proven within this module.
  **Suggested Fix:** Provide verified wrappers or discharge the assumptions by linking to verified `ProcessState`/`RunningThread` modules.

### Low
- **Location:** `interrupted_resume` (exec: `verus/split/kernel/pm/process/state/running.rs`).
  **Description:** Still `external_body` with a strong spec (exact element preservation). Soundness depends on a sibling module proof not shown here.
  **Suggested Fix:** Prove the corresponding properties in the InterruptedProcess module and replace this with a verified call.

## Positive Observations
- The core state-machine transitions (`schedule`, `sleep`, `exit`, `exit_thread`) remain well specified with detailed branch conditions.
- The `try_join_thread` implementation now enforces the zombie removal post-state when the oracle indicates success, tightening the mutation effects compared to the previous external_body.

## Summary
Several previously reported issues remain unresolved: the verification still relies on oracle parameters for `try_join_thread`, `find_thread(_mut)`, and `wakeup`, so core search/join behavior is not actually proven and the exec API diverges from the original. Invariants remain too weak to guarantee exclusivity of thread membership, and multiple external_body functions persist. Overall, the verification is improved in documentation and mutation postconditions, but it is still not fully sound or equivalent to the original code.
