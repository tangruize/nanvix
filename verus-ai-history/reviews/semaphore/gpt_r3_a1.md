# Review: semaphore (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Equivalence gap for concurrency/atomicity** (exec: `Semaphore::down`, `try_down`, `up` in original; exec/spec/proof: `down_or_block`, `try_down`, `up` in `semaphore.rs`/`semaphore.spec.rs`). The verified model is sequential (`&mut self`, plain `usize`) and relies on an informal linearizability argument in comments; no proof ties the atomic `fetch_update`/`fetch_add` behavior to the spec. This leaves the core concurrent correctness of a kernel semaphore unverified. **Suggested fix:** add a refinement/linearizability proof (or a concurrent spec) that relates atomic operations to the sequential model, or explicitly model atomic steps with ghost linearization points.
- **Error-handling semantics are dropped** (exec/spec: `down_or_block`, `try_down`, `up`). The original `down()` and `up()` return `Result` to reflect `Condvar::wait()`/`notify_first()` failures and `try_down()` returns `ErrorCode::TryAgain`. The verified model assumes `notify_first()` always succeeds (T5) and does not represent `SleepError` from `wait()` (T6). This weakens equivalence and omits a safety/liveness risk (e.g., value incremented but no wake). **Suggested fix:** extend the spec to include error outcomes, prove postconditions for error paths, and relate `bool` to `Result` more precisely (including error propagation).
- **Condvar interface is a local assumption, not a shared contract** (spec: `spec_condvar_wake_after_notify` in `semaphore.spec.rs`). The assumption is not imported from the verified condvar module, so changes in condvar specs/exec do not invalidate this proof. **Suggested fix:** define a shared interface spec (trait/axiom module) that both semaphore and condvar import, and prove condvar conforms to it.

### Medium
- **Coverage divergence for `down()` behavior** (exec: `down_or_block`, `down_available`). The verified interface does not include an exact analogue of the looping `down()` (sleep/retry) with `Result<(), SleepError>`; instead it splits into instant-success and “would block” outcomes. This covers state-machine logic but not the complete API behavior. **Suggested fix:** add a verified wrapper that models the loop with explicit sleep/wake/error outcomes or provide a refinement lemma mapping the original `down()` behavior to the two-step model.
- **Waiter ghost state not tied to exec state** (spec/proof: `SemaphoreView.waiters`, `View` returns `waiters: 0`). Proofs about waiters and blocking/wake transitions are performed on manually constructed views and do not correspond to any exec-visible state, so invariants about waiters do not imply properties of the runtime queue. **Suggested fix:** connect exec state to waiter ghost state via a shared condvar view or a coupling invariant that relates `Condvar` state to `SemaphoreView.waiters`.
- **Stronger precondition than runtime for overflow** (exec/spec: `up()` requires `value < usize::MAX`). The original `fetch_add` can overflow silently; the verified model forbids it. If the kernel does not prove a global bound, the spec may be too strong. **Suggested fix:** either prove a system-level bound on semaphore values or explicitly model wrap-around behavior (or return an error) to match runtime.

### Low
- None.

## Positive Observations
- Clear module-level documentation enumerates trust assumptions and out-of-scope areas.
- Specs capture the basic counting-semaphore semantics (availability/exhaustion, decrement/increment, mutual exclusion for binary semaphore).
- `wf()` and `spec_wf()` are simple and sufficient for the sequential model, and the proofs are well-organized with split spec/proof files.

## Summary
The verification provides a solid sequential state-machine model but does not establish equivalence to the concurrent atomic implementation or the error semantics of `Condvar` operations. Strengthening the refinement links to atomic/condvar behavior and modeling error outcomes would raise confidence that the verified properties hold for the real kernel semaphore.
