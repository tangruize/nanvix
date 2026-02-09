# Review: runnable (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `RunnableProcess::state`, `RunnableProcess::state_mut`, `RunnableProcess::find_thread`, `RunnableProcess::find_thread_mut`, `RunnableProcess::earliest_admission_time` (exec/spec coverage notes in `runnable.spec.rs`).
  **Description:** These functions remain omitted from the exec-level model, with only spec-only helpers. This is unchanged and still violates coverage and leaves the reference-based APIs unverified.
  **Suggested Fix:** Provide verified wrappers/refinements or `external_body` stubs with strong postconditions that mirror the original functions and connect to the spec model.

- **Location:** `RunnableProcess::new/from_state/add_thread/wakeup` (exec `runnable.rs`, spec `RunnableProcessView`).
  **Description:** Admission times are still unconstrained ghost inputs with no invariant tying them to concrete `ReadyThread`/`SleepingThread` admission times. `run()` therefore proves correctness only for the ghost sequence and not the real scheduling policy.
  **Suggested Fix:** Verify admission-time accessors and relate `ready_admission_times` to the concrete thread list, or model ready threads as (id, admission_time) records with invariants.

### Medium
- **Location:** `RunnableProcess::wf` (spec `runnable.spec.rs`).
  **Description:** Disjointness/uniqueness of thread IDs across lists is still not part of `wf()`, remaining a trust assumption. This admits impossible states and weakens safety reasoning.
  **Suggested Fix:** Add `spec_ids_disjoint()` to `wf()` (or require it in public preconditions) and prove it is preserved by transitions.

- **Location:** `clock_now()` external body and `EXIT_STATUS_INTERRUPTED()` constant (exec/spec).
  **Description:** `clock_now()` remains `external_body` and the exit status constant is still hard-coded with only a TODO for linkage. This leaves a trust gap in core scheduling behavior.
  **Suggested Fix:** Link to the verified sys error definition and provide a verified clock wrapper or explicit HAL spec for `clock_now()`.

### Low
- **Location:** Boundary models (`RunningProcess`, `InterruptedProcess`, `ZombieProcess`).
  **Description:** Boundary `wf()` predicates and linking lemmas are still minimal with no proof connection to concrete modules, so equivalence remains assumed.
  **Suggested Fix:** Add cross-module linking lemmas once sibling modules are verified.

## Positive Observations
- No new regressions or unsound assumptions were introduced in these files.
- Existing proofs for list content preservation and min-index selection remain intact.

## Summary
The updated files do not show substantive fixes to the prior review items; the same coverage and equivalence gaps remain. Verification is still partial and relies on multiple trust assumptions, so it is not yet complete or sound relative to the original implementation.
