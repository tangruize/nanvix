# Review: interrupted (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `InterruptedThread::thread_state_mut` (exec; `verus/split/kernel/pm/thread/interrupted.rs`)
  - **Description:** The function is still outside the `verus!` block with no spec or `external_body` contract; only a comment was added. It returns `&mut ThreadState`, allowing arbitrary mutation that can violate `wf()` and thread identity, leaving a soundness gap and a public API coverage hole.
  - **Suggested Fix:** Replace with verified setter APIs carrying pre/postconditions, or mark it `external_body` with explicit contracts that preserve `wf()`/`spec_id()` and restrict its use.

### Medium
- **Location:** `InterruptedThread::join_cond` (missing from verified model)
  - **Description:** The accessor remains omitted; documentation was added but there is still no boundary spec or stub. This leaves a public API unmodeled and join synchronization behavior unverified.
  - **Suggested Fix:** Add an `external_body`/boundary spec returning an opaque `Condvar` token with identity/equality properties, or model its use explicitly.

## Positive Observations
- `thread_state()` now guarantees full view equality (`result@ == self.state@`), fixing the previous weak accessor spec.
- The `ReadyThread` admission-time omission is now explicitly documented as out-of-scope.

## Summary
Two prior issues remain: the unverified `thread_state_mut` escape hatch and the missing `join_cond` boundary. Documentation improved, but verification is still incomplete and not fully sound for these APIs.
