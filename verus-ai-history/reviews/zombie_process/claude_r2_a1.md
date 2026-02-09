# Review: zombie_process (claude-opus-4.6)

## Grade: A-

## Verification Result

**PASSED**: 19 verified, 0 errors.

## Issues Found

### Critical

None.

### High

None.

### Medium

- **Location:** `find_thread()` and `find_thread_mut()` (exec, zombie.rs lines 269-316)
  - **Description:** Both functions are `external_body` with spec contracts as postconditions. The executable iterator-based search (`iter().find(|t| t.id() == tid)`) is not verified. While the ghost-level search correctness is proven via `lemma_ghost_search_correctness`, the refinement from executable to ghost relies on three unproven integration obligations (`spec_find_thread_integration_obligation`, `spec_find_thread_search_predicate_obligation`, and `spec_find_thread_mut_caller_obligation`). No integration test module discharges these obligations.
  - **Suggested Fix:** Create an integration proof module that witnesses the predicate equivalence between `ZombieThread::id()` integer comparison and the ghost model's `Seq<int>` indexing, or document a concrete plan for when/how these obligations will be discharged. The ghost search proof is solid; the remaining gap is the executable↔ghost link.

- **Location:** `bury()` (exec, zombie.rs lines 232-247)
  - **Description:** The original `bury()` transfers ownership of `(NonEmptyVecDeque<ZombieThread>, Box<ProcessState>, ExitStatus)` — moving actual heap-allocated thread objects and process state to the parent. The ghost model only returns `(Ghost<Seq<int>>, Ghost<int>, Ghost<int>)` and verifies identity preservation but not ownership transfer or resource consumption. `spec_bury_ownership_integration_obligation` documents this gap but no tracked-type verification is provided.
  - **Suggested Fix:** This is an inherent limitation of the ghost model approach. For a complete verification, model with Verus tracked types or add a comment-level argument for why Rust's move semantics guarantee the ownership transfer. The identity preservation proof is sufficient for design-level verification.

### Low

- **Location:** `state()` (exec, zombie.rs lines 183-189)
  - **Description:** Returns `Ghost<int>` modeling only the PID, while the original returns `&ProcessState` with ~9 fields (pid, capabilities, vmem, events, mailbox, mmio, pmio, mutexes, conditions). If any caller of `ZombieProcess::state()` reasons about non-PID fields, the ghost model is insufficient. This is documented but worth flagging.
  - **Suggested Fix:** No change needed unless future modules need to reason about non-PID fields through a `ZombieProcess` reference. The abstraction is appropriate for identity-focused verification.

- **Location:** `new()` (exec, zombie.rs lines 146-169)
  - **Description:** The verified constructor adds a `zombie_count: u64` parameter not present in the original. The original `new()` receives `NonEmptyVecDeque<ZombieThread>` which implicitly knows its length. The extra parameter creates a precondition (`zombie_count as nat == zombie_ids@.len()`) that is a modeling artifact rather than a semantic requirement.
  - **Suggested Fix:** Document at the integration boundary that callers must establish `zombie_count == zombie_threads.len()`. This is a standard ghost-model seam and not a soundness issue.

- **Location:** `lemma_bury_preserves_pid()`, `lemma_bury_preserves_status()` (proof, zombie.proof.rs lines 122-133)
  - **Description:** These lemmas are trivially true by definition (they restate that `self.spec_pid() == self.pid@`). They add proof clutter without substantive verification value.
  - **Suggested Fix:** Consider removing or consolidating into the existing `lemma_bury_matches_view()` which already covers these properties.

- **Location:** `lemma_state_mut_stability_consistent()` (proof, zombie.proof.rs lines 325-332)
  - **Description:** Proves only that the PID stability obligation is satisfied when PID is compared against itself (`self.spec_pid(), self.spec_pid()`). This is tautological. The substantive proof is in the `process_state` module, which is correctly documented, but this lemma provides no additional assurance.
  - **Suggested Fix:** Rename to clarify its role as a well-formedness check (e.g., `lemma_pid_stability_obligation_well_formed`) or remove. The real value is the cross-module PID preservation verified in `process_state.rs`.

## Positive Observations

- **Complete function coverage:** All 6 functions from the original source (`new`, `state`, `state_mut`, `bury`, `find_thread`, `find_thread_mut`) have verified counterparts.
- **No `assume` statements:** The verification contains zero unjustified assumptions. All trust boundaries use `external_body` with documented justifications.
- **Well-documented trust boundaries:** Every `external_body` function has detailed documentation explaining (1) why it's external, (2) what the postcondition asserts, and (3) what integration obligations remain. The header documentation is particularly thorough.
- **Cross-module PID immutability verified:** The `state_mut()` external_body claim is backed by verified evidence — every public mutator in `process_state.rs` has a verified `self.spec_pid() == old(self).spec_pid()` postcondition (confirmed by inspection of 19+ postcondition occurrences).
- **Strong well-formedness invariant:** `wf()` captures three essential properties: length consistency, non-empty guarantee, and thread ID uniqueness. The no-duplicates invariant correctly models Rust's ownership-based uniqueness.
- **Ghost search correctness is proven end-to-end:** `lemma_ghost_search_correctness` proves forward, backward, and completeness properties. `lemma_predicate_obligation_implies_search_equivalence` bridges ghost-to-real with an explicit `real_ids` parameter, avoiding tautology.
- **Clean three-file split:** Spec, proof, and exec are properly separated with clear responsibilities.
- **Integration obligations are well-structured:** Six formal obligations decompose the trust boundary into independently dischargeable proof goals.

## Summary

This is a solid design-level verification of `ZombieProcess`. The ghost model correctly captures the essential correctness properties: non-empty zombie thread invariant, thread ID uniqueness, PID/status immutability, decomposition correctness (`bury`), and search correctness (`find_thread`). The four `external_body` functions are well-justified — `state()`/`state_mut()` are backed by cross-module verified evidence, and `find_thread()`/`find_thread_mut()` have detailed integration obligations decomposing the refinement proof. The main gap is that integration obligations remain formally unproven at this module level, and ownership transfer in `bury()` is not modeled. These are inherent limitations of ghost-model verification, clearly documented. The proof lemmas are meaningful, with `lemma_ghost_search_correctness` and `lemma_predicate_obligation_implies_search_equivalence` being the most substantive contributions.
