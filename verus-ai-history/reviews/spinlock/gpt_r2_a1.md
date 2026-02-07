# Review: spinlock (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Missing RAII guard coverage** (exec/proof/spec): The original `SpinlockGuard` type and its `Drop`-based unlock are not modeled or verified. The verified code replaces RAII with a `LockToken`/`unlock()` protocol, so the automatic unlock behavior is not proven equivalent to the original API.
  - **Suggested Fix:** Introduce a verified `SpinlockGuard<'a>` in exec code with a tracked token field and a `Drop` implementation that calls `unlock()` (or an `external_body` drop with a proof obligation), and prove it matches the original `Drop` semantics.

- **API and concurrency semantics divergence** (exec `Spinlock::lock`): The verified `lock` uses `&mut self` and models a sequential state machine, while the original uses `&self` with `AtomicBool` interior mutability to allow concurrent callers. This changes the API contract and does not verify mutual exclusion under shared access.
  - **Suggested Fix:** Model lock state with a shared/atomic abstraction (e.g., an atomic ghost state with permissions) so `lock(&self)` is verified under shared access, or provide a wrapper proof that the sequential model refines the concurrent one.

- **Overly strong precondition for `lock()`** (exec `Spinlock::lock`): The verified `lock()` requires the lock to be unlocked, but the real implementation allows calling `lock()` while locked and spins. This is a stronger spec that excludes a major behavior of the original.
  - **Suggested Fix:** Permit `lock()` to be called when locked; model the loop with a fairness/liveness assumption, or split into a `try_lock()` spec and a `lock()` spec that allows blocking until unlock.

- **Unverified core body** (exec `Spinlock::lock`): `lock()` is marked `#[verifier::external_body]` and `unimplemented!()`, so the core acquisition path is trusted rather than verified, despite being a small state transition under the current preconditions.
  - **Suggested Fix:** Implement `lock()` by calling `try_lock()` once and asserting success given `spec_is_unlocked`, eliminating the need for `external_body` in the core module.

### Medium
- **Ghost identity uniqueness is an unproven global assumption** (exec `Spinlock::new`, spec T1): Correctness relies on unique ghost IDs per lock instance, but this is not enforced or modeled. This assumption does not exist in the original implementation which relies on reference identity.
  - **Suggested Fix:** Add a global ghost allocator or a tracked uniqueness token to `new()` to enforce/track uniqueness; alternatively, encode identity via reference borrowing in the spec model.

- **Public mutable fields can violate invariants** (exec `Spinlock`): `locked` and `token_issued` are `pub`, so external code could mutate them directly and break `wf()`, undermining proofs.
  - **Suggested Fix:** Make fields private (or `pub(crate)`), and expose spec-only accessors; add an invariant or encapsulated API to preserve `wf()`.

- **Liveness/progress not verified** (spec/proof scope): The verification does not establish any progress property (e.g., that `lock()` eventually succeeds under fairness). This is an explicit out-of-scope gap, but it means a key property of a spinlock is unverified.
  - **Suggested Fix:** State and prove a liveness lemma under fairness assumptions, or clearly separate safety vs. liveness in the spec and proofs.

### Low
- **`try_lock()` failure branch is unreachable under preconditions** (exec `Spinlock::try_lock`): With `wf()` and `!token_issued()`, `locked` is false, so `result.0 == false` is impossible. This can mislead readers about the real behavior.
  - **Suggested Fix:** Either relax the precondition to allow the locked case (so failure is meaningful), or mark it as an internal helper with a precondition stating unlocked.

## Positive Observations
- The spec clearly documents the sequential model scope and explicitly lists out-of-scope concurrency and liveness aspects.
- The `SpinlockView` and `LockToken` ghost model cleanly capture ownership obligations and prevent cross-instance unlocks.
- Invariants and lemmas are structured to support protocol correctness (new/lock/unlock round-trip, identity preservation, token binding).

## Summary
The verification captures a clean sequential state-machine model with explicit ownership tokens, but it diverges materially from the original API and concurrency semantics and relies on unverified or externalized behaviors. The biggest gaps are missing RAII guard coverage, the stronger precondition on `lock()`, and the reliance on `external_body` in a core function. Addressing these would significantly improve equivalence and soundness for a kernel spinlock.
