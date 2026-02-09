# Review: zombie_process (gpt-5.2-codex)

## Grade: C+

## Issues Found

### Critical
- None.

### High
- **Location:** `find_thread` / `find_thread_mut` (exec `zombie.rs`), `spec_find_thread_integration_obligation` (spec `zombie.spec.rs`)
  - **Description:** These functions are now `external_body` with postconditions that assert the spec result, but there is still no refinement proof linking the real iterator search to the ghost model. The obligation is explicitly marked unproven, so semantic equivalence is still missing for a core API.
  - **Suggested Fix:** Provide refinement proofs that `iter().find(|t| t.id() == tid)` corresponds to `spec_find_thread`, or discharge the obligation in an integration layer that ties the real `NonEmptyVecDeque<ZombieThread>` iteration to the ghost sequence.

### Medium
- **Location:** `state_mut` (exec `zombie.rs`), `spec_state_mut_pid_stability_obligation` (spec `zombie.spec.rs`)
  - **Description:** The spec claims PID immutability is verified cross-module, but the ProcessState module still uses multiple `external_body` stubs for public mutators (mailbox, mmio, vmem, events, pmio), so PID preservation remains partially assumed rather than fully proven. This weakens the soundness of the `state_mut()` frame condition.
  - **Suggested Fix:** Either prove PID preservation for the concrete implementations of those mutators (or remove `external_body`), or downgrade the claim to a documented trust assumption and narrow the `state_mut()` postcondition accordingly.

- **Location:** `wf()` / `spec_no_duplicates` (spec `zombie.spec.rs`), `new()` (exec `zombie.rs`)
  - **Description:** Thread ID uniqueness remains a precondition justified only by comments; there is still no integration proof that the thread subsystem guarantees uniqueness. The spec can remain stronger than the constructor.
  - **Suggested Fix:** Add a verified lemma from the thread subsystem guaranteeing uniqueness of `ThreadIdentifier`s or relax `wf()`/`new()` to allow duplicates.

- **Location:** `find_thread_mut` (exec `zombie.rs`), `spec_find_thread_mut_caller_obligation` (spec `zombie.spec.rs`)
  - **Description:** Caller discipline for mutation is still only documented. Because mutable borrow effects are not modeled, invariants can still be violated without proof.
  - **Suggested Fix:** Model mutable borrowing with tracked references and require/verify postconditions at call sites, or restrict mutation to an API with explicit invariant-preserving contracts.

- **Location:** `bury()` (exec `zombie.rs`), `spec_bury_ownership_integration_obligation` (spec `zombie.spec.rs`)
  - **Description:** Ownership/resource transfer semantics remain unverified; only identity preservation is modeled. This is still too weak for the OS-level meaning of `bury()`.
  - **Suggested Fix:** Introduce tracked ownership models and prove transfer of thread/process ownership, or discharge the obligation in a higher-level integration proof.

### Low
- **Location:** `new()` / `spec_process_state_pid_integration_obligation` (spec `zombie.spec.rs`)
  - **Description:** The PID linkage between the ghost model and the real `ProcessState` is still only an integration obligation and is not proven at construction.
  - **Suggested Fix:** Add a refinement proof that ties `pid@` to `process.pid()` at the single construction site.

## Positive Observations
- The misleading predicate-equivalence lemma was corrected to use an explicit `real_ids` sequence, avoiding tautological reasoning.
- The trust boundary documentation is clearer, and `find_thread`/`find_thread_mut` are now explicitly marked `external_body` with stated refinement obligations.
- ProcessState mutators visibly carry `spec_pid()` preservation postconditions, improving the basis for PID immutability claims (even if some remain external stubs).

## Summary
Several improvements clarify the trust boundaries and fix a prior proof-lemma weakness, but the key verification gaps remain: executable search equivalence, mutable-reference effects, PID linkage at construction, and ownership transfer are still unproven. Verification is not yet complete or fully sound for the core behaviors.
