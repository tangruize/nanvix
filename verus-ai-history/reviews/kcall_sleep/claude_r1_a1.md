# Review: kcall_sleep (claude-opus-4.6)

## Grade: C

## Issues Found

### Critical

- **Location:** `SleepResultModel::spec_view()` (exec, line 184) and `spec_sleep_result` (spec, line 174)
  - **Description:** `Killed` interruption is incorrectly classified as success. `SleepResultModel::Killed` maps to `SleepResultView::Interrupted` via `spec_view()`, and then `spec_sleep_result` maps all `Interrupted` results to `SleepResultView::Success`. However, in the original code (sleep.rs:62-66), only `InterruptReason::TimedOut` is mapped to `Ok(())`; `Interrupted(Killed)` falls through to `Err(error) => Err(error)` and is propagated as an error. This means the spec claims `Killed` is a success, contradicting the original semantics.
  - **Suggested Fix:** `SleepResultView` must distinguish `TimedOut` from `Killed`. Either: (a) add a `Killed` variant to `SleepResultView` and map it correctly in `spec_sleep_result`, or (b) split `Interrupted` into `InterruptedTimedOut` and `InterruptedKilled` variants with only `InterruptedTimedOut` mapping to `Success`.

- **Location:** `sleep_model()` (exec, lines 356-363)
  - **Description:** The exec model does not implement the result classification from the original code. It returns `pm_result` directly (line 362) without mapping `TimedOut` to success or propagating `Killed` as an error. The original code has an explicit `match` that converts `Ok(()) | Interrupted(TimedOut)` → `Ok(())` and propagates all other errors. The comment on line 361 says "Classify the result" but no classification occurs.
  - **Suggested Fix:** Apply the classification inside `sleep_model`: match on `pm_result`, return `SleepResultModel::Ok` for `Ok` and `TimedOut`, and propagate `Killed` and `GenericError` as errors. This must match the original's 3-arm match pattern.

### High

- **Location:** `sleep_model()` postcondition (exec, lines 345-348)
  - **Description:** The postcondition is too weak. It only specifies behavior when `checked_add` fails (overflow → `GenericError`). It says nothing about the success path: when `checked_add` succeeds, there is no postcondition relating the result to `spec_sleep_result` or to the PM result. This means the verification proves almost nothing about the normal (non-overflow) execution path.
  - **Suggested Fix:** Add a postcondition connecting the result to `spec_sleep_result`, e.g.: `spec_sleep_success_condition(...) ==> result.spec_view() == spec_sleep_result(now.spec_view(), seconds as nat, nanoseconds as nat, pm_result_view)`. This requires threading the PM result through the postcondition.

- **Location:** `classify_sleep_result()` (exec, lines 305-314)
  - **Description:** This function is dead code — it is defined but never called by `sleep_model` or `sleep_end_to_end`. Its logic (`Ok | TimedOut → true`, `_ → false`) is also insufficient because it doesn't distinguish `Killed` from `GenericError` in its return type (just returns `bool`). The original code propagates the specific error variant, not just a boolean.
  - **Suggested Fix:** Either integrate proper classification logic directly into `sleep_model`, or redesign `classify_sleep_result` to return a `SleepResultModel` with the correct mapping and call it from `sleep_model`.

- **Location:** `lemma_timed_out_is_success` (proof, lines 92-105)
  - **Description:** This lemma is named "timed_out_is_success" but actually proves that ALL `Interrupted` results (including `Killed`) map to success, because `SleepResultView::Interrupted` covers both. The lemma is technically correct with respect to the (incorrect) spec, but it proves a property that contradicts the original code's behavior.
  - **Suggested Fix:** After fixing `SleepResultView` to distinguish `TimedOut` from `Killed`, update this lemma to only prove that `TimedOut` maps to success, and add a separate lemma proving `Killed` propagates as an error.

### Medium

- **Location:** `SleepResultView` (spec, lines 72-79)
  - **Description:** The view type collapses `TimedOut` and `Killed` into a single `Interrupted` variant, losing the semantic distinction required by the original code. This is the root cause of the Critical issue above. The abstraction is too coarse for the properties being verified.
  - **Suggested Fix:** Refine to three error variants: `InterruptedTimedOut`, `InterruptedKilled`, and `GenericError`, matching the original's four-outcome model (success, timed-out-as-success, killed-as-error, generic-error).

- **Location:** `sleep_model()` signature (exec, line 340)
  - **Description:** Parameter types differ from original: `sleep_model` takes `(u64, u32)` while the original takes `(usize, usize)`. On x86-32 (Nanvix's target), `usize` is 32 bits. The `seconds as u64` cast in the original widens from 32 to 64 bits, meaning `seconds` can never exceed `u32::MAX`. The model accepts the full `u64` range, which is a domain widening that could mask overflow issues at the cast boundary.
  - **Suggested Fix:** Either add a precondition `seconds <= u32::MAX as u64` to match the 32-bit origin, or document that the model intentionally verifies the post-cast domain and that cast safety is verified elsewhere.

- **Location:** `sleep_end_to_end()` postcondition (exec, lines 390-399)
  - **Description:** `sleep_end_to_end` has no postconditions beyond what `sleep_model` provides (which is already too weak). As the "end-to-end" entry point, it should establish the strongest guarantees about the complete sleep operation.
  - **Suggested Fix:** Add postconditions that fully characterize the function's behavior, tying the result to `spec_sleep_result`.

### Low

- **Location:** `sleep_model()` (exec, line 366)
  - **Description:** The error code `22i32` is hardcoded. While it matches `ErrorCode::InvalidArgument`, it would be more maintainable and verifiable to reference `ERROR_CODE_INVALID_ARGUMENT()` symbolically and prove the connection.
  - **Suggested Fix:** Use a verified constant or add an assertion that `22i32 as int == ERROR_CODE_INVALID_ARGUMENT()`.

- **Location:** `process_manager_sleep()` postcondition (exec, lines 236-241)
  - **Description:** The external body for `process_manager_sleep` has no postcondition constraining what results it can return. This means the verification cannot reason about PM outcomes at all for the success path. A minimal postcondition (e.g., result is one of the defined variants) would strengthen the model.
  - **Suggested Fix:** Add a postcondition such as `matches!(result, SleepResultModel::Ok | SleepResultModel::TimedOut | SleepResultModel::Killed | SleepResultModel::GenericError { .. })` (trivially true for the enum, but useful as documentation and for future refinement).

- **Location:** Documentation (exec, line 29)
  - **Description:** The doc header comments say "Maps Ok(()) and Interrupted(TimedOut) to success; propagates other errors" which is the correct behavior of the original, but the verified code does not actually implement this mapping. The documentation is misleading about what the model proves.
  - **Suggested Fix:** Either fix the code to match the documentation, or update the documentation to accurately reflect what is currently verified.

## Positive Observations

- **Clean separation:** The spec/proof/exec split is well-organized, with clear file boundaries and include-based composition.
- **Thorough documentation:** Trust boundaries (T1-T4) are explicitly identified and documented with an API mapping table. The module-level doc comment is comprehensive.
- **Duration normalization:** `duration_new()` correctly models `Duration::new()`'s carry semantics and proves well-formedness (`lemma_duration_new_wf`).
- **Alarm well-formedness:** `lemma_alarm_wf` correctly proves that `checked_add_duration` preserves the nanoseconds invariant via case split on carry.
- **Overflow detection:** The overflow path (checked_add fails → InvalidArgument) is correctly modeled and verified.
- **Zero-duration edge case:** `lemma_zero_duration_always_valid` verifies that sleeping for zero time always succeeds, which is a useful boundary property.
- **Result dichotomy:** `lemma_sleep_result_dichotomy` proves exhaustiveness of the result classification (though the classification itself is wrong).
- **Verification passes cleanly:** 13 verified, 0 errors — the proofs are mechanically sound with respect to the stated specs.
- **External body contracts:** `checked_add_duration` has a strong bidirectional postcondition linking success/failure to `spec_checked_add_succeeds`.

## Summary

The verification has a solid structural foundation with clean spec/proof/exec separation, thorough documentation, and correct modeling of Duration normalization and overflow detection. However, it contains two critical semantic errors that undermine the core correctness claim:

1. **The `Killed` vs `TimedOut` distinction is lost** in the `SleepResultView` abstraction, causing the spec to incorrectly classify `Killed` interruptions as success. The original code explicitly only treats `TimedOut` as success.

2. **The exec model skips the result classification entirely**, returning the raw PM result without applying the original's `match` logic. The `classify_sleep_result` helper exists but is dead code.

Additionally, the `sleep_model` postcondition is too weak to establish meaningful properties about the non-overflow path, meaning the verification mainly proves overflow detection correctness while leaving the normal execution path essentially unverified.

**Recommendations (priority order):**
1. Refine `SleepResultView` to distinguish `TimedOut` from `Killed`.
2. Implement result classification in `sleep_model` matching the original's 3-arm match.
3. Strengthen `sleep_model`'s postcondition to cover the success path.
4. Update all proof lemmas to reflect the corrected semantics.
5. Add a postcondition to `sleep_end_to_end`.
