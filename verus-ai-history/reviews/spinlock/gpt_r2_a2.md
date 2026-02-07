# Review: spinlock (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Missing RAII guard coverage** (exec/spec/proof): The API still uses `lock(&mut self) -> Tracked<LockToken>` and `unlock(&mut self, token)` with no `SpinlockGuard` type or `Drop`-based release modeled. This remains a semantic divergence from the original RAII guard API.
- **API and concurrency semantics divergence** (exec `Spinlock::lock`): The verified API still requires `&mut self` and a sequential state machine; it does not verify shared access or atomicity under concurrency.
- **Overly strong precondition for `lock()`** (exec `Spinlock::lock`): `lock()` still requires `spec_is_unlocked()` and `!token_issued()`, so it cannot be called when locked (no spinning behavior), which excludes the original blocking semantics.

### Medium
- **Ghost identity uniqueness remains an assumption** (exec/spec T1): The code still relies on caller-supplied unique ghost IDs without enforcement, so cross-instance token isolation is not guaranteed if IDs collide.
- **Public mutable fields can violate invariants** (exec `Spinlock`): `locked`, `id`, and `token_issued` are still `pub`, so external code can mutate them and break `wf()` or token isolation, undermining proof assumptions.
- **Liveness/progress not verified**: Termination of `lock()` under fairness (spinning progress) remains out of scope and unproven.

### Low
- **`try_lock()` failure branch unreachable under preconditions**: With `wf()` and `!token_issued()`, `locked` must be false, so `try_lock()` always succeeds; the failure path does not model the real CAS failure case.

## Positive Observations
- `lock()` is now implemented (no `external_body`), and acquisition is verified by delegating to `try_lock()` under strong preconditions.
- `wf()` is strengthened to `locked == token_issued`, aligning reachable protocol states with token ownership.
- Documentation more clearly spells out scope and assumptions.

## Summary
The update removes the unverified `lock()` body and clarifies invariants, but the major semantic gaps (RAII `Drop` behavior, concurrent `&self` API, and blocking lock semantics) remain. Verification still does not establish correctness of the original spinlock API or its concurrency properties.
