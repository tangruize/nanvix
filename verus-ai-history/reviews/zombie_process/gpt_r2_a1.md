# Review: zombie_process (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `ZombieProcess::find_thread` / `ZombieProcess::find_thread_mut` (exec: `verus/split/kernel/pm/process/state/zombie.rs`, spec: `zombie.spec.rs`).
  **Description:** Both functions are `external_body` and the module explicitly states the iterator-based search is not verified; the integration obligations that link `NonEmptyVecDeque::iter()` and `ZombieThread::id()` to the ghost sequence are left unproven. This leaves the core lookup semantics (and correctness of returned references) as a trust assumption, weakening soundness and equivalence.
  **Suggested Fix:** Provide an integration proof that the iterator order and `id()` predicate match the ghost sequence, or introduce a verified wrapper over `NonEmptyVecDeque`/`ZombieThread` that supplies the required refinement. Alternatively, model `ThreadRef`/`ThreadRefMut` with tracked types and prove the search implementation directly.

- **Location:** `spec_find_thread_mut_caller_obligation` and `find_thread_mut` (spec/proof: `zombie.spec.rs`, `zombie.proof.rs`; exec: `zombie.rs`).
  **Description:** The mutable finder relies on a caller obligation to preserve thread identity and well-formedness, but this obligation is not enforced or discharged in this module. As written, the proof does not guarantee that `find_thread_mut` preserves `wf()` if callers mutate the thread ID or membership.
  **Suggested Fix:** Discharge the obligation at each call site (prove identity preservation), or restrict mutable access to verified methods that preserve ID, or strengthen the API to prevent ID mutation through the returned reference.

### Medium
- **Location:** `ZombieProcess::state` / `ZombieProcess::state_mut` (exec: `zombie.rs`, spec: `zombie.spec.rs`).
  **Description:** The model abstracts `ProcessState` to just PID; other fields (capabilities, vmem, mailbox, etc.) are ignored. If zombie correctness depends on any of these fields (or if callers rely on the returned references for properties beyond PID), the spec is too weak and equivalence is not captured.
  **Suggested Fix:** Extend the ghost model to include the relevant `ProcessState` fields and add frame conditions/postconditions for those fields, or explicitly document and justify that only PID is relevant for all verified uses of `ZombieProcess`.

- **Location:** `ZombieProcess::bury` (exec: `zombie.rs`, spec: `zombie.spec.rs`, proof: `zombie.proof.rs`).
  **Description:** The ghost model only verifies identity preservation (IDs, PID, status) and does not verify ownership/resource transfer semantics of `bury(self)`. The correctness of resource handoff to the parent process is therefore assumed, not proven.
  **Suggested Fix:** Use tracked resources or ownership tokens to model the transferred `ZombieThread` and `ProcessState` objects and prove that `bury()` moves them without aliasing.

- **Location:** `ZombieProcess::new` preconditions and `wf()` (spec: `zombie.spec.rs`, exec: `zombie.rs`).
  **Description:** The spec requires `spec_no_duplicates` and exact `zombie_count`/sequence-length alignment, but the original constructor does not enforce uniqueness or validate count. The argument relies on an external uniqueness guarantee from the thread subsystem, which is not proven here.
  **Suggested Fix:** Either relax the spec to match the constructor behavior and prove properties without uniqueness, or add integration proofs linking the thread subsystem’s uniqueness guarantee to this module’s `wf()` preconditions.

### Low
- None.

## Positive Observations
- All original functions (`new`, `state`, `state_mut`, `bury`, `find_thread`, `find_thread_mut`) have corresponding verified versions with explicit contracts and invariants.
- The ghost model clearly documents trust boundaries and integrates PID stability via a verified dependency (`process_state`).
- Proofs include useful lemmas for search correctness, view equality, and preservation of well-formedness under documented frame conditions.
- Spec/proof/exec separation is clean and consistent with the repository’s split structure.

## Summary
The verification provides a clear, well-structured ghost model and proves identity and well-formedness properties, but several core behaviors remain at the trust boundary (notably thread lookup and mutable access). Strengthening the integration proofs for `find_thread`/`find_thread_mut`, and modeling ownership transfer and richer `ProcessState` effects, would materially improve soundness and semantic equivalence. Overall, the design-level verification is solid, but critical behavioral refinements are still assumed rather than proven.
