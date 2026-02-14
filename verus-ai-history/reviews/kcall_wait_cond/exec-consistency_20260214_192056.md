# Review: kcall_wait_cond Exec Consistency (claude-opus-4.6)

## Grade: A

## Verification Status

- **31 verified, 0 errors** (confirmed via `./verus-ai/scripts/verify.sh kcall_wait_cond`)
- Duration: 10s
- No `assume` or unjustified `external_body` added

## Review Criteria Assessment

### 1. Were all MISMATCH functions properly restored or equivalence documented?

**PASS.** The consistency report lists 0 mismatches fixed, which is correct — there were no pre-existing mismatches. All 5 "KEPT" functions have documented equivalence justifications in the fix report.

### 2. Were MISSING functions added with proper verification?

**PASS.** One missing function was added: `wait_cond`, a thin wrapper matching the original `pub unsafe fn wait_cond(...)` signature. The wrapper:
- Accepts all 6 original parameters (`pid`, `tid`, `cond_addr`, `mutex_addr`, `timeout_s`, `timeout_ns`) as concrete `u32` values.
- Forwards `pid` and `tid` as `Ghost` since they only affect trace logging and PM-internal ownership in the original, not pipeline control flow — this is correct per the original source (lines 81–88 use `pid`/`tid` only in `trace!()` and `error!()` macros).
- Delegates to `wait_cond_model` and returns `ret.0`, discarding the ghost state.
- Has appropriate `requires` (safety preconditions, currently-running) and `ensures` (mutex protocol on stored-result return).

The wrapper faithfully represents the original public API entry point.

### 3. Are equivalence justifications sound?

**PASS.** Each justification is reviewed:

| Function | Justification | Assessment |
|----------|---------------|------------|
| `cond_wait_model` | external_body for `Condvar::wait` — necessary trust boundary | **Sound.** `Condvar::wait` is opaque runtime behavior. Postcondition correctly constrains `TimedOut` to only occur when `has_alarm` is true. |
| `get_cond_and_wait_model` | Verified helper combining `get_cond` + `cond.wait` into stored-result computation | **Sound.** Directly models lines 116–129 of the original. When `get_cond` fails, `cond.wait` is never called and a don't-care ghost value is used for `cw_view`. The postcondition `ret.0.spec_view() == spec_stored_result(ret.1@, ret.2@)` ties exec to spec. |
| `mutex_lock_model` | external_body for `Mutex::lock(None)` | **Sound.** Postcondition `!matches!(result, LockOutcomeModel::TimedOut)` correctly proves `TimedOut` is impossible with `None` timeout. Postcondition also establishes `spec_mutex_reacquired` on success. |
| `parse_timeout_model` | Verified model of timeout parsing (lines 94–109) | **Sound.** Three branches match the original: (1) both `usize::MAX` → infinite, (2) valid nanoseconds → finite, (3) invalid → error. Uses `u32::MAX` which is correct for x86-32 (`usize` = 32-bit). |
| `wait_cond_model` | Core verified exec model with exec-spec equivalence postcondition | **Sound.** See detailed analysis below. |

### 4. Does the exec code now faithfully represent the original source?

**PASS with minor observations.**

#### Control Flow Mapping (Original → Model)

| Original (line) | Model Step | Faithful? |
|-----------------|-----------|-----------|
| L92: `ConditionAddress::from(cond_addr)` | Not modeled (type wrapper) | ✓ Correct to skip |
| L93: `MutexAddress::from(mutex_addr)` | Not modeled (type wrapper) | ✓ Correct to skip |
| L94–109: Timeout parsing | `parse_timeout_model` (Step 1) | ✓ |
| L112: `take_mutex_guard(pid, tid, mutex_addr).map_err(...)? ` | `take_mutex_guard_model` (Step 2) | ✓ Short-circuits on error |
| L116–129: `get_cond` + `cond.wait` stored result | `get_cond_and_wait_model` (Steps 3+4) | ✓ |
| L130: `put_cond(cond_addr).map_err(...)?` | `put_cond_model` (Step 5) | ✓ Runs unconditionally, `?` overrides stored result |
| L133: `get_mutex(mutex_addr).map_err(...)?` | `get_mutex_model` (Step 6) | ✓ |
| L134: `mutex.lock(None)?` | `mutex_lock_model` (Step 7) | ✓ |
| L135: `put_mutex_guard(mutex_addr, guard).map_err(...)?` | `put_mutex_guard_model` (Step 8) | ✓ |
| L137: `result` (return stored result) | Step 9: return `stored` | ✓ |

All `?` operator short-circuit semantics are correctly modeled: continuation errors override the stored result.

#### Semantic Details Verified

- **Stored result semantics**: When `get_cond` fails (line 119), `cond.wait` is NOT called — correctly modeled in `get_cond_and_wait_model` which only calls `cond_wait_model` in the `GetCondOutcomeModel::Ok` branch.
- **Continuation pipeline**: Steps 5–8 execute regardless of the stored result — correctly modeled with sequential match-and-return patterns.
- **pid/tid as Ghost**: The original uses `pid`/`tid` in `trace!()` (line 81), `error!()` (lines 102, 121), and `take_mutex_guard(pid, tid, ...)` (line 112). The model correctly passes them as `Ghost` to `take_mutex_guard_model` and omits logging. This is faithful because logging is a side effect that does not affect the return value.
- **Error wrapping**: The original wraps errors via `.map_err(SleepError::Generic)` — the model collapses this into `error_code: i32` which is equivalent since `SleepError::Generic(Error)` just wraps the error code.

### 5. Does verification still pass?

**PASS.** 31 verified, 0 errors. No regressions.

## Issues Found

### Critical

None.

### Minor

1. **Don't-care ghost values in error paths use inconsistent patterns.** In the `InvalidTimeoutError` and `TakeMutexGuardError` early-return paths (lines 676–688, 699–711), the ghost `lo` field uses `LockOutcomeView::LoGenericError { error_code: 0 }` to avoid triggering `spec_mutex_reacquired`. This is correct and documented, but the choice of `error_code: 0` technically violates `spec_is_valid_error_code(code > 0)`. This is harmless since the spec short-circuits before inspecting these values, but a comment noting `error_code: 0` is intentionally invalid would add clarity.

2. **`put_cond` is called unconditionally in both original and model, even when `get_cond` fails.** The fix report notes this as an "overapproximation" (trust boundary comment, lines 74–79 of the exec file). The model treats `get_cond` and `put_cond` outcomes as independent, which is sound but imprecise. This is properly documented.

3. **`LockTimedOut` dead branch.** The `LockTimedOut` arm (lines 783–798) is retained for exhaustive matching even though `mutex_lock_model` postcondition proves it unreachable. This is correct engineering for verification soundness, and `lemma_lock_timed_out_unreachable` in the proof file confirms the property globally.

## Summary

The exec consistency fix is well-executed. The added `wait_cond` wrapper correctly matches the original function's public signature and delegates to the fully-verified `wait_cond_model`. All 5 kept functions have sound equivalence justifications. The exec model faithfully represents every branch of the original's 7-step pipeline, including the critical stored-result + unconditional-continuation semantics. The `?` operator short-circuit behavior is accurately modeled at each step. Verification passes cleanly with 31 verified, 0 errors. The proof suite is comprehensive, covering error propagation, short-circuiting, result exhaustiveness, and the mutex release/reacquire protocol. No critical issues were found; the three minor observations are documentation-level and do not affect soundness.
