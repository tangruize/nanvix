# Review: interrupted (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `InterruptedThread::thread_state_mut` (exec; `verus/split/kernel/pm/thread/interrupted.rs`)
  - **Description:** The function is now marked `#[verifier::external]`, but it still has no machine-checked spec. It returns `&mut ThreadState`, so callers can violate `wf()` and `spec_id()` without any enforced contract. This remains a soundness and coverage gap; the new comments only document intent.
  - **Suggested Fix:** Provide a verified API (setter methods with pre/postconditions) or refactor usage so the mutable access is within verus-checked code. If `external` is unavoidable, add a formal spec via a verus-visible wrapper that enforces identity/wf preservation.

### Medium
- **Location:** `InterruptedThread::join_cond` (missing from verified model)
  - **Description:** The omission is now justified by the lack of a `Condvar` type in the verification model (confirmed in `state.rs` header). This is still a public API coverage hole: join synchronization behavior is unmodeled for interrupted threads.
  - **Suggested Fix:** Introduce an opaque spec-level `Condvar` token and an `external_body` accessor, or model join semantics in a higher-level module that includes `Condvar` as an abstract type.

## Positive Observations
- The external annotation on `thread_state_mut` makes the trust boundary explicit and avoids accidental proof assumptions.
- The documentation now consistently explains why `join_cond()` is omitted and ties it to the broader `ThreadState` model.

## Summary
The fixes are mostly documentation and boundary annotations; the two core issues remain. Verification is still incomplete for `thread_state_mut` and `join_cond`, so the module is not fully sound or complete.
