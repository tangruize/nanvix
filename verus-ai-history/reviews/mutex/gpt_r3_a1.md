# Review: mutex (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Coverage gaps** (exec/spec/proof): `Mutex::reference_count()`, `MutexInner::unlock_unchecked()`, `MutexGuard::drop()`, and `fmt::Debug for MutexGuard` from the original are not modeled or verified. This violates coverage requirements and leaves RAII-based unlock and error-logging behavior unverified. **Suggested Fix:** Add verified wrappers or explicit model functions for these APIs (even if modeled as no-ops for display), and prove `Drop`-style unlock semantics against the token model.
- **Lock semantics too strong** (exec: `lock`): The verified `lock(&mut self)` requires `spec_is_unlocked()` and omits timeout/`SleepError` handling, while the original `lock(&self, timeout)` blocks and can be called on a locked mutex. This changes observable behavior and omits liveness and contended-lock semantics. **Suggested Fix:** Model blocking/timeout behavior (e.g., as a state machine with a wait step), or weaken equivalence claims and add a separate verified spec for contended `lock()`.
- **Token forgery trust gap** (spec/proof: `MutexToken`, exec: `unlock`): `MutexToken` exposes a `pub ghost view`, so external code can fabricate tokens; mutual exclusion relies on Trust Assumption T3 rather than an enforced invariant. **Suggested Fix:** Encapsulate token construction (e.g., use a private field with a trusted constructor pattern) or add a sealed module boundary to prevent external token fabrication.

### Medium
- **ID uniqueness is assumed, not enforced** (spec: `spec_new_view`, exec: `new`): Correctness relies on unique `id` per mutex instance (T1), but there is no invariant or constructor enforcement. If IDs collide, token isolation and unlock correctness break. **Suggested Fix:** Model IDs as an abstract, non-forgeable token (e.g., a tracked unique id resource) or add a global ghost allocator invariant.
- **Condvar interaction and wake-up behavior omitted** (exec/spec/proof): The verified model does not capture `Condvar::wait()`/`notify_first()` or the error/logging path in `unlock_unchecked()`/`Drop`, so progress guarantees and wakeup semantics are unproven. **Suggested Fix:** Add an abstract condvar state machine or a refinement lemma relating unlock to a wakeup effect.

### Low
- **Atomicity/memory-ordering not modeled** (exec: `try_lock`, `unlock`): The sequential `&mut self` model ignores `AtomicBool` ordering and shared aliasing, so equivalence to non-x86 memory models is unaddressed. **Suggested Fix:** Document this as an explicit refinement limitation or add a linearizability lemma that ties atomic operations to the sequential model under stated architecture assumptions.

## Positive Observations
- Clear separation of exec/spec/proof with extensive scope documentation and explicit trust boundaries.
- Core state-machine invariant `wf()` is simple and consistently used across operations.
- Proofs cover important sequential safety properties (lock/unlock round-trip, no double unlock, token/identity binding).

## Summary
The verification is a well-documented sequential model, but it omits several original APIs and the contended/timeout behavior that defines the mutex’s real-world semantics. The reliance on a forgeable token and assumed ID uniqueness leaves key soundness gaps. Strengthening coverage and refining the blocking behavior model would significantly improve confidence.
