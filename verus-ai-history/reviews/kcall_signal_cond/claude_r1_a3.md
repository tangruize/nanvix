# Review: kcall_signal_cond (claude-opus-4.6)

## Grade: A

## Previous Issues Disposition

### Low #1: Documentation duplication in "Known Limitations" — **FIXED**

The "Known Limitations" section (exec lines 71-79) now contains only the `put_cond`-on-notify-failure item. The duplicated "ProcessManager correctness" and "Liveness" bullet points that were previously copied from "Properties NOT Proven Here" have been removed. **Verified: no duplication remains.**

### Low #2: Tautological ensures in `lemma_cond_ref_released_on_get_cond_success` — **FIXED**

The lemma (proof lines 423-440) was simplified to remove the `spec_cond_ref_released(cond_addr)` ensures that merely repeated its own requires. The ensures now only contains `!spec_is_get_cond_error(...)`, which carries genuine information. A comment (proof lines 418-422) explains the structural purpose of retaining `spec_cond_ref_released` in the requires clause. **Verified: the tautology is eliminated; the remaining ensures is meaningful.**

### Low #3: Redundant `spec_is_success ==> spec_cond_ref_released` ensures — **FIXED**

The subsumed ensures clause was removed from `signal_cond_model`. The stronger clause at exec lines 390-393 (`gs.gc == GcOk ==> spec_cond_ref_released(...)`) now has a comment on line 389 reading "This subsumes the success case." The `spec_cond_slot_returned` ensures at lines 394-396 is correctly retained as a distinct property. **Verified: no redundancy remains.**

## Issues Found

### Critical

(none)

### High

(none)

### Medium

(none)

### Low

(none)

## Positive Observations

- **All issues across three rounds fully resolved.** The prover addressed 5 issues from round 1 (1 High, 2 Medium, 2 Low) and 3 Low issues from round 2, all with genuine fixes rather than dismissals.
- **Clean verification at 18/18:** All 18 verification conditions pass with 0 errors.
- **Correct semantic equivalence:** The exec model faithfully mirrors the original `signal_cond` control flow, including: (a) Rust drop semantics for Condvar after `get_cond` success, (b) `?`-operator short-circuit on notify failure skipping `put_cond`, (c) sequential error propagation with error code preservation.
- **Sound trust boundaries:** The 4 `external_body` functions (get_cond, notify, drop, put_cond) have appropriate postconditions — error codes are valid (>0), drop guarantees `spec_cond_ref_released`, put_cond guarantees `spec_cond_slot_returned` on success. No unjustified `assume` statements exist.
- **Strong postconditions on `signal_cond_model`:** The ensures clauses cover: (a) spec-exec linkage via ghost state, (b) success iff all steps succeed, (c) error iff some step fails, (d) condvar ref released on all get_cond-success paths, (e) condvar slot returned on success.
- **Comprehensive proof suite:** 18 lemmas covering error propagation, short-circuit ordering, result exhaustiveness, mutual exclusion, error code preservation, awakened count preservation, condvar lifecycle, architecture guard, and safety precondition structure.
- **Transparent documentation:** The "Known Limitations" section honestly documents the `put_cond` skip on notify failure. The "Properties NOT Proven Here" section correctly identifies out-of-scope concerns. The API mapping table provides clear traceability.
- **Clean spec/proof/exec separation:** Specs define abstract views and pipeline logic. Proofs contain only lemmas. Exec contains models and the verified function. No leakage between concerns.

## Summary

The verification of `kcall_signal_cond` is now complete and sound. All issues raised across three review rounds have been genuinely addressed. The spec correctly models the three-step kernel call pipeline with short-circuit error propagation, the exec code faithfully mirrors the original's control flow including Rust drop semantics, and the proof suite comprehensively covers error handling, resource lifecycle, and result categorization properties. The trust boundaries are well-chosen with appropriate contracts, and the documentation is thorough and honest about limitations. No remaining issues.
