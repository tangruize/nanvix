# Review: kcall_dispatcher (gemini-3-pro-preview)

## Grade: A-

## Issues Found

### High
- **ABI Signature Mismatch**: The verified `do_kcall` function has the signature `fn(DispatchArgs) -> DispatchResult`, while the original is `extern "C" fn(u32, u32, u32, u32, u32) -> i64`. The verified code cannot strictly replace the original without an unverified wrapper to marshal arguments and return values.
  - **Location**: `dispatcher.rs` (exec)
  - **Suggested Fix**: Add a verified `extern "C"` wrapper function that takes the primitive `u32` arguments, constructs `DispatchArgs`, calls `do_kcall`, and returns the `i64` from `encode_result`. This would verify the ABI boundary fully.

### Medium
- **External Body Reliance**: The correctness of the dispatcher relies entirely on the correctness of the `external_body` definitions for subsystem calls (e.g., `pm_get_pid`, `pm_join_thread`). If the actual implementations deviate from these specs (e.g., returning a negative success value), the dispatcher will be unsound.
  - **Location**: `dispatcher.rs` (exec - external bodies)
  - **Suggested Fix**: Ensure that the subsystems themselves are verified against these same specifications, or add runtime assertions in the glue code (if performance permits) to validate assumptions at the boundary.

### Low
- **Manual Constant Mirroring**: The spec constants (e.g., `KCALL_GET_PID`) are manually defined and checked against hardcoded values in `lemma_kcall_constants_consistency`. They are not automatically derived from the `sys::number::KcallNumber` enum.
  - **Location**: `dispatcher.spec.rs` (spec) and `dispatcher.proof.rs` (proof)
  - **Suggested Fix**: Add a CI step or build script to verify that the spec constants match the Rust enum values, or generate the spec constants from the enum definition.

## Positive Observations
- **Architecture**: The verification structure is excellent, with clear separation between specification (`dispatcher.spec.rs`), proof (`dispatcher.proof.rs`), and execution (`dispatcher.rs`).
- **Divergence Handling**: The modeling of the `panic!` path in `handle_sleep_error` using `handle_sleep_error_killed` and `diverge_after_exit` is sound and explicitly calls out the divergence behavior.
- **Coverage**: The verification covers all 32 defined kernel calls and correctly handles the "remote" wildcard case.
- **Trust Boundaries**: The documentation explicitly lists trust boundaries (T1-T6), making it clear what is assumed versus what is proven.

## Summary
The verification of `kcall_dispatcher` is high-quality and thorough regarding the internal routing logic. It successfully proves that the dispatcher correctly classifies and routes kernel calls. The primary limitation is the ABI gap: the verified function signature does not match the C ABI required by the kernel entry point, necessitating a small amount of unverified glue code. If this wrapper were added to the verification scope, the component would be nearly perfect.
