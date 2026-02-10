# Review: kcall_wait_cond (claude-opus-4.6)

## Grade: A-

## Previous Issues Assessment

### Critical (R1): Semantic divergence — get_cond failure short-circuited pipeline
**Status: FIXED.** The spec now uses `spec_stored_result` (spec lines 279-298) to combine get_cond and cond.wait into a stored result, and `spec_wait_cond_result` (spec lines 340-382) continues the put_cond → get_mutex → lock → put_guard pipeline unconditionally, returning the stored result only when all continuation steps succeed. The exec model's `get_cond_and_wait_model` helper (exec lines 496-535) correctly stores the error and continues. New proof lemmas `lemma_continuation_overrides_stored_error` (proof line 187) and `lemma_get_cond_fail_ignores_cond_wait` (proof line 150) verify the semantics. Verified against original source lines 109-130 — the model now faithfully captures the "store-then-continue" pattern.

### High (R1): Missing exec-spec equivalence proof
**Status: FIXED.** `wait_cond_model` now returns `Ghost<WaitCondGhostState>` capturing all step outcomes (spec lines 201-216), and its postcondition (exec lines 582-586) asserts `ret.0.spec_view() == spec_wait_cond_result(timeout_s as nat, timeout_ns as nat, ret.1@.tmg, ret.1@.gc, ret.1@.cw, ret.1@.pc, ret.1@.gm, ret.1@.lo, ret.1@.pg)`. This is the key exec-spec linkage — Verus verifies this postcondition against the actual exec control flow, not a trivially-true tautology. Each early-return path constructs ghost state with the actual step views observed, and don't-care values for unreached steps (which the spec ignores due to short-circuit semantics).

### Medium (R1): pid/tid absent from model
**Status: FIXED.** `wait_cond_model` now takes `pid: Ghost<u32>` and `tid: Ghost<u32>` (exec lines 574-575). `take_mutex_guard_model` takes `pid: Ghost<u32>, tid: Ghost<u32>` (exec lines 344-345). New spec predicate `spec_is_currently_running` (spec line 469) ensures the identifiers match the running thread. Ghost parameters correctly model that pid/tid are used for the ProcessManager call but don't directly affect the return value.

### Medium (R1): Uninterpreted predicates were dead code
**Status: FIXED.** `wait_cond_model` postcondition (exec lines 588-592) now asserts that on success, `spec_mutex_released`, `spec_cond_ref_released`, and `spec_mutex_reacquired` all hold. New proof `lemma_success_implies_all_predicates_set` (proof lines 742-775) decomposes this. The predicates are now connected to the verified correctness properties through the external_body postconditions.

### Low (R1): Safety preconditions unused
**Status: FIXED.** `wait_cond_model` now requires `spec_wait_cond_safety_preconditions(pid@ as nat, tid@ as nat)` (exec line 578).

### Low (R1): LockTimedOut dead branch
**Status: ADDRESSED.** Branch retained for exhaustive matching with a clear unreachability comment (exec lines 697-700). Acceptable approach — removing the variant from the enum would be cleaner but is not strictly necessary since the postcondition proves it unreachable.

### Low (R1): u32 vs usize triviality
**Status: UNCHANGED (acceptable).** The `requires` clauses that checked `<= USIZE_MAX_X86_32()` have been removed since they were trivially true for u32. This is fine — the architectural equivalence is documented and proven in `lemma_architecture_guard`.

## Issues Found

### Medium

- **Location:** `get_cond_and_wait_model` (exec, line 513)
- **Description:** When `get_cond` fails, the ghost `cw_view` is set to `CondWaitOutcomeView::CwOk` as a don't-care value. While functionally correct (the spec's `spec_stored_result` ignores the cond_wait outcome when get_cond fails, and `lemma_get_cond_fail_ignores_cond_wait` proves irrelevance), this specific value choice is semantically misleading — it suggests `cond.wait` was called and succeeded when in fact it was never invoked. A more honest choice would be to use `arbitrary()` or document why CwOk was chosen. This is a code quality issue, not a soundness issue.
- **Suggested Fix:** Use Verus `arbitrary()` for the don't-care value, or add a comment explaining the choice is irrelevant per `lemma_get_cond_fail_ignores_cond_wait`.

### Low

- **Location:** Proof file — missing continuation override lemma for get_mutex/lock/put_guard
- **Description:** `lemma_continuation_overrides_stored_error` (proof lines 187-209) only demonstrates that put_cond error overrides a stored get_cond error. There are no analogous lemmas showing get_mutex error, lock error, or put_guard error also override a stored get_cond error (or a stored cond_wait error). While these properties follow from the existing per-step propagation lemmas (which are now generalized over get_cond_outcome/cond_wait_outcome), explicit lemmas showing "get_cond fails + get_mutex fails → GetMutexError returned" would strengthen the proof coverage for the key semantic property that motivated the R1 critical fix.
- **Suggested Fix:** Add 2-3 additional `lemma_continuation_overrides_*` lemmas showing different continuation steps overriding stored results. Alternatively, a single generalized lemma showing any continuation error overrides any stored result.

- **Location:** `lemma_success_implies_all_predicates_set` (proof, lines 742-775)
- **Description:** The lemma proves that success implies all step outcomes were Ok, but does not explicitly conclude `spec_mutex_released`, `spec_cond_ref_released`, or `spec_mutex_reacquired`. The connection from step-Ok to these predicates exists only via external_body postconditions used during exec verification (not in a standalone proof). The doc comment claims it "verifies that Success implies all three uninterpreted predicates must have been established" but the ensures clause only proves the step variant matches, not the predicates themselves. This is a documentation accuracy issue — the actual predicate implications are proven by `wait_cond_model`'s postcondition, not this lemma.
- **Suggested Fix:** Either extend the ensures clause to explicitly include the predicate conclusions (which would require taking the mutex/cond addresses as parameters and assuming the external_body postconditions as axioms), or adjust the doc comment to accurately describe what the lemma proves: "Success implies all steps returned Ok."

## Positive Observations

- **Semantic accuracy greatly improved.** The "store-then-continue" pipeline pattern is now correctly modeled with `spec_stored_result` and the unconditional continuation in `spec_wait_cond_result`. This matches the original source exactly.
- **Exec-spec equivalence is now machine-checked.** The `WaitCondGhostState` approach with ghost tracking of all step outcomes is well-designed. The postcondition on `wait_cond_model` (lines 582-586) is the strongest possible assertion — the exec result equals the spec applied to the actual observed step outcomes.
- **New helper `get_cond_and_wait_model` is well-abstracted.** Cleanly encapsulates the original code's lines 109-122 block, with its own postcondition (`spec_stored_result`) verified independently.
- **Proof lemmas generalized.** Several lemmas (`lemma_put_cond_error_propagates`, `lemma_get_mutex_error_propagates`, `lemma_lock_error_propagates`, `lemma_put_guard_error_propagates`) are now universally quantified over `get_cond_outcome` and `cond_wait_outcome`, correctly reflecting that continuation errors are independent of the stored result.
- **New proofs for the semantic fix.** `lemma_get_cond_fail_ignores_cond_wait` and `lemma_continuation_overrides_stored_error` directly verify the key properties of the corrected model.
- **Mutex protocol on success.** The postcondition `spec_is_success(ret.0.spec_view()) ==> spec_mutex_released ∧ spec_cond_ref_released ∧ spec_mutex_reacquired` is a valuable safety property.
- **25 verification conditions pass** with zero assumes, zero errors.
- **Clean code.** No `assume`, `expect`, `unwrap`, or `panic!` in any file.

## Summary

All issues from the R1 review have been substantively addressed. The critical semantic divergence (get_cond short-circuit) is fully corrected — the spec and exec model now faithfully capture the original code's "store result, continue pipeline unconditionally, return stored result only if continuation succeeds" semantics. The exec-spec equivalence is machine-checked via ghost state. Safety preconditions, pid/tid parameters, and mutex protocol predicates are properly integrated.

The remaining issues are minor: a cosmetic concern about don't-care value choice, incomplete lemma coverage for continuation overrides (the property holds, just not all cases are explicitly instantiated), and a doc comment that slightly overstates what a lemma proves. None affect soundness. The verification is solid and comprehensive for this module's scope.
