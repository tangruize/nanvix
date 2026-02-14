# Review: zombie_process Abstraction (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

- None.

### High

- None.

### Medium

- **Location:** `state_mut()` in `zombie.rs` (exec, line 207) and `spec_state_mut()` in `zombie.spec.rs` (line 386)
  **Description:** The exec `state_mut()` returns `u64` (PID) while the original returns `&mut ProcessState`. The model captures the frame condition (ZombieProcess fields unchanged) but cannot represent that callers receive a mutable reference through which they can modify non-PID ProcessState fields (capabilities, vmem, mailbox, etc.). The View-level `spec_state_mut()` returns `self` unchanged, which is correct for this module's abstraction scope, but downstream modules that compose ZombieProcess with ProcessState mutations would need to reason about this gap manually. The trust boundary documentation (lines 46–64 of `zombie.rs`) is thorough, and PID preservation is verified cross-module — but if future verification needs to track capability or vmem changes through a ZombieProcess, the model will need extension.
  **Suggested Fix:** No immediate fix needed — the trust boundary is well-documented. Consider adding a comment on `spec_state_mut()` in the View noting that it models only the ZombieProcess-level frame, not the ProcessState-internal mutation that callers can perform.

### Low

- **Location:** `lemma_new_is_wf()` in `zombie.proof.rs` (line 49)
  **Description:** This lemma's postconditions exactly restate its preconditions (`zombie_count as nat == zombie_ids.len()`, `zombie_ids.len() >= 1`, `spec_no_duplicates`). It proves nothing beyond what the requires clause already guarantees. While harmless, it adds no verification value.
  **Suggested Fix:** Either remove this lemma or strengthen it to prove something non-trivial, e.g., that the resulting View satisfies `ZombieProcessView::wf()` by constructing the view explicitly.

- **Location:** `lemma_new_establishes_pid_obligation()` in `zombie.proof.rs` (line 70)
  **Description:** This lemma requires `spec_process_state_pid_integration_obligation(pid, real_pid)` and ensures the same predicate — a tautology. It serves as documentation but adds no proof power.
  **Suggested Fix:** Consider converting to a doc comment or combining with `lemma_new_is_wf` to show that construction satisfies both wf and the PID obligation simultaneously.

- **Location:** `ZombieProcessView` in `zombie.spec.rs` (line 94)
  **Description:** No `spec_state()` getter on the View type. While `state()` is a pure getter with no side effects (so no state transition to model), having a `spec_state(self) -> u64` that returns `self.pid` would provide symmetry with `spec_state_mut()` and give downstream modules a uniform API for both accessor variants.
  **Suggested Fix:** Add `pub open spec fn spec_state(self) -> u64 { self.pid }` to `ZombieProcessView`. This is optional and purely for API completeness.

- **Location:** `spec_find_thread()` / `spec_find_thread_mut()` on `ZombieProcessView` in `zombie.spec.rs` (lines 393, 405)
  **Description:** `spec_find_thread` returns `Option<u64>` where `Some(0)` means "found in zombie list." The sentinel value `0` is an implementation artifact from the exec model (where the original uses `ThreadRef::Zombie` variant). At the View level, a boolean or `Option<()>` would be more abstract since a ZombieProcess has only one thread list, making the list-index encoding unnecessary.
  **Suggested Fix:** Low priority. The current encoding is consistent between exec and View levels and doesn't cause correctness issues. If refactoring, consider `Option<()>` at the View level.

## Positive Observations

- **Thorough trust boundary documentation.** The module header and per-function doc comments clearly identify every `external_body` annotation, what it assumes, and where the assumption is discharged (cross-module references to `process_state` verification). This is exemplary for a verification project.
- **Complete set of View-level spec transition functions.** Every exec function that modifies or decomposes state (`new`, `bury`, `state_mut`, `find_thread`, `find_thread_mut`) has a corresponding View-level spec function. The bridging lemmas (`lemma_*_matches_spec_*`) connect exec postconditions to View-level transitions.
- **Strong bridging lemma coverage.** Seven bridging lemmas connect exec to View: `lemma_new_matches_spec_new`, `lemma_bury_matches_spec_bury`, `lemma_state_mut_matches_spec`, `lemma_find_thread_matches_view_spec`, `lemma_find_thread_mut_matches_spec`, `lemma_exec_wf_implies_view_wf`, `lemma_has_zombie_thread_matches_view`. This is comprehensive.
- **Integration obligations are well-structured.** The five obligations (`spec_find_thread_integration_obligation`, `spec_find_thread_search_predicate_obligation`, `spec_find_thread_mut_caller_obligation`, `spec_state_mut_pid_stability_obligation`, `spec_bury_ownership_integration_obligation`) decompose the trust boundary into independently dischargeable pieces with clear integration proof plans.
- **Proof of ghost search correctness.** `lemma_ghost_search_correctness` and `lemma_predicate_obligation_implies_search_equivalence` provide a solid proof chain from ghost-level search to real implementation correctness (modulo the external_body gap). The `real_ids` parameter in the predicate equivalence lemma avoids the tautology trap.
- **Clean verification.** 24 verified, 0 errors. All exec functions, spec functions, and proof lemmas verify successfully.
- **View wf() is properly simplified.** The View-level `wf()` omits the `zombie_count` field (which is redundant with sequence length) — correct abstraction since the View doesn't carry implementation-level bookkeeping.

## Summary

The abstraction improvements are well-executed. Every exec-level state transition has a matching View-level spec function, bridging lemmas connect the two levels, and integration obligations clearly delineate the trust boundary. The `ZombieProcessView` type properly abstracts away implementation details (`zombie_count`, `Vec` vs `Seq`) while preserving the essential properties (non-emptiness, uniqueness, immutability of PID and status). The only substantive concern is the `state_mut()` model's inability to represent ProcessState-internal mutations, which is a known limitation that is well-documented. Two proof lemmas are tautological but harmless. Verification passes cleanly with 24/0.
