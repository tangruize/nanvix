# Review: kcall_dispatcher (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** do_kcall_context / do_kcall (exec: verus/split/kernel/kcall/dispatcher.rs)
  - **Description:** The new guarantees only state that GetPid/GetTid successes return non-negative values, but they still do not specify that the returned value equals the ProcessManager pid/tid when retrieval succeeds. This leaves the key functional behavior under-specified at the context/ABI boundary.
  - **Suggested Fix:** Strengthen postconditions to relate successful pid/tid retrieval to the returned value (e.g., expose the pid/tid outcomes in a helper spec or add a wrapper lemma that captures equality).
- **Location:** convert_sleepable / remote_dispatch_verified (exec)
  - **Description:** Error mapping for sleepable failures is not captured in postconditions. The bodies call `handle_sleep_error`, but the contracts only ensure `!result.is_success`, so the precise error-code mapping (Generic preserves code, TimedOut maps to 110) is not exposed at the interface level.
  - **Suggested Fix:** Add postconditions like `!outcome.succeeded ==> result@ == spec_handle_sleep_error(...)` and the corresponding relation for remote dispatch sleep-error paths.

### Low
- **Location:** lemma_kcall_constants_consistency (proof/spec)
  - **Description:** Constant consistency is still a manual cross-reference and not mechanically linked to `src/libs/sys/src/sys/number.rs`, so drift could still go unnoticed.
  - **Suggested Fix:** Generate constants from the source or add a build-time check to compare enum values.
- **Location:** do_kcall (exec)
  - **Description:** The ABI entrypoint remains an `external_body`, so the link to `do_kcall_context` is still assumed.
  - **Suggested Fix:** If feasible, add a small verified wrapper that models ABI conversion and proves delegation to `do_kcall_context`.

## Positive Observations
- The remote dispatch path is now split into small external bodies with verified routing logic (`remote_dispatch_verified`), fixing the previous trust-boundary gap.
- Success-value constraints for ok()-returning calls and JoinThread are now enforced via external-body contracts and dispatch postconditions.
- Conditional non-negative guarantees for GetPid/GetTid are now surfaced at `do_kcall_context` and `do_kcall`.

## Summary
Major gaps from the previous review are resolved, especially the remote dispatch routing and success-value constraints. Remaining issues are mostly specification-strength gaps (GetPid/GetTid equality and sleep-error mapping exposure) plus residual trust assumptions for the ABI boundary and manual constant checks. Overall verification is significantly improved but not yet fully complete in specification precision.
