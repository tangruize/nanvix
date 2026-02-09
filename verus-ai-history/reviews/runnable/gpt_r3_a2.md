# Review: runnable (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `RunnableProcess::state`, `RunnableProcess::state_mut`, `RunnableProcess::find_thread`, `RunnableProcess::find_thread_mut`, `RunnableProcess::earliest_admission_time` (exec/spec coverage notes in `runnable.spec.rs`).
  **Description:** These functions are still explicitly omitted from the exec-level model, and only spec-only helpers exist. This is unchanged from the prior review and still violates coverage and leaves correctness of reference-based accessors unverified.
  **Suggested Fix:** Add verified wrappers/refinements that connect these functions to the spec model (even via abstracted ghost return types) or add verified `external_body` stubs with strong postconditions that match the original behavior.

- **Location:** `RunnableProcess::new/from_state/add_thread/wakeup` (exec `runnable.rs`, spec `RunnableProcessView`).
  **Description:** Admission times remain unconstrained ghost inputs with no linkage to concrete thread admission times. `run()` thus proves correctness only for the ghost sequence and does not establish semantic equivalence to the real scheduler selection.
  **Suggested Fix:** Verify `ReadyThread`/`SleepingThread` admission-time accessors and tie `ready_admission_times` to the concrete thread list, or model ready threads as records containing both ID and admission time with corresponding invariants.

### Medium
- **Location:** `RunnableProcess::wf` (spec `runnable.spec.rs`).
  **Description:** `wf()` still does not enforce disjointness/uniqueness of thread IDs across lists; it is documented as a trust assumption. This allows impossible states and weakens safety arguments.
  **Suggested Fix:** Integrate `spec_ids_disjoint()` into `wf()` (or require it in public preconditions) and prove it is preserved by transitions.

- **Location:** `clock_now()` external body and `EXIT_STATUS_INTERRUPTED()` constant (exec/spec).
  **Description:** `clock_now()` remains `external_body` and the exit status is still a hard-coded constant with only a TODO for linkage. This leaves a trust gap in a core module and can silently diverge from kernel semantics.
  **Suggested Fix:** Tie `EXIT_STATUS_INTERRUPTED()` to the verified sys error definition and replace `clock_now()` with a verified wrapper or explicit assumptions justified by a HAL spec.

### Low
- **Location:** Boundary models (`RunningProcess`, `InterruptedProcess`, `ZombieProcess`).
  **Description:** Boundary `wf()` predicates and linking lemmas are still minimal; no proof connects these boundary models to the concrete modules, so equivalence is assumed rather than verified.
  **Suggested Fix:** Add cross-module linking lemmas once the sibling modules are verified.

## Positive Observations
- No new unsound assumptions were introduced; the proof structure remains clean and consistent.
- The earlier verified properties (e.g., min-index selection and list content preservation) remain intact.

## Summary
No substantive fixes were observed; the same coverage gaps and semantic mismatches remain. The verification is still partial and relies on trust assumptions for key behaviors, so the module cannot yet be considered complete or sound relative to the original implementation.
