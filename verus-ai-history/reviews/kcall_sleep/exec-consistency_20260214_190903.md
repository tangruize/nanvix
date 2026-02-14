# Review: kcall_sleep Exec Consistency (claude-opus-4.6)

## Grade: A

## Verification Status
- **17 verified, 0 errors** — all proofs pass.
- Command: `./verus-ai/scripts/verify.sh kcall_sleep`

## Review Criteria Assessment

### 1. Were all MISMATCH functions properly restored or equivalence documented?

**Yes.** The consistency report lists 0 mismatches fixed, which is consistent with the code — there were no structural mismatches between the original and the model. All 9 "documented equivalence" entries are justified.

### 2. Were MISSING functions added with proper verification?

**Yes.** One missing function was added: the `sleep` function as an `external_body` (lines 651–663 of sleep.rs). This is the original entry point `pub unsafe fn sleep(seconds: usize, nanoseconds: usize)` modeled as `pub fn sleep(seconds: u64, nanoseconds: u32)`. The `external_body` approach is justified because the original depends on real kernel types (`SystemTime`, `Duration`, `ProcessManager`, `SleepError`) unavailable in Verus. However, see Minor Issue #1 below regarding its postconditions.

### 3. Are equivalence justifications sound?

**Yes.** The key justifications are:

- **`SystemTimeModel` / `DurationModel`**: Required model types for private-field kernel types. Sound.
- **`clock_now()`**: External body for `clock::now()`. Postcondition captures well-formedness guarantee from the clock module. Sound.
- **`checked_add_duration()`**: External body for `SystemTime::checked_add_duration()`. Postcondition ties result to `spec_checked_add_succeeds` and `spec_compute_alarm`. Sound — the spec correctly models nanosecond carry behavior.
- **`process_manager_sleep()`**: External body for `ProcessManager::sleep(Some(alarm))`. The postcondition only asserts the result is one of the defined variants (which is trivially true for an enum). This is intentionally minimal — timing/liveness properties are explicitly out of scope and documented in detail (lines 326–342). Sound design decision.
- **`duration_new()`**: Verified model of `Duration::new()`. Correctly implements nanosecond normalization (carry into seconds, remainder). The proof invokes `lemma_duration_new_wf` and the postcondition ties the result to `spec_duration_new`. Sound.
- **`sleep_model()`**: Core verified model. Faithfully mirrors the original's 3-arm match (lines 62–66). The `classify_pm_result` helper correctly maps `Ok → Ok`, `TimedOut → Ok`, `Killed → Killed`, `GenericError → GenericError`. Sound.
- **`sleep_end_to_end()`**: End-to-end composition. Cleanly separates clock acquisition from sleep logic. Sound.

### 4. Does the exec code faithfully represent the original source?

**Yes, with one acceptable abstraction.** Detailed comparison:

| Original (sleep.rs) | Exec Model (sleep.rs) | Faithful? |
|---|---|---|
| `clock::now()` (line 50) | `clock_now()` external_body | ✓ |
| `Duration::new(seconds as u64, nanoseconds as u32)` (line 53) | `duration_new(seconds, nanoseconds)` with verified normalization | ✓ |
| `now.checked_add_duration(&timeout)` (line 54) | `checked_add_duration(now, &timeout)` external_body | ✓ |
| `Some(wakeup_time) => wakeup_time` (line 55) | `Some(alarm) =>` path in `sleep_model` (line 531) | ✓ |
| `None => Err(SleepError::Generic(Error::new(ErrorCode::InvalidArgument, "...")))` (lines 56–59) | `None => GenericError { error_code: 22i32 }` (line 550) | ✓ (error code verified by `lemma_error_code_matches`) |
| `ProcessManager::sleep(Some(alarm))` (line 62) | `process_manager_sleep(&alarm)` (line 533) | ✓ |
| `Ok(()) => Ok(())` (line 63) | `SleepResultModel::Ok => SleepResultModel::Ok` (line 452) | ✓ |
| `Err(SleepError::Interrupted(InterruptReason::TimedOut)) => Ok(())` (line 64) | `SleepResultModel::TimedOut => SleepResultModel::Ok` (line 453) | ✓ |
| `Err(error) => Err(error)` (line 65) | `Killed => Killed`, `GenericError => GenericError` (lines 454–455) | ✓ |

The error reason string `"invalid sleep time"` is intentionally abstracted away with documented justification (lines 96–101 of sleep.spec.rs and the module doc). This is correct — the string is diagnostic only and callers match on the error code.

**Type signature difference**: The original takes `(usize, usize)` while the model takes `(u64, u32)`. This is documented in Trust Boundary T5 (lines 89–93) and validated by `lemma_usize_cast_safety` which proves the cast is safe on x86-32. The precondition `seconds <= USIZE_MAX_X86_32()` enforces the domain restriction.

### 5. Does verification still pass?

**Yes.** 17 verified, 0 errors.

## Issues Found

### Critical
- None.

### Minor

1. **`sleep` external_body postconditions are weaker than `sleep_end_to_end`'s**: The `sleep` function (lines 651–663) only guarantees `!matches!(result, SleepResultModel::TimedOut)` — it doesn't expose the overflow/success path classification that `sleep_end_to_end` proves. This means callers of `sleep` get weaker guarantees than callers of `sleep_end_to_end`. This is acceptable since `sleep` is primarily a signature placeholder and `sleep_model`/`sleep_end_to_end` are the verification workhorses, but strengthening `sleep`'s postconditions to match `sleep_end_to_end` would close the gap. **Impact: Low** — the verified model (`sleep_model`) carries the full proof obligations.

2. **`SleepResultModel` has 4 variants but `SleepError` has 2 (with nested enums)**: The model flattens `SleepError::Interrupted(InterruptReason::TimedOut)` and `SleepError::Interrupted(InterruptReason::Killed)` into top-level `TimedOut` and `Killed` variants. This is a sound simplification — the flattened representation captures all distinct behaviors — but worth noting as a structural divergence from the original type hierarchy. **Impact: None** — the flattening is semantically equivalent.

3. **`process_manager_sleep` postcondition is trivially true**: The `ensures` clause `matches!(result, Ok | TimedOut | Killed | GenericError { .. })` is always true for the `SleepResultModel` enum since those are all its variants. While the documentation explains this is intentional (timing/liveness out of scope), it means the external body contributes no verifiable constraint. **Impact: None** — this is by design and well-documented.

## Proof Coverage Assessment

The proof suite is comprehensive:

| Lemma | Property | Status |
|---|---|---|
| `lemma_duration_new_wf` | Duration normalization well-formedness | ✓ |
| `lemma_alarm_wf` | Alarm time well-formedness | ✓ |
| `lemma_overflow_returns_invalid_argument` | Overflow → InvalidArgument | ✓ |
| `lemma_timed_out_is_success` | TimedOut → Success | ✓ |
| `lemma_killed_is_error` | Killed → Error (not success) | ✓ |
| `lemma_pm_success_is_success` | PmOk → Success | ✓ |
| `lemma_pm_error_propagates` | GenericError passes through | ✓ |
| `lemma_sleep_result_exhaustive` | Result trichotomy + mutual exclusion | ✓ |
| `lemma_zero_duration_always_valid` | Zero sleep always valid | ✓ |
| `lemma_success_only_from_ok_or_timed_out` | Success ↔ {PmOk, PmTimedOut} | ✓ |
| `lemma_error_code_matches` | Spec constant = ErrorCode discriminant | ✓ |
| `lemma_usize_cast_safety` | x86-32 usize cast is safe | ✓ |

## Summary

The exec consistency fix is thorough and correct. The verified model faithfully represents the original `sleep` kernel call's control flow, with all external dependencies properly modeled as `external_body` functions with appropriate trust boundary documentation. The 3-arm match classification — the core logic of this kcall — is fully verified through `classify_pm_result` and `sleep_model`. The proof suite covers all paths (overflow, success via Ok, success via TimedOut, error via Killed, error via GenericError) with mutual exclusion and exhaustiveness guarantees. The only notable gap is the weak postconditions on the `sleep` external_body itself, which has low impact since `sleep_model` and `sleep_end_to_end` carry the full proof obligations.
