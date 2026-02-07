# Review: mutex (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Coverage gaps still unresolved** (exec/spec/proof): The updated model still does not implement or verify `Mutex::reference_count()`, `MutexInner::unlock_unchecked()`, `MutexGuard::drop()`, or `fmt::Debug for MutexGuard`. The new API mapping explicitly marks these as “not modeled,” so the prior coverage issue remains unfixed; the RAII unlock path and error logging behavior are still unverified.
- **Lock semantics remain too strong** (exec: `lock`): `lock(&mut self)` still requires `spec_is_unlocked()` and omits timeout and blocking behavior. This continues to diverge from the original `lock(&self, timeout)` semantics, so the prior contention/timeout concern is not fixed—only documented.
- **Token forgery trust gap persists** (spec: `MutexToken`): `MutexToken` still exposes a `pub ghost view` and relies on Trust Assumption T3. The fix is documentation-only; there is still no enforcement preventing external token fabrication.

### Medium
- **ID uniqueness still assumed, not enforced** (spec: `spec_new_view`, exec: `new`): Trust Assumption T1 remains unchanged; no global invariant or allocator is introduced to prevent `id` collisions, so token isolation remains conditional on an external assumption.
- **Condvar interaction and wake-up behavior still omitted** (exec/spec/proof): The revised text clarifies out-of-scope behavior, but the underlying verification still does not model `Condvar::wait()`/`notify_first()` or the error/logging path in `unlock_unchecked()`/`Drop`. Progress/wakeup semantics remain unproven.

### Low
- **Atomicity/memory-ordering remains informal** (exec/spec): The added x86 TSO refinement argument is not machine-checked, and the model still ignores atomic ordering on weakly ordered architectures. This is a documentation improvement but not a verified refinement.

## Positive Observations
- The Init/Token commentary is clearer and the trust assumptions are explicitly documented.
- The token-based protocol and `wf()` invariant remain consistent and mechanically checked for the sequential model.
- The API mapping table makes divergence from the runtime implementation explicit.

## Summary
The prover mostly documented limitations rather than closing the previously reported gaps. Core coverage omissions, contended-lock semantics, and token/ID soundness assumptions remain; verification is still a sequential model with significant behavioral divergence from the runtime mutex. Verification is improved in clarity but not complete or sound relative to the original implementation.
