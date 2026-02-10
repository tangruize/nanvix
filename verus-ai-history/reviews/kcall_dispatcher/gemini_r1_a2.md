# Review: kcall_dispatcher (gemini-3-pro-preview)

## Grade: A

## Issues Found

### Low
- **ABI Representation Gap:** The top-level `do_kcall` function in the verified model uses `DispatchArgs` and `DispatchResult` structs, whereas the actual kernel entry point uses the C ABI (`extern "C"` with `u32`/`i64` primitives). While the logic is now fully verified, there remains a small translation gap between the C ABI arguments and the verified model types that must be trusted.
  - **Location:** `verus/split/kernel/kcall/dispatcher.rs` (function `do_kcall`)
  - **Suggested Fix:** Ensure the C ABI shim that calls into this verified logic (if it were to be hooked up for real) performs the struct construction/destructuring exactly as assumed. This is documented in the code comments and is an acceptable limitation of the current split model.

## Positive Observations
- **Entry Point Now Verified:** The previous "High" severity issue has been resolved. `do_kcall` is no longer `external_body` but is now a verified function that delegates to `do_kcall_context`. This closes the verification gap for the top-level dispatch logic.
- **Explicit Divergence Modeling:** The `handle_sleep_error_killed` function now uses `ensures false` to explicitly model divergence, allowing the verifier to correctly reason about the unreachable code following a process kill.
- **Strong Postconditions:** The `do_kcall_dispatch` function maintains strong postconditions for `GetPid`/`GetTid` (guaranteeing success and correct values), justifying the weaker classification-level spec which must account for potential failures in the surrounding context.
- **Verification Pass:** The module passes verification with 58 verified blocks, covering the entire dispatch stack.

## Summary
The `kcall_dispatcher` verification is now complete and sound. The prover correctly addressed the critical issue of the unverified entry point by implementing `do_kcall` as a verified wrapper. The handling of the divergent `Killed` path using `ensures false` is the correct Verus idiom for this scenario. The remaining limitation regarding the C ABI mismatch is well-documented and acceptable for this stage of verification. The logic is rigorous, the separation of concerns is clean, and the proofs are comprehensive.
