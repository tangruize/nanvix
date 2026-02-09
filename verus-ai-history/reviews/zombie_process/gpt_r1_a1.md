# Review: zombie_process (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `find_thread` / `find_thread_mut` (exec `zombie.rs`), `spec_find_thread_integration_obligation` (spec `zombie.spec.rs`)
  - **Description:** Executable iterator-based search is not linked to the spec model; the proof only validates a ghost search and leaves the integration obligation unproven. This means bugs in the real predicate/iteration order or `ThreadIdentifier` equality are not caught, so semantic equivalence is not established.
  - **Suggested Fix:** Add refinement proofs that `iter().find(|t| t.id() == tid)` corresponds to `spec_has_zombie_thread` under `wf()` and that the returned reference’s `id` matches `tid`, or downgrade the spec to an external-body contract and prove the obligation at integration time.

- **Location:** `state_mut` (exec `zombie.rs`)
  - **Description:** `state_mut()` is `external_body` with strong frame conditions that assume PID immutability and no changes to zombie threads/status, but this is only justified by a trust assumption about `ProcessState`’s API. This is a soundness gap in a core module until the `ProcessState` interface is verified.
  - **Suggested Fix:** Prove (in the `ProcessState` module) that no public setter can mutate PID, and add an integration lemma linking that proof to this frame condition; alternatively, weaken the postcondition to only guarantee PID stability when enforced by an explicit capability or invariant.

### Medium
- **Location:** `wf()` / `spec_no_duplicates` (spec `zombie.spec.rs`), `new()` (exec `zombie.rs`)
  - **Description:** The model requires unique thread IDs, but the original constructor does not enforce uniqueness. If the global thread subsystem does not guarantee uniqueness, the spec is stronger than the implementation.
  - **Suggested Fix:** Either prove uniqueness as a global invariant from the thread subsystem and link it here, or relax `wf()` and related lemmas to allow duplicates while adjusting `find_thread` properties accordingly.

- **Location:** `find_thread_mut` (exec `zombie.rs`)
  - **Description:** The model returns a ghost `Option<int>` and asserts `self` is unchanged, but the real function returns `&mut ZombieThread`, allowing callers to mutate thread identity or state. The spec does not capture the required post-mutation obligations, so invariants can be broken without proof.
  - **Suggested Fix:** Model mutable borrowing with tracked references and specify (and prove) caller obligations to preserve thread identity/list membership, or add a dedicated mutation API with explicit postconditions.

- **Location:** `bury()` (exec `zombie.rs`)
  - **Description:** The ghost model only returns IDs/status and does not verify ownership/resource transfer semantics, which are the key OS-level effect of `bury()` (handing resources back to the parent). This makes the spec too weak for resource safety.
  - **Suggested Fix:** Introduce tracked ownership models for threads and `ProcessState` and prove transfer of ownership/cleanup rights, or explicitly document and discharge this as an integration obligation in a higher-level proof.

### Low
- **Location:** `new()` / `spec_process_state_pid_integration_obligation` (spec `zombie.spec.rs`, exec `zombie.rs`)
  - **Description:** PID linkage between the ghost model and the real `ProcessState` is only an unproven integration obligation; the model does not establish that `pid@` matches `process.pid()` in the original code.
  - **Suggested Fix:** Add an integration proof that extracts PID from `ProcessState` at construction and ties it to the ghost `pid`.

## Positive Observations
- Clear separation of exec/spec/proof with extensive documentation of trust boundaries.
- Well-formedness, PID immutability, and exit-status immutability are consistently specified and preserved.
- Ghost-level search correctness is proven, and the integration obligations are explicitly named.

## Summary
The verification provides a solid design-level model with good invariants and proofs, but several core behaviors (iterator search, mutable reference effects, PID linkage, and ownership transfer) remain unproven or modeled only by assumptions. Addressing the unproven integration obligations and modeling resource transfer would significantly strengthen equivalence and soundness.
