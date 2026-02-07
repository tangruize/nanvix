# Review: semaphore (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `down_or_block` / `up` (exec, `verus/split/kernel/pm/sync/semaphore.rs`).
  **Description:** The verified model assumes `Condvar::wait()` and `notify_first()` always succeed and omits the `SleepError`/`Error` outcomes of the original API. This makes the specification weaker than the real behavior and breaks equivalence for error paths.
  **Suggested Fix:** Model these functions as `Result`-returning in the verified API (or include an error outcome in `DownOutcome`), add postconditions for failure cases, and connect them to condvar specs that allow failure or require explicit preconditions justifying success.
- **Location:** `Semaphore` model and “Refinement Argument” (exec, `verus/split/kernel/pm/sync/semaphore.rs`).
  **Description:** The verified model is purely sequential (`&mut self`, plain `usize`) while the original uses `AtomicUsize` with `SeqCst` and a `Condvar`. The refinement to the concurrent implementation is only described informally; no formal linearizability or atomicity proof is provided.
  **Suggested Fix:** Add a refinement layer or linearization-point proof that connects the sequential model to `fetch_update`/`fetch_add` under `SeqCst`, or introduce a verified atomic abstraction and prove the semaphore against it.

### Medium
- **Location:** `up` precondition (exec, `verus/split/kernel/pm/sync/semaphore.rs`).
  **Description:** The verified `up()` requires `old(self).value < usize::MAX`, which is stronger than the original implementation (which can wrap on overflow). This spec may be too strong without a proven global invariant that bounds semaphore counts.
  **Suggested Fix:** Prove a system-level invariant that the semaphore value is always bounded, or model wrapping behavior (or return an error) so the verified spec matches the runtime semantics.
- **Location:** `SemaphoreView`/`View::view` and blocking lemmas (spec/proof, `semaphore.spec.rs` and `semaphore.proof.rs`).
  **Description:** The view always sets `waiters = 0` and exec functions never update ghost waiters. Blocking protocol lemmas reason over manually constructed views, so the waiters-based invariants and liveness claims are disconnected from exec state.
  **Suggested Fix:** Thread a ghost view or tracked waiter count through exec functions and add ensures clauses that update it on the `WouldBlock` path and on wake, or connect it to a shared condvar spec.
- **Location:** `down_or_block` and `lemma_all_waiters_eventually_served` (exec/proof).
  **Description:** Liveness is not proven for real executions: `down_or_block` only models a single decision and the “eventually served” lemma is spec-only and not tied to exec transitions or fairness assumptions.
  **Suggested Fix:** Introduce a progress model (e.g., a fairness assumption plus a ghost transition relation) and connect `down_or_block`/`up` to it, or clearly mark the property as unproven in the spec and remove it from the verified claims.
- **Location:** `spec_condvar_wake_after_notify` (spec, `semaphore.spec.rs`).
  **Description:** The condvar interface assumption is local to the semaphore spec and not imported or enforced by the condvar module, so cross-module changes won’t be detected.
  **Suggested Fix:** Define a shared condvar interface spec in a common module and make both the semaphore and condvar proofs depend on it.

### Low
- None.

## Positive Observations
- All public APIs in the original module (`new`, `down`, `try_down`, `up`) have verified counterparts, with clear API mapping and explicit divergence notes.
- Core safety properties (resource conservation, mutual exclusion for binary semaphores, monotonicity) are stated and proven for the sequential model.
- Spec/proof separation is clean: exec code includes spec and proof files without mixing proof bodies into the exec functions.

## Summary
The verification provides a solid sequential state-machine model of the semaphore but falls short on error handling, concurrency refinement, and executable liveness guarantees. Strengthening the linkage between ghost waiters and exec behavior and formalizing the refinement to atomic/condvar semantics would materially improve coverage and equivalence.
