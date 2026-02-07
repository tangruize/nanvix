# Review: semaphore (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `down_or_block` / `up` and T5/T6 trust assumptions (exec, `verus/split/kernel/pm/sync/semaphore.rs`).
  **Description:** The model still omits the `Condvar::wait()` / `notify_first()` failure outcomes and the original `SleepError`/`Error` results. The changes document these as assumptions but do not model failures or add preconditions that justify success, so the verified API remains weaker than the runtime semantics.
  **Suggested Fix:** Model error outcomes explicitly (e.g., extend `DownOutcome`/`up` return type), or add explicit preconditions/ghost obligations that guarantee condvar success and prove those obligations from caller context or condvar specs.
- **Location:** Refinement argument (exec, `verus/split/kernel/pm/sync/semaphore.rs`).
  **Description:** The sequential `&mut self` model is still not formally linked to the atomic/condvar implementation; the new refinement section is informative but remains an informal narrative with no linearizability/atomicity proof.
  **Suggested Fix:** Add a refinement layer or atomic abstraction with a proof that `fetch_update`/`fetch_add` and the condvar protocol refine the sequential spec under `SeqCst`.

### Medium
- **Location:** `up` precondition (exec, `verus/split/kernel/pm/sync/semaphore.rs`).
  **Description:** The `value < usize::MAX` precondition is still stronger than runtime behavior. The new lemma shows bounded pools suffice but does not connect to a system-level invariant or caller proof, leaving the precondition unproven in practice.
  **Suggested Fix:** Prove or assume a global bound in caller modules, or model wrapping/return an error to match runtime semantics.
- **Location:** View/waiters and blocking protocol (spec/exec, `semaphore.spec.rs`, `semaphore.rs`).
  **Description:** The exec view still fixes `waiters = 0`, and exec functions do not update ghost waiters. The new `spec_down_or_block_ghost_view` helper is not tied to exec postconditions, so blocking protocol lemmas remain disconnected from actual exec state.
  **Suggested Fix:** Thread a tracked ghost waiter count through exec operations (with ensures that update it) or tie the ghost view to the condvar spec state.
- **Location:** Condvar interface assumption (spec, `semaphore.spec.rs`).
  **Description:** `spec_condvar_wake_after_notify` remains a local assumption not imported by the condvar module, so cross-module changes will not be checked against this contract.
  **Suggested Fix:** Move this assumption to a shared condvar interface spec and make both semaphore and condvar proofs depend on it.

### Low
- None.

## Positive Observations
- Documentation now clearly states scope limits (no liveness/progress guarantees) and trust assumptions.
- Added ghost-view helper and mapping lemmas improve structure for blocking semantics.
- Exec/spec/proof separation remains clean and consistent.

## Summary
Most changes are documentation or auxiliary lemmas; the core verification gaps (error outcomes, concurrency refinement, ghost waiters linkage, and shared condvar contract) remain. Verification is still a sequential model with significant trust assumptions and no formal refinement to runtime behavior.
