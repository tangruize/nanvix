# Review: kcall_dispatcher (gemini-3-pro-preview)

## Grade: A-

## Issues Found

### High
- **Entry Point Verification Gap:** The top-level function `do_kcall` is marked as `#[verifier::external_body]` in `verus/split/kernel/kcall/dispatcher.rs`. The verification proves properties of the helper functions `do_kcall_context` and `do_kcall_dispatch`, but strictly speaking, the actual entry point is trusted, not verified. There is no mechanical guarantee that the original C-ABI `do_kcall` implementation (in `src/kernel/src/kcall/dispatcher.rs`) matches the verified model `do_kcall_context`.
  - **Location:** `verus/split/kernel/kcall/dispatcher.rs` (function `do_kcall`)
  - **Suggested Fix:** If possible, implement `do_kcall` in the verified file to call `do_kcall_context` directly, rather than marking it `external_body`. If the C ABI signature prevents this in Verus, document the manual verification step required to ensure the original source matches the model.

### Medium
- **Unverified Panic on Killed Path:** The original `handle_sleep_error` panics if `ProcessManager::exit()` fails during a `Killed` interruption. The verified model handles this via `handle_sleep_error_killed`, which is `external_body` and effectively assumes divergence. While this models the behavior, the safety of the panic itself (and the unreachable code after it) is not mechanically verified.
  - **Location:** `verus/split/kernel/kcall/dispatcher.rs` (function `handle_sleep_error_killed`)
  - **Suggested Fix:** Model the divergence explicitly if Verus supports it, or ensure the `external_body` contract explicitly states it does not return.

### Low
- **Weakened Specification for LocalImmediate:** The specification `spec_dispatch_result_constrained` for `DispatchCategory::LocalImmediate` (GetPid/GetTid) only requires that *if* it fails, it's fine. It does not enforce success. The stronger property (success when PM is accessible) is only captured in the `do_kcall_dispatch` postconditions.
  - **Location:** `verus/split/kernel/kcall/dispatcher.spec.rs` (function `spec_dispatch_result_constrained`)
  - **Suggested Fix:** Consider strengthening the spec constraint or documenting why the weaker constraint is sufficient for the classification level.

## Positive Observations
- **Comprehensive Coverage:** The verification covers all 32 defined kernel call numbers and ensures they are correctly classified and routed.
- **Clean Split:** The separation between executable model (`dispatcher.rs`), specifications (`dispatcher.spec.rs`), and proofs (`dispatcher.proof.rs`) is excellent and makes the verification logic easy to follow.
- **Detailed Modeling:** The modeling of `DispatchResult`, `SleepError`, and the various outcome types (`SleepableOutcome`, `FallibleOutcome`) provides a high-fidelity representation of the underlying kernel logic while abstracting away ABI details.
- **Proof Structure:** The use of lemmas to prove properties about the dispatch table (e.g., `lemma_local_remote_partition`, `lemma_defined_kcalls_classified`) provides high confidence in the correctness of the routing logic.

## Summary
The `kcall_dispatcher` verification is high quality, demonstrating that the complex dispatch logic is correct with respect to the specification. The dispatch table is exhaustively covered, and the handling of different return types (sleepable, fallible, direct) is rigorously modeled. The primary limitation is the use of `external_body` for the top-level `do_kcall` function, which creates a trust gap between the verified model and the actual C-ABI entry point. Despite this, the core routing logic `do_kcall_dispatch` is fully verified, providing strong assurance of correctness.
