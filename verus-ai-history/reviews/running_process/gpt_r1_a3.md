# Review: running_process (gpt-5.2-codex)

## Grade: C+

## Issues Found

### Critical
- None.

### High
- **`wakeup()` still relies on an oracle parameter**
  - **Location:** `RunningProcess::wakeup` (exec), `verus/split/kernel/pm/process/state/running.rs`.
  - **Description:** The `found: bool` parameter remains and is not part of the original API. Correctness is shifted to callers via a precondition, which is a soundness gap and breaks equivalence.
  - **Suggested Fix:** Remove the oracle from the public interface and compute the branch internally from ghost state (or provide a verified wrapper that enforces `found == spec_seq_contains(...)`).

- **Known divergence in `exit_thread()` persists**
  - **Location:** `RunningProcess::exit_thread` (exec), `verus/split/kernel/pm/process/state/running.rs`.
  - **Description:** The model still passes `new_zombie_ids` into the interrupted path, while the original passes `self.zombie.take()` (always `None` after line 261). This is a semantic mismatch and fails equivalence.
  - **Suggested Fix:** Fix the original source and re-verify, or change the model to match the original behavior and document the bug separately.

### Medium
- **`try_join_thread()` is still trusted (`external_body`) though it is modelable**
  - **Location:** `RunningProcess::try_join_thread` (exec), `verus/split/kernel/pm/process/state/running.rs`.
  - **Description:** This function now exists but remains `external_body`. It mutates the zombie list and could be modeled directly like `wakeup()`. Relying on a stub in this core module weakens soundness (criterion 3).
  - **Suggested Fix:** Implement `try_join_thread()` directly in Verus using ghost sequences (remove a zombie if present) and prove the spec, avoiding `external_body`.

- **Well-formedness still allows duplicate thread IDs**
  - **Location:** `wf()` in `running.spec.rs`.
  - **Description:** `wf()` only checks count/length equality and does not enforce disjointness or exclude the running thread from lists. This permits models with duplicated IDs, weakening reasoning about `find_thread`/`try_join_thread`/`wakeup`.
  - **Suggested Fix:** Require `wf_strict()` in public APIs or strengthen `wf()` to enforce ID uniqueness and disjointness.

### Low
- **Doc comments still say `try_join_thread()` / `find_thread()` are spec-only**
  - **Location:** Module docs (exec), `verus/split/kernel/pm/process/state/running.rs`.
  - **Description:** The docs say these are spec-only, but exec stubs now exist. This is minor but can mislead reviewers about coverage.
  - **Suggested Fix:** Update documentation to reflect the new external_body stubs and their trust boundaries.

## Positive Observations
- Coverage improved: `state_mut`, `running_mut`, `try_join_thread`, `find_thread`, and `find_thread_mut` are now modeled with explicit specs.
- `exit()`’s runnable branch now specifies the ready thread and interrupted tail precisely, fixing the prior underspecification.
- Frame conditions for mutable accessors are explicit and consistent with PID/TID immutability.

## Summary
Several fixes were real (coverage improvements and stronger `exit()` specs), but key soundness/equivalence gaps remain: the oracle-based `wakeup`, the persistent `exit_thread` divergence, and reliance on `external_body` for `try_join_thread`. The verification is improved but still not complete or fully sound.
