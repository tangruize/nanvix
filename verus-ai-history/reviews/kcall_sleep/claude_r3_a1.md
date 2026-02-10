# Review: kcall_sleep (claude-opus-4.6)

## Grade: A

## Issues Found

### Critical

None.

### High

None.

### Medium

- **Location:** `process_manager_sleep()` postcondition (exec: sleep.rs:314-316)
  - **Description:** The `ensures` clause `matches!(result, SleepResultModel::Ok | SleepResultModel::TimedOut | SleepResultModel::Killed | SleepResultModel::GenericError { .. })` is a tautology — it matches every variant of `SleepResultModel` and thus constrains nothing. Verus may optimize it away, but it gives a false impression of specification strength. The external body provides no meaningful postcondition about the relationship between the `alarm` input and the result (e.g., that `Ok` or `TimedOut` can only occur for well-formed alarms). While timing/liveness is explicitly out of scope, a stronger postcondition such as ensuring that `GenericError` carries a valid error code (e.g., `error_code != 0`) would add non-trivial value without importing PM ghost state.
  - **Suggested Fix:** Either (a) remove the tautological `matches!` ensures and add a comment that the external body is intentionally unconstrained, or (b) add a minimal non-tautological postcondition, e.g., `matches!(result, SleepResultModel::GenericError { error_code }) ==> error_code != 0i32`.

- **Location:** `spec_checked_add_succeeds()` (spec: sleep.spec.rs:168-173)
  - **Description:** The spec models overflow as `new_seconds <= u64::MAX`, which assumes `SystemTime` internally uses `u64` for seconds and that overflow is purely a seconds-field overflow. If the actual `SystemTime::checked_add_duration()` has a different maximum (e.g., a max representable time smaller than `u64::MAX` seconds), the `checked_add_duration` external body's bidirectional contract (`succeeds <==> result.is_some()`) would be unsound. This is mitigated by the external body trust boundary, but the spec could note that this is an assumption about the internal representation.
  - **Suggested Fix:** Add a comment in `spec_checked_add_succeeds` documenting the assumption that `SystemTime` uses `u64` seconds and that the checked_add overflow condition is exactly seconds overflow. Alternatively, verify this against the actual `SystemTime` implementation in `src/libs/sys/`.

### Low

- **Location:** `sleep_model()` signature (exec: sleep.rs:437)
  - **Description:** The function returns `(SleepResultModel, Ghost<PmSleepResultView>)` instead of the original's `Result<(), SleepError>`. While necessary for connecting the classified result to the original PM outcome in postconditions, the return type divergence from the original means the exec model is not directly linkable as a drop-in replacement. This is well-documented but worth flagging for maintainability — if the original function's signature changes, the model's signature must be updated in a non-obvious way.
  - **Suggested Fix:** No code change needed. Consider adding a comment in the API Mapping table noting the return type divergence and why it exists.

- **Location:** `sleep_model()` precondition (exec: sleep.rs:441)
  - **Description:** The precondition `seconds <= u32::MAX as u64` encodes the Trust Boundary T5 assumption that `usize` is 32-bit on x86-32. If Nanvix ever targets a 64-bit architecture, this precondition would be too restrictive (rejecting valid inputs) or too lax (the cast semantics change). This is documented but not machine-checked against the target architecture.
  - **Suggested Fix:** No immediate change needed given x86-32-only targeting. If multi-arch support is added, this precondition and T5 documentation should be revisited.

- **Location:** `sleep_end_to_end()` (exec: sleep.rs:519-554)
  - **Description:** This function duplicates all postconditions from `sleep_model()` with the additional ghost `now` tracking. If `sleep_model()` postconditions are updated, `sleep_end_to_end()` must be updated in sync. This is minor duplication but creates a maintenance burden.
  - **Suggested Fix:** Consider whether `sleep_end_to_end()` could delegate postcondition specification to a shared spec function to reduce duplication.

## Positive Observations

- **Excellent documentation:** The module-level documentation is exceptionally thorough. Trust boundaries (T1-T5) are explicitly enumerated with clear justification. Out-of-scope properties (timing, liveness) are documented with reasoning for why they belong to other modules. The API mapping table provides clear traceability from original to model.
- **Sound external body contracts:** The three external bodies (`clock_now`, `checked_add_duration`, `process_manager_sleep`) have well-chosen contracts. `checked_add_duration` in particular has a strong bidirectional specification tying `Some`/`None` to `spec_checked_add_succeeds`, with well-formedness and value equality guarantees on the `Some` path.
- **Complete coverage of result classification:** All four PM outcomes (Ok, TimedOut, Killed, GenericError) are individually proven via dedicated lemmas (`lemma_timed_out_is_success`, `lemma_killed_is_error`, `lemma_pm_success_is_success`, `lemma_pm_error_propagates`). The `lemma_sleep_result_exhaustive` proves both exhaustiveness and mutual exclusion.
- **`lemma_error_code_matches` links spec to concrete enum:** The proof that `ERROR_CODE_INVALID_ARGUMENT() == ErrorCode::InvalidArgument as int` prevents silent drift if the error code value changes. This is a good practice.
- **Clean spec/proof/exec separation:** Spec types and functions are purely declarative. Proof lemmas are isolated in the proof file. Exec code contains only model structs, external bodies, and verified functions. The `include!` mechanism provides clean file-level separation.
- **`duration_new` is fully verified (not external_body):** The Duration normalization logic is modeled and verified rather than trusted, reducing the trust surface. The carry/modulus logic is proven correct.
- **Ghost-based PM result tracking:** The use of `Ghost<PmSleepResultView>` to capture the original PM result before classification enables the postcondition to tie the final result to `spec_sleep_result` non-tautologically, avoiding a circular proof where the spec just reflects back the exec result.
- **Verification passes cleanly:** 16 verified, 0 errors.

## Summary

This is a high-quality verification of the `kcall_sleep` module. The single original function (`pub unsafe fn sleep`) is fully modeled with complete coverage of all control flow paths: Duration construction with nanosecond normalization, checked alarm computation with overflow detection, and the critical 3-arm match that classifies PM results (Ok/TimedOut → success, Killed/GenericError → error propagation).

The specification is well-calibrated — strong enough to prove meaningful properties (result classification, error code correctness, exhaustiveness, mutual exclusion) while appropriately abstracting away timing and liveness concerns that belong to other modules. The trust boundaries are minimal and well-justified: `clock_now()`, `checked_add_duration()`, and `process_manager_sleep()` are the only external bodies, and each has documented rationale.

The two medium-priority items are: (1) the tautological postcondition on `process_manager_sleep`, which should be either removed or strengthened to add real value; and (2) the `spec_checked_add_succeeds` overflow model which assumes `SystemTime` uses `u64` seconds — an assumption that should be documented or validated. Neither issue threatens the soundness of what IS proven, but they represent missed opportunities for stronger guarantees at the trust boundaries.

Overall, this verification successfully establishes that the sleep kcall correctly classifies all PM outcomes, correctly handles overflow, and returns the proper error codes — which are the essential safety properties for this thin kcall wrapper.
