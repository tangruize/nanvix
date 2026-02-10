# Review: kcall_sleep (claude-opus-4.6)

## Grade: B+

## Previous Issues Status

### Critical Issues — Both FIXED

1. **`Killed` incorrectly classified as success** — **FIXED.** The spec now has two separate types:
   `PmSleepResultView` (4 variants: `PmOk`, `PmTimedOut`, `PmKilled`, `PmGenericError`) for raw PM
   outcomes, and `SleepResultView` (3 variants: `Success`, `KilledError`, `GenericError`) for
   classified final results. `spec_classify_pm_result` correctly maps `PmKilled → KilledError`
   (spec line 195), not to `Success`. Verified by `lemma_killed_is_error` (proof lines 106-124)
   which proves `PmKilled` is error AND NOT success. Well done.

2. **`sleep_model()` didn't implement result classification** — **FIXED.** `sleep_model` now
   calls `classify_pm_result(pm_result)` on exec line 436. `classify_pm_result` (exec lines
   353-373) implements the exact 3-arm match: `Ok→Ok`, `TimedOut→Ok`, `Killed→Killed`,
   `GenericError→GenericError`. The postcondition of `classify_pm_result` ties the result to
   `spec_classify_pm_result` via `spec_classified_view` (exec line 355).

### High Issues — All FIXED

3. **`sleep_model()` postcondition too weak** — **PARTIALLY FIXED.** The postcondition now
   covers both paths (exec lines 404-420): overflow → `GenericError(InvalidArgument)`, and
   success path → result is `{Ok, Killed, GenericError}` with `TimedOut` excluded. See
   remaining issue below for remaining gap.

4. **`classify_sleep_result()` was dead code** — **FIXED.** Renamed to `classify_pm_result`,
   now returns `SleepResultModel` (not `bool`), and is called from `sleep_model` (exec line 436).

5. **`lemma_timed_out_is_success` proved wrong property** — **FIXED.** Now proves only
   `PmTimedOut` maps to success (proof line 94). New `lemma_killed_is_error` (proof lines
   106-124) proves `PmKilled` maps to `KilledError`, is error, and is NOT success — exactly
   right.

### Medium Issues — All ADDRESSED

6. **`SleepResultView` too coarse** — **FIXED.** Split into `PmSleepResultView` (4 PM
   variants) and `SleepResultView` (3 classified variants: `Success`, `KilledError`,
   `GenericError`). The two-tier abstraction is cleaner than a single overloaded type.

7. **Parameter types `(u64, u32)` vs original `(usize, usize)`** — **ADDRESSED via
   documentation.** Trust Boundary T5 (exec lines 76-80) explicitly documents that `usize` is
   32-bit on Nanvix's x86-32 target and that cast safety is at the ABI boundary. No
   precondition `seconds <= u32::MAX as u64` was added, but the rationale is documented.
   Acceptable.

8. **`sleep_end_to_end()` no postconditions** — **NOT FIXED.** See remaining issue below.

### Low Issues — All FIXED

9. **Hardcoded error code `22i32`** — **FIXED.** Explicit proof assertion added:
   `assert(22i32 as int == ERROR_CODE_INVALID_ARGUMENT())` (exec line 441).

10. **`process_manager_sleep()` no postcondition** — **FIXED.** Added exhaustive `matches!`
    postcondition (exec lines 277-278).

11. **Documentation misleading** — **FIXED.** Module documentation now correctly describes
    the 3-arm match (exec lines 17-20), Killed-as-error (line 32-33), and full exec model
    correctness (lines 44-46).

## New/Remaining Issues

### Medium

- **Location:** `sleep_model()` success-path postcondition (exec, lines 415-420)
  - **Description:** The success-path postconditions are structural (`matches!` on variants)
    but don't use `spec_classified_view()` to connect the result to the spec-level
    classification. The overflow path does use it (line 407-409:
    `result.spec_classified_view() == SleepResultView::GenericError { ... }`), but the success
    path only says the result is one of `{Ok, Killed, GenericError}`. Since the PM result is
    obtained internally and is non-deterministic, the caller cannot predict the exact outcome.
    However, the postcondition could still state:
    `spec_sleep_success_condition(...) ==> result.spec_classified_view() == spec_classify_pm_result(result.spec_pm_view())`
    which proves that whatever PM result was obtained, the classification was applied correctly.
    This is not tautological — it confirms the 3-arm match was applied, not bypassed.
  - **Suggested Fix:** Add:
    `spec_sleep_success_condition(now.spec_view(), seconds as nat, nanoseconds as nat) ==> result.spec_classified_view() == spec_classify_pm_result(result.spec_pm_view())`

### Low

- **Location:** `sleep_end_to_end()` (exec, lines 467-476)
  - **Description:** Still has no `ensures` clause. As the top-level entry point modeling the
    complete kcall, callers get zero verified guarantees from its signature. At minimum, it
    should expose the structural invariant that `TimedOut` never appears in the output:
    `!matches!(result, SleepResultModel::TimedOut)`. This propagates the core invariant
    established by the 3-arm match classification.
  - **Suggested Fix:** Add:
    ```
    ensures
        !matches!(result, SleepResultModel::TimedOut),
        result.spec_classified_view() == spec_classify_pm_result(result.spec_pm_view()),
    ```

- **Location:** `lemma_sleep_result_trichotomy` naming (proof, line 175)
  - **Description:** The lemma name says "trichotomy" (three-way), but the ensures clause
    uses `spec_is_success(...) || spec_is_error(...)` where `spec_is_error` includes both
    `killed` and `generic_error`. So it actually proves a dichotomy (success vs error), not a
    trichotomy. The proof body correctly case-splits into three categories. The name is
    slightly misleading.
  - **Suggested Fix:** Either rename to `lemma_sleep_result_exhaustive` or strengthen the
    ensures to: `spec_is_success(...) ^^ spec_is_killed(...) ^^ spec_is_generic_error(...)`.

## Positive Observations

- **Correct 3-arm match modeling:** The core semantic issue is resolved. `classify_pm_result`
  (exec lines 367-372) exactly mirrors the original's match statement, and the
  postconditions verify it against `spec_classify_pm_result`.
- **Two-tier result abstraction:** The `PmSleepResultView`/`SleepResultView` split is a
  clean design that separates PM outcomes from classified kcall results. This is actually
  better than having a single overloaded type.
- **Strong new lemmas:** `lemma_killed_is_error` (proof lines 106-124) proves three
  properties at once (is killed, is error, is NOT success). `lemma_success_only_from_ok_or_timed_out`
  (proof lines 232-244) proves a biconditional: success ⟺ PM result was `PmOk` or
  `PmTimedOut`. This fully characterizes when success occurs.
- **Proof count increased:** 15 verified (up from 13), reflecting the added lemmas without
  any regressions.
- **Trust boundary documentation:** T5 (usize→u64 cast) is now explicitly documented with
  rationale for the design choice.
- **Error code assertion:** The connection between `22i32` and `ERROR_CODE_INVALID_ARGUMENT()`
  is now machine-checked in proof mode.
- **No `assume` statements:** All three files are free of `assume` — no unjustified
  assumptions anywhere in the module.
- **Documentation accurately reflects code:** The module-level doc comment now correctly
  describes the 3-arm match, the Killed-as-error behavior, and the full exec model
  correctness.

## Summary

The prover has genuinely addressed all critical and high-priority issues from Round 1. The
two critical semantic errors (Killed-as-success misclassification, and missing result
classification in the exec model) are properly fixed with correct spec types, exec
implementation, and proof lemmas. The verification count increased from 13 to 15, reflecting
added coverage without regressions.

The remaining issues are minor: the success-path postcondition of `sleep_model` could be
strengthened to expose the spec-level classification relationship, and `sleep_end_to_end`
still lacks postconditions. These do not affect soundness — the internal proofs are correct —
but they limit the usefulness of the verified interface for downstream callers.

**Grade rationale:** B+ reflects a verification that is semantically correct, well-structured,
and sound, with good coverage of the key safety properties. The deduction from A- is for
the postcondition gaps that prevent callers from fully leveraging the proven properties.
