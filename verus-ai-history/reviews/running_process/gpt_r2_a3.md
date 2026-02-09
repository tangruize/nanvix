# Review: running_process (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `try_join_thread` (exec: `verus/split/kernel/pm/process/state/running.rs`).
  **Description:** The previous external_body is gone, but the function still depends on an oracle `tag` parameter to choose the branch and define the result. This is not semantically equivalent to the original API (which performs the search internally) and the core join logic remains unverified. The precondition only shifts correctness to the caller.
  **Suggested Fix:** Compute the join outcome internally over the ghost sequences (e.g., by a verified search/`remove_if` model) and eliminate the oracle parameter.

- **Location:** `wakeup` (exec: `verus/split/kernel/pm/process/state/running.rs`).
  **Description:** The oracle parameter `found: bool` remains. The function’s behavior still depends on caller-supplied truth instead of internal search, so equivalence to the original is not established.
  **Suggested Fix:** Remove the oracle parameter and derive membership using a verified search over the ghost sequence (similar to how the exec model now removes by index once found).

### Medium
- **Location:** `wf()` (spec: `running.spec.rs`).
  **Description:** The invariant is unchanged and still does not enforce uniqueness/disjointness of thread IDs across queues. This leaves safety properties (no double-queuing, unambiguous join target) unproven.
  **Suggested Fix:** Strengthen `wf()` to include the `wf_strict()` disjointness conditions, or require `wf_strict()` for all public operations.

- **Location:** `state`, `state_mut`, `running_mut` (exec: `verus/split/kernel/pm/process/state/running.rs`).
  **Description:** These remain `external_body`; PID/thread-ID preservation is assumed rather than proven within this module.
  **Suggested Fix:** Provide verified wrappers or link to verified `ProcessState`/`RunningThread` modules to discharge the frame-condition obligations.

### Low
- **Location:** `interrupted_resume` (exec: `verus/split/kernel/pm/process/state/running.rs`).
  **Description:** Still `external_body` with a strong element-preservation spec; soundness depends on sibling proofs not shown here.
  **Suggested Fix:** Replace with a verified call or cite lemmas from the InterruptedProcess module that prove the same properties.

## Positive Observations
- The oracle parameters for `find_thread`/`find_thread_mut` were removed; these now return the spec-computed variant directly, which is an improvement over the previous caller-supplied oracle.
- The core state-machine transitions (`schedule`, `sleep`, `exit`, `exit_thread`) remain precisely specified and consistent with the source.

## Summary
The update fixes the `find_thread` oracle issue, but major soundness and equivalence gaps remain: `try_join_thread` and `wakeup` still rely on oracle parameters, and invariants/external bodies are unchanged. Verification is improved but still incomplete and not fully sound with respect to the original behavior.
