# Review: fence (claude-opus-4.6) — Round 2

## Grade: A-

## Previous Issues — Disposition

### High

- **`wait()` requires `self.spec_is_satisfied()` precondition (retained, acknowledged)**
  - **Status:** Not fixed — inherent limitation of sequential modeling.
  - **Verification:** The documentation at lines 76–84 of `fence.rs` now explicitly labels this as a "Key verification gap" and explains the trust boundary clearly. The prover did not adopt the suggested `external_body` alternative, which is a valid choice — both approaches involve a trust assumption; the precondition approach is more conservative (it requires the caller to prove satisfaction, rather than assuming it axiomatically). The documentation is now exemplary in its transparency. **Accepted as a documented limitation.**

- **`signal(&mut self)` vs original `&self` (retained, acknowledged)**
  - **Status:** Not fixed — inherent limitation of sequential modeling.
  - **Verification:** Documentation at lines 85–89 clearly explains the limitation and points to `lemma_signal_commutativity` for spec-level mitigation. **Accepted as a documented limitation.**

### Medium

- **Stale `kmain.rs` over-signaling documentation (FIXED)**
  - **Status:** Fixed. Lines 65–72 now correctly trace the call chain: `startup::init(ncores - 1)` at line 347 → `Fence::new(ncores)` at line 117 where the parameter `ncores` is the already-decremented value. I verified against the source: `ncores = madt.cores_count() - 1` (line 346), `startup::init(ncores - 1)` (line 347), `Fence::new(ncores)` inside `init` (line 117). The fence total is `madt.cores_count() - 2`, but `madt.cores_count() - 1` application cores each signal once, confirming over-signaling by exactly one. **Documentation is now accurate.**

- **Trivial definitional lemmas (ADDRESSED)**
  - **Status:** Addressed. Lines 12–19 of `fence.proof.rs` now include a clear header comment: "The following lemmas are intentionally shallow definition-unfolding properties. They serve as executable documentation and regression tests that guard against accidental spec changes." This adequately sets expectations. **Accepted.**

- **`signal()` precondition strengthening documentation (ADDRESSED)**
  - **Status:** Addressed via the corrected `kmain.rs` documentation. The over-signaling divergence is now accurately described with the correct call chain. **Accepted.**

### Low

- **`lemma_total_signals_satisfies` redundancy (FIXED)**
  - **Status:** Fixed. Renamed to `lemma_equality_implies_satisfaction` (line 124 of `fence.proof.rs`). The description at lines 117–123 now explicitly clarifies it as "a static arithmetic fact, not an inductive proof" and directs the reader to `lemma_signals_accumulate_to_satisfaction` for the inductive property. **Good fix.** However, this rename introduced a new issue — see below.

- **`pub` fields (retained, Verus limitation):** Documented at line 121 of `fence.rs`. No action possible.
- **`const fn` (retained, Verus limitation):** Documented at lines 138–139. No action possible.

## New Issues Found

### Low

- **Location:** `lemma_signals_accumulate_to_satisfaction` doc comment in `fence.proof.rs`, line 238
  - **Description:** The doc comment references the old lemma name `lemma_total_signals_satisfies` which was renamed to `lemma_equality_implies_satisfaction` in this revision. This is a stale reference introduced by the rename.
  - **Suggested Fix:** Change line 238 from `Unlike `lemma_total_signals_satisfies`` to `Unlike `lemma_equality_implies_satisfaction``.

## Positive Observations

- **Exemplary documentation.** The module-level documentation in `fence.rs` is among the best I've reviewed for a verification module. Every trust boundary, API divergence, and modeling limitation is explicitly documented with precise line-number references to the original codebase. The "Trust Boundaries" section (lines 74–91) is particularly well-written — it is honest about what the verification does and does not prove.

- **No soundness escape hatches.** No `assume`, `external_body`, `trusted`, or `axiom` annotations anywhere. All verification conditions are discharged cleanly by Verus.

- **Clean three-file separation.** The spec/proof/exec split is well-organized and follows the project's split convention.

- **Comprehensive `signal()` postconditions.** The ensures clause covers count increment, total preservation, wf preservation, remaining decrement, and satisfaction on last signal.

- **Proof file organization improved.** The three-section structure (Definitional Properties, Protocol Properties, Concurrency Properties) with explanatory headers provides clear separation of concerns.

- **Responsive to review feedback.** All actionable items from the previous review were addressed: documentation was corrected, the trivial-lemmas concern was acknowledged with a header comment, and the redundant lemma was renamed with a clarified description.

## Summary

The prover addressed all actionable items from the previous review. The `kmain.rs` documentation is now accurate, the trivial-lemma concern is acknowledged with a clear header comment, and the redundant lemma was renamed with better documentation. The two high-severity modeling gaps (`wait()` precondition, `signal(&mut self)`) remain as inherent limitations of the sequential approach, but are now documented with exceptional clarity and transparency.

The only new issue is a stale cross-reference introduced by the lemma rename — a trivial fix.

The verification is sound within its stated scope: it proves sequential state-machine correctness of the fence protocol (well-formedness preservation, satisfaction monotonicity, signal accumulation, and commutativity). The trust boundaries are clearly delineated. This is a well-executed verification module.
