# Review: semaphore (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Concurrency/atomicity refinement still absent** (exec: `semaphore.rs`, docs: "Refinement Argument"). The update adds narrative text but no proof or coupling to `AtomicUsize`/`Condvar`; the verified model remains sequential and unrefined against the concurrent implementation.
- **Error-handling semantics still dropped for `down()`/`up()` and condvar failures** (exec/spec: `down_or_block`, `up`; spec/proof: error mapping). `try_down()` now maps to `ErrorCode::TryAgain`, but `Condvar::wait()`/`notify_first()` errors are still not modeled or propagated, leaving mismatch with the original `Result`-returning API.
- **Condvar interface remains a local assumption** (spec: `spec_condvar_wake_after_notify`). The limitation is now documented, but there is still no shared contract imported by both modules, so changes in the condvar implementation won't invalidate this proof.

### Medium
- **`down()` loop/liveness behavior still not modeled** (exec: `down_or_block`, `down_available`). The model splits instant success vs. would-block but does not refine the real sleep/retry loop or its error path, so it still diverges from the original API behavior.
- **Ghost waiters remain uncoupled from exec state** (spec: `View` returns `waiters: 0`; proof lemmas operate on constructed `SemaphoreView`s). The documentation acknowledges this, but no coupling invariant links the ghost queue length to the runtime condvar state.
- **Overflow precondition still stronger than runtime** (exec: `up` requires `value < usize::MAX`). The new lemma only shows the precondition is satisfiable under an external bound; there is still no system-level proof or wrapper enforcing the bound, so the spec remains stronger than the implementation.

## Positive Observations
- Added explicit trust boundaries, API mapping, and caller-context ghost preconditions to clarify verification scope.
- `try_down()` now has a formal mapping to `ErrorCode::TryAgain` via `spec_try_down_result_maps_ok`.
- Blocking protocol lemmas are well structured and preserve spec well-formedness.

## Summary
Documentation improved, but the core gaps (concurrency/atomicity refinement, condvar contract, and error semantics) remain unverified. The verification is still a sequential model with significant trust assumptions, so it is not yet complete or fully sound for the runtime semaphore.
