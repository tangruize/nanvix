# Review: zombie_process (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `find_thread` / `find_thread_mut` (exec `zombie.rs`), `spec_find_thread_integration_obligation` (spec `zombie.spec.rs`)
  - **Description:** The executable iterator-based search is still not linked to the spec model. The new obligations are documented but remain unproven, so semantic equivalence with `iter().find(|t| t.id() == tid)` is not established.
  - **Suggested Fix:** Provide refinement proofs that the iterator search predicate and iteration order correspond to the ghost model, or move these functions to `external_body` with proven integration lemmas in a higher-level module.

- **Location:** `state_mut` (exec `zombie.rs`), `spec_state_mut_pid_stability_obligation` (spec `zombie.spec.rs`)
  - **Description:** `state_mut()` remains `external_body` with strong frame conditions. The added obligation is still a trust assumption and is not verified against the `ProcessState` API; this is a soundness gap in a core module.
  - **Suggested Fix:** Prove in the `ProcessState` module that no public API can mutate `pid`, and link that proof here; otherwise, weaken the postcondition to reflect only what can be justified.

### Medium
- **Location:** `wf()` / `spec_no_duplicates` (spec `zombie.spec.rs`), `new()` (exec `zombie.rs`)
  - **Description:** Uniqueness of thread IDs is still assumed as a precondition and only justified in comments. There is no proof or integration lemma tying this to the thread subsystem, so the spec can remain stronger than the constructor.
  - **Suggested Fix:** Add an integration proof from the thread subsystem guaranteeing unique `ThreadIdentifier`s, or relax `wf()` to avoid requiring global uniqueness.

- **Location:** `find_thread_mut` (exec `zombie.rs`), `spec_find_thread_mut_caller_obligation` (spec `zombie.spec.rs`)
  - **Description:** The mutable reference side effects are not modeled. The new caller obligation is only documented and not enforced or discharged, so invariants can still be violated by mutation through `ThreadRefMut`.
  - **Suggested Fix:** Model mutable borrowing with tracked references and add verified postconditions for identity/list membership preservation at call sites.

- **Location:** `bury()` (exec `zombie.rs`), `spec_bury_ownership_integration_obligation` (spec `zombie.spec.rs`)
  - **Description:** Ownership/resource transfer semantics remain unverified; only identity preservation is checked. This is still too weak for the OS-level purpose of `bury()`.
  - **Suggested Fix:** Add tracked ownership models and prove transfer of thread/process ownership, or discharge this obligation in an integration layer that reasons about ownership.

### Low
- **Location:** `new()` / `spec_process_state_pid_integration_obligation` (spec `zombie.spec.rs`)
  - **Description:** The PID linkage between the ghost model and the real `ProcessState` is still an unproven integration obligation.
  - **Suggested Fix:** Add an integration proof that `pid@` equals `process.pid()` at construction and preserve that link via the postconditions.

- **Location:** `lemma_predicate_obligation_implies_search_equivalence` (proof `zombie.proof.rs`)
  - **Description:** The lemma uses `spec_find_thread_search_predicate_obligation` with identical ghost IDs (`ghost_id == ghost_id`), which does not connect to real thread IDs or iterator behavior. This can be misleading because it appears to establish search equivalence without referencing the executable search.
  - **Suggested Fix:** Recast the lemma to take explicit real IDs from the concrete list or remove it until a true refinement proof is available.

## Positive Observations
- The updated specs clearly enumerate integration obligations and trust boundaries, improving auditability.
- ProcessState’s `pid` is private by inspection, which supports the intended PID immutability claim, even though it is still unverified.
- The spec/proof split remains clean and well-documented.

## Summary
The changes mostly add documentation and formalized obligations, but the core verification gaps remain: executable search equivalence, mutable-reference effects, PID linkage, and ownership transfer are still unproven. Verification is still design-level and incomplete for soundness and equivalence.
