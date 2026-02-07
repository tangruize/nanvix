# Review: mutex (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Coverage gaps still unresolved** (exec/spec/proof): `Mutex::reference_count()`, `MutexInner::unlock_unchecked()`, `MutexGuard::drop()`, and `fmt::Debug for MutexGuard` remain unmodeled and unverified. The updated files still only document these as out-of-scope, so the RAII unlock path and error logging behavior are still uncovered.
- **Lock semantics remain too strong** (exec: `lock`): `lock(&mut self)` still requires `spec_is_unlocked()` and omits timeout and blocking behavior. This continues to diverge from the original `lock(&self, timeout)` behavior, so contention and timeout semantics remain unverified.
- **Token forgery trust gap persists** (spec: `MutexToken`): `MutexToken` still exposes a `pub ghost view` and relies on Trust Assumption T3. There is no enforcement preventing external token fabrication.

### Medium
- **ID uniqueness still assumed, not enforced** (spec: `spec_new_view`, exec: `new`): Trust Assumption T1 remains unchanged and no global invariant or allocator prevents `id` collisions, so token isolation is still conditional on an external assumption.
- **Condvar interaction and wake-up behavior still omitted** (exec/spec/proof): The revised text reiterates out-of-scope behavior, but the verification still does not model `Condvar::wait()`/`notify_first()` or the error/logging path in `unlock_unchecked()`/`Drop`. Progress/wakeup semantics remain unproven.

### Low
- **Atomicity/memory-ordering remains informal** (exec/spec): The x86 TSO refinement note is still not machine-checked, and the model remains silent on weakly ordered architectures.

## Positive Observations
- Documentation of scope and trust assumptions is clear and consistent.
- The sequential token-based protocol remains mechanically checked with `wf()`.
- No new regressions were introduced in the verified functions.

## Summary
The updates remain documentation-focused and do not close the previously reported verification gaps. Core coverage omissions, contended-lock semantics, and token/ID soundness assumptions still hold, so verification remains incomplete relative to the runtime mutex behavior.
