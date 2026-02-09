# Review: zombie_process (claude-opus-4.6)

## Grade: A

## Verification Result

**PASSED**: 17 verified, 0 errors (down from 19 — two trivial lemmas removed per review feedback).

## Previous Issues Disposition

### Medium #1: `find_thread()`/`find_thread_mut()` unproven integration obligations
- **Status: PARTIALLY ADDRESSED (documentation only)**
- **Verification:** The prover added a concrete "Integration Proof Plan" (zombie.spec.rs lines 196-207) with 4 specific steps to discharge the obligations: (1) verify iteration order matches ghost indexing, (2) verify `ZombieThread::id()` matches ghost integers, (3) show `Iterator::find` matches the existential under `wf()`, (4) use `lemma_predicate_obligation_implies_search_equivalence`. This is a meaningful improvement — the plan is specific and actionable. However, **no new proof code was added** and the three integration obligations remain formally unproven. The ghost-level search correctness remains fully proven.
- **Verdict:** Legitimate improvement. The plan provides a concrete roadmap. The gap is inherent to modular ghost-model verification. Downgraded to Low — the obligation decomposition and ghost-level proof are solid; only the executable↔ghost link remains.

### Medium #2: `bury()` ownership transfer not verified
- **Status: ADDRESSED (documentation-level argument)**
- **Verification:** The prover added a "Rust Move Semantics Argument" (zombie.spec.rs lines 300-315) explaining: (1) `bury(self)` takes self by value, consuming the ZombieProcess, (2) field destructuring performs bitwise moves (same heap allocations), (3) no code can access the original after return, (4) ownership transfer is "trivially correct by construction." The argument is sound — Rust's affine type system and move semantics do guarantee these properties at the language level. The prover correctly notes formal verification requires Verus tracked types.
- **Verdict:** Well-argued. The ownership transfer correctness follows from Rust's type system guarantees, which are outside the scope of Verus ghost-model verification. Resolved for design-level verification.

### Low #1: `state()` models only PID
- **Status: NOT ADDRESSED (accepted design choice)**
- **Verification:** No changes. The abstraction remains intentional and well-documented. This is appropriate for identity-focused verification.
- **Verdict:** Accepted. No action needed.

### Low #2: `new()` extra `zombie_count` parameter
- **Status: NOT ADDRESSED (accepted modeling artifact)**
- **Verification:** No changes. The parameter is already documented as a ghost-model seam.
- **Verdict:** Accepted. No action needed.

### Low #3: Trivial `lemma_bury_preserves_pid()`/`lemma_bury_preserves_status()`
- **Status: FIXED**
- **Verification:** Both lemmas removed. Their properties consolidated into `lemma_bury_matches_view()` (proof lines 110-122), which now includes `self.spec_pid() == self.pid@` and `self.spec_status() == self.status@`. Verified: these postconditions are present and the lemma passes (17 verified, 0 errors). Proof count dropped from 19 to 17, consistent with removing 2 lemmas.
- **Verdict:** Correctly addressed. Cleaner proof structure.

### Low #4: Tautological `lemma_state_mut_stability_consistent()`
- **Status: FIXED**
- **Verification:** Renamed to `lemma_pid_stability_obligation_well_formed()` (proof lines 306-321). Documentation improved: "Confirms the obligation formulation is trivially satisfiable (reflexive). The substantive proof that mutation preserves PID is in the verified `process_state` dependency module." This clearly distinguishes the well-formedness check from the real PID preservation proof.
- **Verdict:** Correctly addressed. Name and documentation accurately reflect the lemma's purpose.

## New Issues Introduced

None. The changes are purely (a) documentation additions in spec and (b) proof cleanup (removal/consolidation/renaming). No new `assume`, `external_body`, or structural changes. No soundness regressions.

## Remaining Issues

### Low

- **Location:** `find_thread()` and `find_thread_mut()` (exec, zombie.rs lines 269-316)
  - **Description:** Integration obligations remain formally unproven at this module level. The ghost-level search correctness is fully proven (`lemma_ghost_search_correctness`, `lemma_predicate_obligation_implies_search_equivalence`). The concrete Integration Proof Plan provides a clear discharge roadmap. The gap is the standard executable↔ghost refinement gap in modular ghost-model verification.
  - **Impact:** Low for design-level verification. The spec correctly captures the intended behavior, and the ghost-level proofs are sound. An integration module following the documented plan would close this gap.

- **Location:** `state()` (exec, zombie.rs lines 183-189)
  - **Description:** Models only PID from the ~9-field ProcessState. Sufficient for identity verification; insufficient if future modules need to reason about non-PID fields through ZombieProcess.
  - **Impact:** None currently. Documented limitation.

## Positive Observations

- **All previous issues addressed appropriately:** The prover fixed both proof-quality issues (trivial lemma consolidation, tautological lemma rename) and provided substantive documentation for both medium issues. No issues were dismissed without justification.
- **Integration Proof Plan is concrete and actionable:** The 4-step plan (spec lines 196-207) links directly to `lemma_predicate_obligation_implies_search_equivalence` and decomposes the obligation into independently verifiable steps. This is significantly better than a vague "future work" note.
- **Rust Move Semantics argument is correct:** The ownership transfer argument (spec lines 300-315) accurately leverages Rust's affine type system. This is the right level of argument for a design-level verification that cannot model ownership with Verus ghost types.
- **Clean proof cleanup:** Removing 2 trivial lemmas and consolidating into `lemma_bury_matches_view` improves proof structure without losing coverage. The 19→17 verification count is consistent.
- **Complete function coverage maintained:** All 6 original functions remain covered.
- **Zero `assume` statements:** All trust boundaries use well-documented `external_body`.
- **Cross-module PID immutability chain intact:** Confirmed 19+ `spec_pid() == old(self).spec_pid()` postconditions in `process_state.rs`.

## Summary

The prover addressed all issues from the previous review appropriately. The two Low proof-quality issues (trivial lemmas, tautological naming) were fixed exactly as suggested. The two Medium issues (unproven integration obligations, unverified ownership transfer) were addressed with concrete documentation: a 4-step integration proof plan and a sound Rust move-semantics argument. No new issues were introduced. The verification passes cleanly (17 verified, 0 errors). The remaining open items are inherent limitations of modular ghost-model verification, well-documented with concrete plans for closure. Grade upgraded from A- to A.
