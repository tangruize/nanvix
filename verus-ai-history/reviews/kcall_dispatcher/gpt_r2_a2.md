# Review: kcall_dispatcher (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `do_kcall_encoded` (exec: `dispatcher.rs`).
  - **Description:** The postcondition only states `exists r` such that `encoded == spec_encode_result(r)`, which is vacuously true for any `i64`. It does not tie the returned value to the verified dispatch result for the given arguments, so the ABI equivalence is still underspecified.
  - **Suggested Fix:** Strengthen the postcondition to relate `encoded` to the concrete result of `do_kcall(args)` (e.g., `encoded as int == spec_encode_result(do_kcall(args)@)`), or return the `DispatchResult` alongside the encoded value in a proof wrapper to make the link explicit.

### Low
- None.

## Positive Observations
- The CondSignal broadcast flag is now modeled as a boolean and the dispatcher passes `arg1 != 0`, matching the original semantics.
- OperationTimedOut uses 116 consistently across spec, proof, and exec comments.
- The new `encode_result` helper closes most of the previous ABI encoding gap by modeling `KcallResult::into::<i64>()` directly.

## Summary
The earlier issues around CondSignal and OperationTimedOut were fixed, and ABI encoding is now modeled, but `do_kcall_encoded` still lacks a meaningful postcondition tying the encoding to the actual dispatch result. Tightening that contract would make the verification materially stronger and complete the ABI equivalence story.
