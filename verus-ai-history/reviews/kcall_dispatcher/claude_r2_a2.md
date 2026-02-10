# Review: kcall_dispatcher (claude-opus-4.6) — Round 2

## Grade: A-

## Verification Result

58 verified, 0 errors. All proofs pass cleanly.

## Previous Issues — Disposition

### Critical #1: Wrong ETIMEDOUT constant value (110 vs 116)
**Status: FIXED.** Verified in diff:
- `dispatcher.spec.rs:292`: `SPEC_ERROR_TIMED_OUT()` now returns `116`.
- `dispatcher.rs:491`: hardcoded value changed from `110i32` to `116i32`.
- `dispatcher.proof.rs:224,254`: comments updated to reference `116` / "ETIMEDOUT in Nanvix".
- The spec, exec code, and proof comments are all now consistent with `src/libs/sysapi/src/errno.rs:209` (`ETIMEDOUT = 116`).

### Medium #1: Tautological postcondition in `do_kcall_dispatch`
**Status: FIXED.** The previously tautological clause `!result.is_success ==> !result.is_success` has been replaced with a meaningful postcondition: `!result.is_success ==> (result.value >= i32::MIN as i64 && result.value <= i32::MAX as i64)`. This constrains error values on sleepable/fallible paths to the i32 range, which is correct given that all error codes originate from `KcallError(i32)`. The postcondition now provides real verification value — it ensures no error path produces an out-of-range value.

### Medium #2: CondSignal boolean argument not modeled
**Status: ADDRESSED (documentation).** The prover added documentation to `pm_signal_cond` (lines 637-645) explaining that the `arg1 != 0` boolean conversion is an internal detail of the subsystem call and the dispatcher only routes raw arguments. This is a reasonable architectural decision — the dispatcher's verified responsibility is routing, not argument interpretation. The trust boundary is now explicitly documented. **Accepted.**

### Medium #3: `handle_sleep_error_killed` uses `ensures false` as `external_body`
**Status: UNCHANGED.** No code changes were made. This was flagged as a necessary soundness assumption with "no code change needed" in the original review. The existing documentation (T4 trust boundary) remains adequate. **Accepted** — this is inherent to the verification model.

### Low #1: `spec_dispatch_result_constrained` trivially true for 5/6 categories
**Status: UNCHANGED.** No changes. This remains a minor observation. The per-call postconditions on `do_kcall_dispatch` and `do_kcall_context` are the primary correctness guarantees; this spec function adds only the `LocalTerminal ==> error` constraint at the top level. **Accepted** as a low-priority design choice.

### Low #2: Magic numbers in exec code
**Status: UNCHANGED.** No changes. The inline comments on each branch already cross-reference the kcall name and numeric value. **Accepted** as cosmetic.

### Low #3: 16 external bodies as large trust surface
**Status: UNCHANGED.** No changes. This is inherent to the dispatcher's role. **Accepted.**

## New Issues Introduced by Fixes

*(None identified.)* The changes are minimal and surgical — three constant value changes, one postcondition strengthening, and one documentation addition. No new code paths, no structural changes to proofs or specs.

## Remaining Issues

### Low

1. **`spec_dispatch_result_constrained` remains trivially true for 5/6 categories**
   - **Location:** `spec_dispatch_result_constrained` in `dispatcher.spec.rs:575-586`
   - **Description:** Carried forward from previous review. The function only constrains `LocalTerminal` meaningfully. The stronger guarantees are captured per-call in `do_kcall_dispatch`/`do_kcall_context`/`do_kcall`. This is a spec-strength observation, not a correctness issue.
   - **Impact:** Low. The verification is sound; the spec could be richer but isn't required to be.

2. **Magic numbers in exec-level classification and dispatch**
   - **Location:** `classify_kcall_number` and `do_kcall_dispatch` in `dispatcher.rs`
   - **Description:** Carried forward. Raw integer literals are used instead of named constants. Mitigated by inline comments.
   - **Impact:** Low. Readability/maintainability concern only.

3. **16 external bodies constitute the trust boundary surface**
   - **Location:** `dispatcher.rs:518-695` (16 `#[verifier::external_body]` functions)
   - **Description:** Carried forward. The cumulative trust surface is significant but inherent to the component.
   - **Impact:** Low. Each external body's postcondition is individually reasonable and documented.

## Positive Observations

- **Critical fix applied correctly.** The ETIMEDOUT value is now 116 throughout spec, exec, and proof, matching the Nanvix source at `src/libs/sysapi/src/errno.rs:209`.
- **Tautological postcondition replaced with meaningful constraint.** The new error-range postcondition on `do_kcall_dispatch` adds genuine verification value by proving that all error paths on sleepable/fallible calls produce i32-range values.
- **Responsive to review feedback.** All critical and medium issues were addressed — either fixed in code or documented with justified rationale.
- **No regressions.** Verification count remains at 58 verified, 0 errors. No existing proofs were weakened or removed.
- **Clean spec/proof/exec separation maintained.** Changes were properly placed in the appropriate files.
- **Comprehensive dispatch verification.** The full match-dispatch structure, pid/tid retrieval flow, sleep error handling, and remote scoreboard path are all verified with meaningful postconditions.
- **Strong per-call postconditions.** GetPid/GetTid return non-negative on success, ok()-returning calls produce value 0, JoinThread success ≥ 0, terminal calls always error — these are substantive guarantees.
- **Trust boundaries remain well-documented.** T1–T5 are clearly identified with assumptions stated.

## Summary

The prover addressed all actionable issues from the previous review. The critical ETIMEDOUT bug (110 → 116) is fixed in all three files. The tautological postcondition was replaced with a meaningful i32-range constraint on error paths. The CondSignal boolean abstraction gap was documented as an intentional design choice (dispatcher routes, doesn't interpret arguments), which is reasonable.

The remaining issues are all low-priority: a spec function that could be richer, magic numbers mitigated by comments, and the inherent trust surface of 16 external bodies. None of these affect soundness or correctness.

The verification is sound, complete for the dispatcher's scope, and provides meaningful guarantees about dispatch routing, error handling, and result well-formedness. The grade improves from B+ to A-.
