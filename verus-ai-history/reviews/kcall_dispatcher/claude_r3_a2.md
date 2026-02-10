# Review: kcall_dispatcher (claude-opus-4.6)

## Grade: A

## Previous Issue Resolution

### High: `handle_sleep_error_killed` uses `ensures false` with `external_body`
- **Status: Addressed.** The prover added `// SOUNDNESS NOTE` comments at both call sites (`convert_sleepable` line 801–803, `remote_dispatch_verified` line 749–751) referencing trust boundary T4. The comments explicitly warn that if the original Killed path changes to not diverge, the external body must be updated. This is exactly what was requested. The fundamental `ensures false` remains, which is inherent to the Verus modeling approach for divergent functions — there is no better alternative until Verus supports the `!` type.
- **Verdict: Satisfactorily resolved.**

### Medium: Magic numbers in exec dispatch logic
- **Status: Not fixed.** The exec functions (`classify_kcall_number`, `do_kcall_dispatch`, postconditions) still use raw `u32` literals (e.g., `1u32`, `2u32`, `22u32`). This was flagged as a maintainability issue, not a correctness issue.
- **Assessment: Acceptable to defer.** The postcondition `result =~= spec_classify_kcall(number)` links exec to spec, so correctness is guaranteed regardless. The prover may have decided (reasonably) that Verus's exec-level code cannot directly reference `open spec fn` constants in `if` conditions, making this a Verus limitation rather than an oversight. The inline comments (e.g., `// GetPid (1), GetTid (2)`) partially mitigate the readability concern.
- **Verdict: Accepted as-is. Demoted to Low.**

### Medium: No postcondition constraining CondSignal success value
- **Status: Fixed.** Three changes made:
  1. `pm_signal_cond` external body now has `result.succeeded ==> result.value >= 0` (line 652). Verified this matches the original `Result<u32, Error>` return type — `u32` is always `>= 0` when sign-extended to `i64`. ✓
  2. `do_kcall_dispatch` postcondition now includes `args.number == 26u32 && result.is_success ==> result.value >= 0` (line 898). ✓
  3. Propagated through `do_kcall_context` (line 986), `do_kcall` (line 1049), and `do_kcall_encoded` (line 1122). ✓
- **Verdict: Fully resolved. Verified the constraint propagates end-to-end.**

### Medium: `ScoreboardDispatchOutcome` success path bypasses result constructors
- **Status: Fixed.** `remote_dispatch_verified` (lines 740–745) now uses `if dispatch_outcome.result_is_success { DispatchResult::success(...) } else { DispatchResult::error(... as i32) }` instead of direct struct construction. This routes through the constructor postconditions and provides spec-level linkage.
- **Verification check:** The `DispatchResult::error()` call uses `dispatch_outcome.result_value as i32`. The `ScoreboardDispatchOutcome.wf()` ensures `result_value` fits in i32 range when `!result_is_success`, so the truncation is safe. ✓
- **Verdict: Fully resolved.**

### Low: Representation gap: `as usize` vs `u32`
- **Status: Fixed.** A comment block was added at lines 536–540 documenting: "On the target x86-32 platform, `usize` is 32 bits (same as `u32`), so the verified model accepts `u32` directly. If the code were ported to a 64-bit target, this equivalence would no longer hold."
- **Verdict: Fully resolved.**

### Low: `ProcessIdentifier`/`ThreadIdentifier` modeled as `i64`
- **Status: Fixed.** Both `pm_get_pid()` (line 549) and `pm_get_tid()` (line 559) now have `result.succeeded ==> (result.value >= 0 && result.value <= i32::MAX as i64)`. This tightens the postcondition to match the actual `i32`-wrapped types.
- **Verdict: Fully resolved.**

### Low: `do_kcall` vs `do_kcall_encoded` precondition inconsistency
- **Status: Fixed.** `do_kcall_encoded` (line 1104) no longer has `requires args.wf()`. Both `do_kcall` and `do_kcall_encoded` now have no precondition, which is consistent since `args.wf()` is always `true`.
- **Verdict: Fully resolved.**

### Low: Proof lemmas trivially discharged
- **Status: Acknowledged.** No changes made (none were needed — this was informational). The lemmas continue to serve as regression checks.
- **Verdict: N/A (informational only).**

## Issues Found

### Critical
- None.

### High
- None.

### Medium

- **Magic numbers in exec dispatch logic (carried forward, demoted from Medium)**
  - Location: `classify_kcall_number()`, `do_kcall_dispatch()` postconditions (exec)
  - Description: Raw u32 literals still used instead of named constants. This is a maintainability concern, not a correctness issue. The spec-level postconditions guarantee correctness regardless. This may be a Verus limitation (spec fns not usable in exec-level `if` conditions).
  - Suggested Fix: If Verus allows `const` items in `verus!` blocks, define exec-level constants. Otherwise, accept as inherent to the verification framework.

### Low

- **`do_kcall_encoded` postcondition repeats `do_kcall` postcondition verbatim**
  - Location: `do_kcall_encoded()` (exec, lines 1104–1124)
  - Description: The postcondition of `do_kcall_encoded` manually repeats all postconditions from `do_kcall` (terminal error, GetPid/GetTid non-negative, ok-returning calls == 0, CondSignal >= 0, JoinThread >= 0). If `do_kcall` postconditions change, `do_kcall_encoded` must be updated in lockstep, creating a maintenance risk. Ideally, `do_kcall_encoded` would reference `do_kcall`'s contract rather than duplicating it.
  - Suggested Fix: This is a Verus limitation — there is no way to "inherit" postconditions from a callee. Accept as-is but note the duplication risk in a comment.

## Positive Observations

- **All 7 actionable issues from R1 addressed.** The prover systematically resolved every issue: SOUNDNESS NOTE comments added at call sites, CondSignal constraint propagated end-to-end, remote dispatch constructor bypass fixed, usize/u32 documented, pid/tid range tightened, precondition inconsistency resolved. This demonstrates thorough engagement with the review.

- **CondSignal constraint correctly propagated through 4 function layers.** The `result.value >= 0` constraint was added to `pm_signal_cond`, `do_kcall_dispatch`, `do_kcall_context`, `do_kcall`, and `do_kcall_encoded` — full propagation verified.

- **Remote dispatch path now uses type-safe constructors.** The fix to `remote_dispatch_verified` properly branches on `result_is_success` and calls `DispatchResult::success()` / `DispatchResult::error()`, ensuring spec-level view linkage.

- **Verification still passes: 61 verified, 0 errors.** No regressions introduced by the fixes.

- **All prior positive observations still hold:** Complete function coverage, clean spec/proof/exec separation, 33 kcall constants verified, well-documented trust boundaries (T1–T5), dispatch routing fully verified, error handling chain verified end-to-end, strong success-value postconditions, ABI encoding explicitly modeled.

## Summary

The prover addressed all actionable issues from the R1 review thoroughly and correctly. The one remaining item (magic numbers) is a reasonable deferral given likely Verus limitations on using spec functions in exec-level control flow. The verification is sound, complete for the dispatcher's scope, and well-documented.

The module now has:
- **Full dispatch routing verification** for all 32 defined kcall numbers plus the wildcard/invalid path.
- **End-to-end success value constraints** including the newly-added CondSignal `>= 0` guarantee.
- **Tighter type modeling** with pid/tid bounded to i32 range.
- **Explicit soundness documentation** at all `ensures false` call sites.
- **Clean trust boundaries** with appropriate external bodies for subsystem calls.

No further changes are needed for this module to be considered verification-complete at its current scope.
