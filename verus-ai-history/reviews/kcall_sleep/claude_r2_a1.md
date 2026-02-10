# Review: kcall_sleep (claude-opus-4.6)

## Grade: A

## Issues Found

### Critical

_None._

### High

_None._

### Medium

1. **Trivially true postcondition on `process_manager_sleep`**
   - **Location:** `sleep.rs` (exec), line 284–285, `process_manager_sleep()` external_body
   - **Description:** The `ensures` clause `matches!(result, SleepResultModel::Ok | SleepResultModel::TimedOut | SleepResultModel::Killed | SleepResultModel::GenericError { .. })` is trivially true for any value of the `SleepResultModel` enum — it matches every variant. This adds no verification value beyond documentation. While the nondeterminism of the PM is intentional (any variant can occur depending on runtime state), a trivially true postcondition is indistinguishable from having no postcondition.
   - **Suggested Fix:** Either remove the trivially true postcondition and add a comment explaining the intentional nondeterminism, or add a lightweight constraint such as `matches!(result, SleepResultModel::GenericError { error_code }) ==> error_code != 0i32` if the PM guarantees non-zero error codes. Alternatively, keep as-is with a comment `// Intentionally trivial: PM result is nondeterministic`.

2. **Ghost return type adds complexity for modest verification benefit**
   - **Location:** `sleep.rs` (exec), line 406, `sleep_model()` return type
   - **Description:** `sleep_model` returns `(SleepResultModel, Ghost<PmSleepResultView>)` to expose the original PM result in postconditions. While this enables the non-tautological postcondition tying the exec result to `spec_sleep_result`, the pattern requires callers to handle the ghost tuple, and the `sleep_end_to_end` wrapper returns a triple `(SleepResultModel, Ghost<PmSleepResultView>, Ghost<SystemTimeView>)`. The verification value is real — it prevents the postcondition from being trivially satisfied by just matching the classified view — but the ergonomic cost should be noted.
   - **Suggested Fix:** No change needed; this is an acceptable engineering trade-off. Consider adding a wrapper function that discards the ghosts for downstream exec callers.

### Low

1. **x86-32 precondition baked into model**
   - **Location:** `sleep.rs` (exec), line 410, `sleep_model()` precondition
   - **Description:** The precondition `seconds <= u32::MAX as u64` encodes the assumption that Nanvix targets x86-32 where usize is 32-bit. This is well-documented (Trust Boundary T5) but would need updating if Nanvix ever targets 64-bit architectures.
   - **Suggested Fix:** No change needed now. Add a comment referencing the architecture assumption for future maintainability, or define a spec constant `MAX_USIZE()` to centralize the assumption.

2. **`lemma_sleep_result_exhaustive` named inconsistently with doc header**
   - **Location:** `sleep.proof.rs` (proof), line 176
   - **Description:** The function name is `lemma_sleep_result_exhaustive` but the doc header comments and the exec file reference it as "result trichotomy" (`lemma_sleep_result_trichotomy`). The proof file's naming diverges from the documentation in the exec file's header (line 38: `lemma_sleep_result_trichotomy`).
   - **Suggested Fix:** Rename to `lemma_sleep_result_trichotomy` to match the documentation, or update the doc header to say `lemma_sleep_result_exhaustive`.

3. **`spec_compute_alarm` lacks explicit well-formedness in its ensures**
   - **Location:** `sleep.spec.rs` (spec), line 167, `spec_compute_alarm()`
   - **Description:** The spec function `spec_compute_alarm` has a `recommends` clause for `spec_checked_add_succeeds` but does not explicitly ensure the result is well-formed (`spec_system_time_wf`). The well-formedness is instead proven externally in `lemma_alarm_wf`. This is acceptable but means callers must invoke the lemma separately rather than getting the property from the spec directly.
   - **Suggested Fix:** No change strictly needed; the current design separates concerns properly. Optionally, add a `recommends`-gated assertion or a note pointing to `lemma_alarm_wf`.

4. **Ghost value on overflow path is arbitrary**
   - **Location:** `sleep.rs` (exec), line 464
   - **Description:** On the overflow path, the ghost PM value is `Ghost(PmSleepResultView::PmOk)` with a comment "Ghost PM value is arbitrary on the overflow path." The postconditions are vacuously true for the success-path ensures (guarded by `spec_sleep_success_condition`), so this is correct. However, using an arbitrary value like `PmOk` when no PM call occurred could be confusing to readers.
   - **Suggested Fix:** Consider using a more obviously "dummy" value or adding a stronger comment. Alternatively, restructure postconditions so the ghost is `Option<PmSleepResultView>` with `None` on the overflow path.

## Positive Observations

- **Thorough documentation**: The module-level doc comment (lines 1–98 in sleep.rs) is exemplary. It documents verified properties, trust boundaries, error reason abstraction, and the API mapping table. This makes the verification model auditable.
- **Clean spec/proof/exec split**: Spec types and functions are in `sleep.spec.rs`, proof lemmas in `sleep.proof.rs`, and exec code with external bodies in `sleep.rs`. The `include!` pattern keeps them in one Verus module while maintaining file separation.
- **Complete path coverage**: All four PM result variants (Ok, TimedOut, Killed, GenericError) and the overflow path are explicitly verified. The `lemma_sleep_result_exhaustive` proves mutual exclusion and completeness.
- **Strong equivalence**: The exec model faithfully mirrors the original's control flow, including the critical 3-arm match where `TimedOut` is folded into `Ok`. The `classify_pm_result` function is a direct transliteration of lines 62–66 of the original.
- **Trust boundaries are well-justified**: Each `external_body` function (T1–T4) has clear documentation of what is trusted and why. The postconditions on `checked_add_duration` are particularly well-specified, tying the result to both `spec_checked_add_succeeds` and `spec_compute_alarm`.
- **Key safety property proven**: `lemma_success_only_from_ok_or_timed_out` proves that success occurs if and only if the PM returned PmOk or PmTimedOut — this is the critical correctness property distinguishing TimedOut (success) from Killed (error).
- **Duration normalization verified inline**: Rather than trusting `Duration::new()` as external, the model explicitly verifies the nanosecond carry normalization, which is a stronger approach.
- **Verification passes**: All 15 verification conditions pass cleanly with no errors.

## Summary

This is a high-quality verification of a relatively small but safety-critical kernel call. The model correctly captures all code paths of the original `sleep()` function: duration construction with nanosecond normalization, overflow detection via `checked_add_duration`, and the crucial 3-arm match that distinguishes `TimedOut` (treated as success) from `Killed` (propagated as error). The spec/proof/exec split is clean, trust boundaries are well-documented with justified `external_body` functions, and the proof lemmas cover exhaustiveness, mutual exclusion, and path-specific correctness. The only notable gap is the trivially true postcondition on `process_manager_sleep` and a naming inconsistency between the proof and documentation. No soundness issues were found.
