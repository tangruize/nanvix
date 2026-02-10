# Review: kcall_wait_cond (claude-opus-4.6)

## Grade: A

## Previous Issues Assessment (R2 → R3)

### Medium (R2): CwOk don't-care value misleading
**Status: FIXED.** Line 513-514 of exec now has a clear comment: "Don't-care value: spec_stored_result ignores cw when gc fails, proven by lemma_get_cond_fail_ignores_cond_wait." This documents the intent and links to the proof. Using `CwOk` rather than `arbitrary()` is acceptable since Verus ghost code requires a concrete value and the proof demonstrates irrelevance.

### Low (R2): Missing continuation override lemmas for get_mutex/lock/put_guard
**Status: FIXED.** Three new lemmas added in the proof file:
- `lemma_get_mutex_overrides_stored_error` (proof lines 781-802): get_cond fails + get_mutex fails → GetMutexError returned.
- `lemma_lock_overrides_stored_error` (proof lines 810-831): get_cond fails + lock fails → LockGenericError returned.
- `lemma_put_guard_overrides_stored_error` (proof lines 839-860): get_cond fails + put_guard fails → PutGuardError returned.

All three are machine-verified (28 total VCs pass). Together with the existing `lemma_continuation_overrides_stored_error` (put_cond override), this provides complete coverage of all four continuation steps overriding a stored error.

### Low (R2): lemma_success_implies_all_predicates_set doc inaccuracy
**Status: FIXED.** Doc comment (proof lines 731-739) now accurately states: "When the result is Success, every step outcome must be the Ok variant. This is a necessary condition for the mutex protocol predicates [...] to hold. The predicates themselves are established by the external_body postconditions during exec verification in `wait_cond_model`." This is honest and precise about what the lemma proves vs. what the exec model proves.

## Issues Found

### Low

- **Location:** `lemma_lock_overrides_stored_error` (proof, lines 810-831)
- **Description:** This lemma only demonstrates the `LoGenericError` variant overriding a stored error. It does not cover `LoKilled` overriding a stored error (which is also a valid lock failure mode with infinite timeout). While `lemma_lock_error_propagates` (proof line 297) covers all non-Ok lock outcomes generically, the override lemma family would be more complete with an `LoKilled` instance.
- **Suggested Fix:** Either add a second lock-override lemma for `LoKilled`, or generalize the existing lemma to accept any non-Ok `lock_outcome` parameter (matching the pattern of `lemma_lock_error_propagates`). This is very minor — the generic propagation lemma already covers it.

## Positive Observations

- **Complete semantic fidelity.** The spec (`spec_stored_result` + continuation pipeline in `spec_wait_cond_result`) exactly mirrors the original source's control flow. I verified line-by-line against the original:
  - Timeout parsing (original lines 87-102) → `parse_timeout_model` + spec
  - take_mutex_guard short-circuit (original line 105) → TmgError early return
  - get_cond + cond.wait stored result (original lines 109-122) → `spec_stored_result` + `get_cond_and_wait_model`
  - Unconditional continuation (original lines 123-128) → put_cond/get_mutex/lock/put_guard pipeline regardless of stored result
  - Return stored result (original line 130) → `stored` at PgOk terminal
- **Strong exec-spec linkage.** The `wait_cond_model` postcondition (exec lines 582-586) is non-trivial: it asserts the exec result equals the spec function applied to ghost-tracked step outcomes. Verus verifies this by checking every control flow path.
- **Mutex protocol property.** The success postcondition (exec lines 588-592) proves `spec_mutex_released ∧ spec_cond_ref_released ∧ spec_mutex_reacquired`, connecting the uninterpreted predicates from external_body postconditions to the overall correctness property.
- **Safety contract modeled.** `wait_cond_model` requires `spec_wait_cond_safety_preconditions` and `spec_is_currently_running`, matching the original `unsafe` contract.
- **Comprehensive proof suite.** 25 proof lemmas covering: timeout parsing (3), error propagation per step (7), short-circuit (2), success biconditional (1), exhaustiveness (1), result preservation (1), reacquisition-no-timeout (1), error code linkage (1), architecture (1), safety preconditions (1), protocol predicates (1), cond_wait irrelevance on get_cond failure (1), continuation overrides stored result (4).
- **28 verification conditions pass** with zero assumes, zero errors, zero `unwrap`/`expect`/`panic!`.
- **Clean three-file split.** Spec contains only `open spec fn`, `uninterp spec fn`, and view types. Proof contains only `proof fn` lemmas. Exec contains enums, `external_body` trust boundaries, and verified functions. No mixing.

## Summary

All issues from the R1 and R2 reviews have been addressed. The verification is sound, complete for the module's scope, and semantically faithful to the original source. The only remaining issue is a cosmetic incompleteness in one override lemma (LoKilled variant not explicitly instantiated, though covered by the generic propagation lemma). The module demonstrates strong verification practices: meaningful exec-spec linkage, appropriate trust boundaries, comprehensive proof lemmas, and accurate documentation.
