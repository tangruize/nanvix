# Review: kcall_dispatcher (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- **Location:** `do_kcall` ABI boundary (exec: `dispatcher.rs`, T5).
  - **Description:** The verified entry point returns `DispatchResult` with an explicit `is_success` flag, but the original ABI returns an `i64` from `KcallResult::into`. There is no proof that the encoded `i64` is equivalent to the verified result, especially for success values in the i32 range that are indistinguishable from error encodings.
  - **Suggested Fix:** Add a verified wrapper (or postcondition) that models the `KcallResult -> i64` encoding and proves `do_kcall`’s return value matches `spec_encode_result`. Alternatively, verify a `do_kcall_abi` function that takes raw u32 args and returns i64 with the correct encoding.

### Medium
- **Location:** CondSignal dispatch (exec: `do_kcall_dispatch`, external body `pm_signal_cond`).
  - **Description:** The original dispatcher converts `arg1` to a boolean (`arg1 != 0`) before calling `pm::signal_cond`, but the verified model forwards raw `arg1: u32` to `pm_signal_cond`. This weakens equivalence and allows behaviors not possible in the original (non-0 values other than 1 are not normalized).
  - **Suggested Fix:** Model the boolean conversion explicitly (e.g., pass `if arg1 != 0 { 1 } else { 0 }` or change the external body to take a `bool`/predicate on `arg1`). Add a proof that the conversion matches the original semantics.

### Low
- **Location:** Documentation comments (exec: `dispatcher.rs`, lines ~44 and ~463).
  - **Description:** Comments state OperationTimedOut is error code 110, but the verified implementation and spec use 116 (Nanvix ETIMEDOUT). This is misleading and suggests potential spec drift.
  - **Suggested Fix:** Update the comments to 116 to match Nanvix’s `ErrorCode::OperationTimedOut`.

## Positive Observations
- All original functions (`do_kcall`, `handle_sleep_error`) are covered with verified counterparts, and the match structure for dispatch routing is fully verified.
- Sleep error handling cleanly separates the divergent Killed path from non-divergent errors, with well-formedness proofs for all constructors.
- Spec/proof/exec separation is clear, and trust boundaries (ProcessManager/ScoreBoard) are explicitly documented.

## Summary
The verification captures the core routing and error-handling behavior, but the ABI encoding gap and the missing boolean normalization for CondSignal weaken equivalence guarantees. Tightening these two areas would bring the model closer to a fully faithful dispatcher proof.
