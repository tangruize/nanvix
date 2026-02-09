# Review: interrupted_process (claude-opus-4.6)

## Grade: A

## Previous Issues Disposition

### High: Boundary RunnableProcess lacks disjointness in wf() — **FIXED** ✅
The boundary `RunnableProcess::wf()` (`interrupted.spec.rs:234-249`) now includes all 4 per-list `spec_no_duplicates` checks, all 6 pairwise `spec_seqs_disjoint` checks, plus `ready_admission_times` length matching and non-negativity. The proof lemmas (`lemma_subrange_preserves_no_duplicates`, `lemma_front_not_in_tail`, `lemma_tail_disjoint_sleeping`, `lemma_tail_disjoint_zombie`) are now actively invoked in the `resume()` proof block (lines 272-337). The inline proof blocks (lines 284-328) additionally prove singleton-ready no-duplicates and ready-vs-{remaining, sleeping, zombie} disjointness. Verified: the `result.wf()` postcondition is now proven against the full strong predicate. **Genuine fix.**

### Medium: Cross-module boundary inconsistency — **Documented** ✅
The `InterruptedProcess` boundary in `runnable.spec.rs:138-145` still omits `sleeping_thread_ids`. However, the prover added explicit documentation in the trust boundary section (`interrupted.rs:38-40`) explaining that cross-module linking involving sleeping threads must use this module's primary model. The spec trust assumptions section (`interrupted.spec.rs:40-44`) also documents this. **Acceptable resolution** — the inconsistency is a cross-module concern outside this module's scope, and it's now clearly documented for future maintainers.

### Medium: find_thread/find_thread_mut spec-only — **Documented** ✅
The trust boundary section (`interrupted.rs:43-47`) now explicitly documents that these functions are spec-only, that the original iterator-based search logic is unverified, and that this should be revisited if Verus adds relevant support. The spec file trust assumptions (`interrupted.spec.rs:46-53`) also document this. **Acceptable resolution** — this is a fundamental Verus limitation.

### Medium: Boundary RunnableProcess omits ready_admission_times — **FIXED** ✅
The boundary `RunnableProcess` struct now includes `ready_admission_times: Ghost<Seq<int>>` (line 93). The `resume()` postcondition constrains it: `result.ready_admission_times@.len() == 1` and `result.ready_admission_times@[0] >= 0` (lines 248-249). The `wf()` validates matching lengths and non-negative times (lines 236-238). The `RunnableProcessView` also includes `ready_admission_times` (line 87). **Genuine fix.**

### Low: state_mut() lacks explicit wf() — **FIXED** ✅
`state_mut()` now has `requires old(self).wf()` (line 215) and `ensures self.wf()` (line 222). Verified as part of the 18/18 conditions. **Genuine fix.**

### Low: Redundant construction lemmas — **Documented** ✅
Comments added (`interrupted.proof.rs:31-32, 57`) explaining these are retained as regression guards for downstream proofs that may construct `InterruptedProcess` values directly. **Acceptable resolution.**

## Issues Found

### Critical

- None.

### High

- None.

### Medium

- **Boundary RunnableProcess::wf() is strictly stronger than real module's wf()** (spec — `interrupted.spec.rs:234-249` vs `runnable.spec.rs:346-357`)
  - **Description:** The boundary `RunnableProcess::wf()` in this module includes no-duplicates and pairwise-disjoint conditions (6 disjointness + 4 uniqueness checks). The real `RunnableProcess::wf()` in `runnable.spec.rs` does NOT include these — disjointness is a separate optional predicate `spec_ids_disjoint()`. The real `wf()` also includes exec-level counter matching (`interrupted_count as nat == ...`) which the boundary model doesn't have. This means: (a) the boundary `wf()` guarantees MORE than what the real module's `wf()` provides, and (b) downstream code that receives a `RunnableProcess` from the real module and checks only `wf()` won't get the disjointness guarantees that this module's boundary promises. This is conservative (the boundary overpromises relative to what it can verify), but creates a cross-module consistency gap. If cross-module linking is attempted, the boundary `wf()` cannot be directly equated to the real `wf()`.
  - **Suggested Fix:** Document this explicitly. Consider splitting the boundary `wf()` into a base `wf()` (matching the real module's definition, minus exec counters) and a `wf_strong()` that adds disjointness, so the relationship is clear. Alternatively, document that `resume()` guarantees `wf() && spec_ids_disjoint()` from the real module's perspective.

### Low

- **Duplicated spec helpers across InterruptedProcess and RunnableProcess** (spec — `interrupted.spec.rs:179-189, 252-262`)
  - **Description:** `spec_no_duplicates` and `spec_seqs_disjoint` are defined identically on both `InterruptedProcess` and `RunnableProcess` impl blocks. The `resume()` proof uses `InterruptedProcess::spec_no_duplicates` in assertions, while `RunnableProcess::wf()` uses `RunnableProcess::spec_no_duplicates`. This works because both are `open spec fn` with identical bodies — the SMT solver unfolds them and sees equivalence. However, if either were ever changed to `closed`, the proof would silently break. This coupling is fragile.
  - **Suggested Fix:** Extract shared helpers to a common module or at minimum add a comment in the proof block noting the reliance on identical definitions. A `proof fn lemma_helpers_equivalent` asserting the equivalence would make the dependency explicit.

- **Admission time initialized to 0 without documenting the abstraction choice** (exec — `interrupted.rs:343`)
  - **Description:** The `resume()` function sets `ready_admission_times` to `[0]`. In the original code, a resumed thread's admission time is determined by the runtime clock (`clock::now()`). The choice of 0 is a valid abstraction (any non-negative value satisfies the spec), but the rationale isn't documented. A reader might wonder if 0 has special significance or if it's an approximation.
  - **Suggested Fix:** Add a brief comment explaining that 0 is an abstract placeholder for the runtime clock value, and that the postcondition only constrains it to be non-negative.

## Positive Observations

- **All 6 prior issues addressed.** 4 were genuinely fixed with code changes, 2 were documented with clear rationale. No issues were dismissed without justification.
- **100% function coverage maintained.** All 7 original methods plus standalone `interrupt()` are modeled and verified (8 functions total).
- **Zero assume statements.** No unjustified assumptions. Only 2 `external_body` annotations on `state()`/`state_mut()`, both justified by Verus reference-type limitations.
- **18/18 verification conditions pass.**
- **The proof of `resume()` is now substantive.** The proof block (lines 263-338) actively invokes 4 lemmas and contains 4 inline proof-by-assertion blocks, discharging all 10 structural invariant conditions of the strengthened boundary `wf()`. This is real proof work, not just SMT auto-discharge.
- **Strong boundary model.** The `RunnableProcess` boundary now models admission times, per-list uniqueness, and pairwise disjointness — stronger than what most boundary models provide. While this creates a consistency question with the real module, it means `resume()` guarantees maximum structural integrity.
- **Clean split maintained.** Exec code contains no spec leakage. Proof lemmas are in the proof file. Spec functions and `wf()` predicates are in the spec file. The `include!` composition is clean.
- **Trust boundary documentation is thorough.** Both the exec file header (lines 32-49) and spec file header (lines 35-56) clearly document all modeling decisions, limitations, and cross-module concerns.

## Summary

The prover addressed all 6 issues from the previous review. The most impactful change was strengthening the boundary `RunnableProcess::wf()` to include full structural invariants (no-duplicates + pairwise-disjoint) and wiring the existing proof lemmas into `resume()`. This transforms the `resume()` postcondition from a weak "at least one ready thread" guarantee into a comprehensive structural integrity proof. The `ready_admission_times` field and `state_mut()` wf-preservation fixes are also genuine improvements.

The remaining medium issue — boundary `wf()` being strictly stronger than the real module's `wf()` — is a cross-module design concern rather than a soundness bug. It means this module's verification is conservative (overpromises), which is safe. The low issues are documentation/hygiene.

**Verdict:** This verification is now comprehensive and sound for its intended scope. The module is ready for cross-module integration, with the documented caveats about boundary model differences.
