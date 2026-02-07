# Review: semaphore (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Concurrency/atomicity refinement still absent** (exec: `semaphore.rs`, docs: "Refinement Argument"). There is still no proof or coupling to `AtomicUsize`/`Condvar`; the verified model remains sequential and unrefined against the concurrent implementation.
- **Error-handling semantics still dropped for `down()`/`up()` and condvar failures** (exec/spec: `down_or_block`, `up`). `try_down()` maps to `ErrorCode::TryAgain`, but `Condvar::wait()`/`notify_first()` errors are still not modeled or propagated, leaving a mismatch with the original `Result`-returning API.
- **Condvar interface remains a local assumption** (spec: `spec_condvar_wake_after_notify`). The limitation is documented, but there is still no shared contract imported by both modules, so changes in condvar semantics won't invalidate this proof.

### Medium
- **`down()` loop/liveness behavior still not modeled** (exec: `down_or_block`, `down_available`). The model splits instant success vs. would-block but does not refine the real sleep/retry loop or its error path.
- **Ghost waiters remain uncoupled from exec state** (spec: `View` returns `waiters: 0`; proof lemmas operate on constructed `SemaphoreView`s). No coupling invariant links ghost waiters to runtime condvar state.
- **Overflow precondition still stronger than runtime** (exec: `up` requires `value < usize::MAX`). No system-level proof or wrapper enforces the bound; the spec remains stricter than the implementation.

## Positive Observations
- The verification scope and trust boundaries are explicit and consistent.
- `try_down()` has a formal mapping to `ErrorCode::TryAgain` via `spec_try_down_result_maps_ok`.
- Blocking protocol lemmas remain well-structured and preserve spec well-formedness.

## Summary
No substantive changes were found relative to the previous review; the same core gaps remain. The verification is still a sequential model with significant unverified assumptions, so it is not yet complete or fully sound for the runtime semaphore.
