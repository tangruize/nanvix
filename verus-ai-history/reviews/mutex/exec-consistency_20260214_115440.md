# Review: mutex Exec Consistency (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

- None.

### Minor

1. **`reference_count()` is a constant stub, not a behavioral model.**
   The verified `reference_count()` always returns `1usize`. The original returns `Arc::strong_count()`, which can be >1 when the `Mutex` is cloned (it derives `Clone`). The fix report documents this as "Without `Arc`, there is exactly one owner" which is sound within the sequential model, but the `Clone` derive on the original `Mutex` means multi-owner scenarios are part of the original API surface. The constant `1` does not model the reference-counting protocol—it models a single snapshot. Acceptable as a stub but should not be treated as verifying reference count behavior.

2. **`try_lock()` return type diverges from original.**
   The original returns `Result<MutexGuard, ()>` (Rust idiom for fallible acquisition). The verified version returns `(bool, Tracked<Option<MutexToken>>)`. This is a reasonable Verus adaptation (Verus lacks `Result` ergonomics and needs tracked tokens), but the fix report's equivalence table does not explicitly document this return-type divergence—it focuses on `&self` vs `&mut self` and atomics.

3. **`lock()` signature divergence under-documented.**
   The original `lock()` takes `timeout: Option<SystemTime>` and returns `Result<MutexGuard, SleepError>`. The verified version takes no timeout and returns `Tracked<MutexToken>` (infallible). The fix report mentions "timeout not modeled" but does not note that the error path (`SleepError`) is also removed, meaning the verified model cannot represent lock failures due to timeout or process-level errors.

4. **`unlock_unchecked()` error path dropped.**
   The original returns `Result<(), Error>` because `notify_first()` can fail. The verified version returns `()`. The `Drop` impl in the original logs a warning on failure (`warn!(...)`). This error-handling path is silently absent in the verified model. The fix report says "external dependency not modeled" which is correct, but the error path elimination should be explicitly called out as a trust boundary.

5. **`Mutex` struct fields are `pub`.**
   The fix report notes this is "required by Verus for `pub open spec fn` access" and the module documentation includes Trust Assumption T3 about token construction uniqueness. This is acceptable given Verus tooling constraints, but diverges from the Nanvix coding standard requiring private fields with getters.

### Observations (Non-Issues)

- **`fmt::Debug` and `Drop` not modeled:** Correctly documented as Verus limitations. `Drop` is replaced by explicit `unlock()` with token consumption—this is the standard Verus pattern for RAII modeling.
- **`MutexInner` flattened into `Mutex`:** Sound simplification. The `Arc<MutexInner>` indirection exists only for shared ownership, which is out of scope.
- **`MutexGuard` replaced by `MutexToken`:** Correct Verus idiom. The tracked ghost struct provides the same proof obligation (must be consumed to release the lock).
- **`is_locked()` extra function:** Useful verification helper, does not pollute the exec model.
- **No `assume`, `admit`, `external_body`, or `trusted`:** Clean verification with no escape hatches. All 29 items verified.

## Verification Results

- **Status:** PASS (29 verified, 0 errors)
- **No escape hatches:** No `assume`, `admit`, `external_body`, or `trusted` used.
- **Verification time:** ~9 seconds.

## Assessment of Fix Report Claims

| Claim | Verdict |
|-------|---------|
| "0 mismatches fixed (all 3 documented as Verus limitations)" | ✅ Correct. `fmt`, `Drop`, and atomics are genuine Verus limitations with sound workarounds. |
| "2 missing functions added (`reference_count`, `unlock_unchecked`)" | ✅ Correct. Both present with appropriate modeling. |
| "5 documented equivalences" | ✅ All five equivalences (`new`, `try_lock`, `lock`, `Mutex` struct, `MutexGuard`) are documented with justifications. |
| "+2 verified items (27→29)" | ✅ Confirmed. Verification passes with 29 items. |
| "No `assume`, `admit`, or `external_body` added" | ✅ Confirmed by code inspection. |

## Summary

The exec consistency fixes are thorough and well-documented. All original functions are either faithfully modeled in the Verus exec code or explicitly documented as out-of-scope with sound justification. The two added functions (`reference_count`, `unlock_unchecked`) correctly fill gaps identified in the consistency analysis. The module documentation is exceptionally detailed, with clear API mapping tables, trust assumptions, verification scope boundaries, and a refinement argument connecting the sequential model to the concurrent implementation.

The grade is A- rather than A because: (1) the `try_lock()` and `lock()` return-type divergences and the `lock()` error-path removal are under-documented in the fix report's equivalence table, and (2) the `unlock_unchecked()` error path elimination deserves explicit mention as a trust boundary rather than being folded into the general "Condvar not modeled" note. These are documentation completeness issues, not correctness issues—the verified code itself is sound.
