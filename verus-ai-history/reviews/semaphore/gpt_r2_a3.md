# Review: semaphore (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `down_or_block` / `up` and T5/T6 trust assumptions (exec, `verus/split/kernel/pm/sync/semaphore.rs`).
  **Description:** The model still omits the `Condvar::wait()` / `notify_first()` failure outcomes and the original `SleepError`/`Error` results. The updates only restate assumptions; no failure cases or success preconditions are modeled, so the verified API remains weaker than runtime semantics.
  **Suggested Fix:** Model error outcomes explicitly (e.g., extend `DownOutcome`/`up` return type), or add explicit preconditions/ghost obligations that guarantee condvar success and prove those obligations from caller context or condvar specs.
- **Location:** Refinement argument (exec, `verus/split/kernel/pm/sync/semaphore.rs`).
  **Description:** The sequential `&mut self` model is still not formally linked to the atomic/condvar implementation; the refinement section remains informal narrative without a linearizability/atomicity proof.
  **Suggested Fix:** Add a refinement layer or atomic abstraction with a proof that `fetch_update`/`fetch_add` and the condvar protocol refine the sequential spec under `SeqCst`.

### Medium
- **Location:** `up` precondition (exec, `verus/split/kernel/pm/sync/semaphore.rs`).
  **Description:** The `value < usize::MAX` precondition is still stronger than runtime behavior. The bounded-pool lemma is not tied to a system invariant or caller proof, so the precondition remains unproven in practice.
  **Suggested Fix:** Prove or assume a global bound in caller modules, or model wrapping/return an error to match runtime semantics.
- **Location:** View/waiters and blocking protocol (spec/exec, `semaphore.spec.rs`, `semaphore.rs`).
  **Description:** The exec view still fixes `waiters = 0`, and exec functions do not update ghost waiters. The new postcondition only ties the *Acquired* path to `spec_down_or_block_ghost_view` (which leaves waiters unchanged); the *WouldBlock* path still leaves `self@` unchanged, so waiter tracking remains disconnected from exec state.
  **Suggested Fix:** Thread a tracked ghost waiter count through exec operations (with ensures that update it on WouldBlock/wake) or tie the ghost view to the condvar spec state.
- **Location:** Condvar interface assumption (spec, `semaphore.spec.rs`).
  **Description:** `spec_condvar_wake_after_notify` remains a local assumption not imported by the condvar module, so cross-module changes will not be checked against this contract.
  **Suggested Fix:** Move this assumption to a shared condvar interface spec and make both semaphore and condvar proofs depend on it.

### Low
- None.

## Positive Observations
- Added a postcondition linking the Acquired path of `down_or_block` to the ghost-view helper, though it does not address waiter tracking.
- Documentation still clearly states scope limits and trust assumptions.

## Summary
The core verification gaps from the previous review remain; changes are largely additive but do not resolve error semantics, concurrency refinement, waiter tracking, or shared condvar contracts. Verification remains a sequential model with significant trust assumptions and no formal refinement to runtime behavior.
