# Review: runnable (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `RunnableProcess::state`, `RunnableProcess::state_mut`, `RunnableProcess::find_thread`, `RunnableProcess::find_thread_mut`, `RunnableProcess::earliest_admission_time` (exec/spec coverage notes in `runnable.spec.rs`).
  **Description:** These original functions have no exec-level verified counterparts; `find_thread*` and `earliest_admission_time` are only modeled as spec-only helpers, and `state/state_mut` are omitted entirely. This violates the coverage requirement and leaves correctness obligations (e.g., reference semantics and ProcessState invariants) unverified.
  **Suggested Fix:** Add verified wrappers or refinement lemmas that connect exec behavior to specs (even if using `external_body` with strong postconditions), or model these operations with ghost-return types and prove they satisfy the same search/immutability properties as the original.

- **Location:** `RunnableProcess::new/from_state/add_thread/wakeup` (exec `runnable.rs`, spec `RunnableProcessView`).
  **Description:** Admission times are supplied as unconstrained ghost inputs (`ready_time`/`new_ready_time`) with no linkage to actual `ReadyThread::admission_time()` or `SleepingThread::wakeup()` semantics. As a result, `run()` proving “earliest admission time” only holds for the ghost sequence, not for the real kernel threads, so semantic equivalence of scheduling is not established.
  **Suggested Fix:** Introduce a verified spec for `ReadyThread`/`SleepingThread` that exposes admission time, and add invariants tying `ready_admission_times` to the concrete thread list (or model ready threads as a sequence of records containing both ID and admission time).

### Medium
- **Location:** `RunnableProcess::wf` (spec `runnable.spec.rs`).
  **Description:** The invariant does not enforce uniqueness or disjointness of thread IDs across ready/interrupted/sleeping/zombie lists, even though the correctness of `run/terminate/wakeup` assumes exclusive ownership. This permits models with duplicate IDs that are impossible in the real Rust implementation, weakening safety arguments.
  **Suggested Fix:** Include `spec_ids_disjoint()` in `wf()` (or add it as a required precondition for public operations) and propagate proofs that transitions preserve disjointness.

- **Location:** `clock_now()` external body (exec `runnable.rs`) and `EXIT_STATUS_INTERRUPTED()` constant (spec `runnable.spec.rs`).
  **Description:** `clock_now()` is an `external_body` and `EXIT_STATUS_INTERRUPTED()` is a hard-coded value with only a TODO to link to the real errno value. These are trust assumptions in a core module; if the clock can return negative values or the errno value changes, proofs still pass but behavior diverges.
  **Suggested Fix:** Replace the constant with a shared verified definition from the sys error module, and add a proof/lemma tying `clock_now()` to the real `SystemTime` specification (or make it a verified wrapper around the HAL clock).

### Low
- **Location:** Boundary models (`RunningProcess`, `InterruptedProcess`, `ZombieProcess`) in `runnable.rs`/`runnable.spec.rs`.
  **Description:** Boundary types are modeled with minimal `wf()` predicates and no linking proof to their concrete modules, so cross-module equivalence is assumed rather than proven.
  **Suggested Fix:** Add linking lemmas that show the boundary models’ postconditions follow from the verified specs of the real modules once those are available.

- **Location:** Global properties (module-level).
  **Description:** Liveness/fairness properties beyond “select minimum admission time” are not specified (e.g., eventual scheduling of all ready threads). Safety is the focus, but liveness expectations are not captured.
  **Suggested Fix:** If liveness is required, add higher-level temporal or progress invariants in a system-level proof that uses `RunnableProcess` as a component.

## Positive Observations
- `run()` selection is proved via `spec_min_index_rec` and `lemma_earliest_ready_index_bounds`, avoiding oracle indices.
- `terminate()` and `wakeup()` have strong content-level postconditions that preserve PID and list contents, and `wf()` is preserved throughout.
- Spec/proof separation is clean: exec code is concise, with detailed specs and lemmas isolated in `*.spec.rs` and `*.proof.rs`.

## Summary
The verification is solid for the modeled transitions, but it omits several original functions and relies on unconstrained ghost admission times, which weakens equivalence with the concrete scheduler. Strengthening coverage (especially `state*` and `find_thread*`) and tying ghost timestamps to real thread data would significantly improve the fidelity and soundness of the proof.
