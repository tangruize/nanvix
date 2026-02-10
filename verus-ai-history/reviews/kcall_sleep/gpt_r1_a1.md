# Review: kcall_sleep (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- **Location:** `sleep_model` postconditions (exec: `verus/split/kernel/pm/kcall/sleep.rs`).
  **Description:** The postconditions only relate `result` to its own `spec_pm_view` (`result.spec_classified_view() == spec_classify_pm_result(result.spec_pm_view())`), which is tautological and does not tie the return value to the *actual* `pm_result` returned by `process_manager_sleep`. This weakens equivalence: any classified result (Ok/Killed/GenericError) is allowed regardless of the PM outcome, so the model does not fully capture the 3-arm match semantics.
  **Suggested Fix:** Expose the PM result via a ghost/local spec variable or return a tuple so the postcondition can assert `result == classify_pm_result(pm_result)` and `result.spec_classified_view() == spec_sleep_result(..., pm_result.spec_pm_view())`. Alternatively, strengthen `process_manager_sleep` with a spec/ensures and add an explicit `ensures` linking the returned result to the specific PM outcome used.

### Medium
- **Location:** `sleep_model` / `sleep_end_to_end` preconditions (exec: `verus/split/kernel/pm/kcall/sleep.rs`).
  **Description:** The model uses `(u64, u32)` inputs but does not enforce the `seconds <= u32::MAX` bound that is implied by the original `(usize, usize)` on the x86-32 target (and even documented in comments). This allows values that cannot occur in the real API and can hide potential overflow behaviors if the target widens or the API changes.
  **Suggested Fix:** Add explicit `requires seconds <= u32::MAX as u64` (and, if desired for clarity, note that `nanoseconds` already fits `u32`) in both `sleep_model` and `sleep_end_to_end`.

- **Location:** `sleep_end_to_end` postconditions (exec: `verus/split/kernel/pm/kcall/sleep.rs`).
  **Description:** The end-to-end wrapper does not specify the InvalidArgument behavior on overflow (only the generic classification properties). If this function is treated as the public verified model, its spec is too weak and omits a key correctness property.
  **Suggested Fix:** Mirror the overflow-related ensures from `sleep_model` or add an explicit postcondition that ties overflow to `GenericError(InvalidArgument)`.

### Low
- **Location:** Error modeling in spec/exec (spec: `sleep.spec.rs`, exec: `sleep.rs`).
  **Description:** The model drops the error reason string from `Error::new(ErrorCode::InvalidArgument, reason)`, retaining only the numeric error code. This is a deliberate abstraction, but it weakens equivalence if the reason string is semantically relevant (e.g., for diagnostics or user-visible error propagation).
  **Suggested Fix:** If the reason string matters, extend `SleepResultModel::GenericError` to include a reason tag or add a spec note that the reason is intentionally abstracted.

## Positive Observations
- Good separation of spec/proof/exec with clear trust boundary documentation and consistent mapping from the original control flow.
- The spec correctly captures the 3-arm match behavior and distinguishes Ok/TimedOut/Killed/Generic errors.
- Key arithmetic well-formedness lemmas (duration normalization and alarm time) are present and sufficient for overflow reasoning.

## Summary
The verification is well-structured and covers the main result-classification logic, but the top-level exec spec is too weak because it does not tie the returned result to the actual PM outcome. Tightening the postconditions and aligning input bounds with the original API would improve equivalence and strengthen the model’s guarantees.
