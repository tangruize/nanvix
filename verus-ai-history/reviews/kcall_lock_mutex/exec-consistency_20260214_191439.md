# Review: kcall_lock_mutex Exec Consistency (claude-opus-4.6)

## Grade: A

## Issues Found

### Critical
- None.

### Minor
1. **`lock_mutex` wrapper postconditions are weaker than `lock_mutex_model`**: The added `lock_mutex` wrapper (line 737) only ensures two postconditions (invalid-timeout → timeout-error, infinite-timeout → no-TimedOut) whereas `lock_mutex_model` ensures six, including the key `spec_lock_mutex_result` linkage. Callers of `lock_mutex` cannot prove the result matches `spec_lock_mutex_result` without calling `lock_mutex_model` directly. This is acceptable since `lock_mutex_model` remains the primary verification entry point, but the wrapper's utility is limited to signature fidelity.

2. **`pid`/`tid` parameter types are `u32`, not model types**: The original uses `ProcessIdentifier` and `ThreadIdentifier`. Using `u32` is pragmatic for the verification model (these are just wrappers over numeric types), but strictly speaking the type mapping is documented only implicitly through the Trust Boundary T5 architecture note. A brief comment on the `lock_mutex` function noting this type simplification would improve traceability.

### Informational
- The `lock_mutex` wrapper has no postcondition linking its result to `spec_lock_mutex_result_with_context(pid, tid, ...)`, which would complete the formal loop between the pid/tid-inclusive spec and the pid/tid-inclusive exec. The `lemma_result_independent_of_pid_tid` proof exists in the spec but is not exercised by the exec wrapper.

## Review Criteria Assessment

### 1. Were all MISMATCH functions properly restored or equivalence documented?
**Yes.** The consistency report notes 0 mismatches to fix and 4 documented equivalences. All documented functions (`lock_mutex_model`, `parse_timeout`, `system_time_new`, `put_mutex_guard_model`) are present in the exec file with correct signatures and behavior matching the original source. The equivalence justifications are well-documented in the consistency report table.

### 2. Were MISSING functions added with proper verification?
**Yes.** The `lock_mutex` wrapper function was added (lines 737–760) as documented. It:
- Matches the original 5-parameter signature `(pid, tid, mutex_addr, timeout_s, timeout_ns)`.
- Delegates to `lock_mutex_model` which contains the full verification.
- Has `requires` clauses for ABI constraints.
- Has `ensures` clauses for key properties (timeout error, TimedOut impossibility).
- Verification passes: 25 verified, 0 errors (up from 24 → 25 as documented).

### 3. Are equivalence justifications sound?
**Yes.** Each justification is technically sound:
- **`lock_mutex_model`**: Correctly identified as the core pipeline model. The extraction pattern (separating trace-only params from logic) is standard across the kcall verification suite.
- **`parse_timeout`**: Correctly models the `if/else` + `SystemTime::new` pattern from lines 76–90 of the original. The postconditions link to `spec_parse_timeout` with all three branches covered.
- **`system_time_new`**: Correctly models `SystemTime::new` with the `nanoseconds < 1_000_000_000` invariant. Verified (not external_body).
- **`put_mutex_guard_model`**: Correctly uses external_body with guard token ghost state to model ownership transfer.

### 4. Does the exec code now faithfully represent the original source?
**Yes.** The exec code faithfully represents the original `lock_mutex` function:
- **Timeout parsing** (original lines 76–90): Modeled by `parse_timeout` with identical branch structure: both-MAX → None, valid-ns → Some, invalid-ns → Error(InvalidArgument).
- **get_mutex** (original line 92): Modeled by `get_mutex_model` with `.map_err(SleepError::Generic)` abstracted as `GetMutexError`.
- **lock** (original line 93): Modeled by `mutex_lock_model` with all four SleepError variants covered.
- **put_mutex_guard** (original line 94): Modeled by `put_mutex_guard_model` with `.map_err(SleepError::Generic)` abstracted as `PutGuardError`.
- **Control flow**: The `match` cascade in `lock_mutex_model` (lines 611–709) mirrors the original's `?` operator short-circuit behavior exactly.
- **Type casts**: `timeout_s as u64` and `timeout_ns as u32` from the original are correctly handled via the u32 parameter types and the `system_time_new(timeout_s as u64, timeout_ns)` call.

### 5. Does verification still pass?
**Yes.** Verification passes with 25 verified, 0 errors. No `assume`, `admit`, or unjustified `external_body` were added. The three pre-existing external bodies (`get_mutex_model`, `mutex_lock_model`, `put_mutex_guard_model`) are trust boundaries with documented postconditions.

## Summary

The exec consistency fix is well-executed. The primary change — adding a `lock_mutex` wrapper with the full 5-parameter signature — correctly restores signature fidelity with the original source while delegating to the fully-verified `lock_mutex_model`. The wrapper's postconditions are intentionally minimal (only the two most important properties), which is a reasonable design choice since `lock_mutex_model` remains available for callers needing the full spec linkage. All equivalence justifications are sound, the control flow faithfully mirrors the original, and verification passes cleanly at 25/0. No security, correctness, or soundness issues were found.
