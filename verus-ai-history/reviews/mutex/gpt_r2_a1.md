# Review: mutex (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- **Location:** `Mutex::lock` (exec: verus/split/kernel/pm/sync/mutex.rs)
  **Description:** The verified `lock()` requires `spec_is_unlocked()` and `!token_issued()` and then succeeds immediately, while the original `lock()` can be called when contended and blocks via `Condvar::wait()` with a possible `SleepError`. This makes the spec too strong and not semantically equivalent to the real API in the contended case, which is the core use of a mutex in the kernel.
  **Suggested Fix:** Model blocking semantics with an abstract wait/condvar protocol (e.g., a ghost queue or fairness assumption) and allow `lock()` to be called in the locked state, or explicitly verify a separate `lock_uncontended()` API and map it to the real `lock()` with a proof obligation.
- **Location:** `Mutex::try_lock`/`Mutex::lock` model (exec/spec)
  **Description:** The verified model uses `&mut self` with a plain `bool` and does not model atomicity, memory ordering, or concurrent interleavings. As a result, the proof does not establish the key safety property of mutual exclusion under concurrent access, which is the essential correctness property of a mutex.
  **Suggested Fix:** Introduce a concurrent/atomic model (e.g., a verified atomic CAS spec or a rely-guarantee model) or clearly constrain the verified API to a sequential-only mutex and prove a refinement layer that justifies use in the concurrent kernel.

### High
- **Location:** `MutexToken` (spec: verus/split/kernel/pm/sync/mutex.spec.rs)
  **Description:** `MutexToken` has a `pub ghost view` field, so external code can fabricate tokens without calling `lock()`/`try_lock()`, making the unlock preconditions satisfiable without actually holding the lock. This is a soundness hole acknowledged by T3, but it weakens the proof significantly.
  **Suggested Fix:** Make token construction private (e.g., keep `MutexToken` opaque in the spec module, or use a sealed tracked type/constructor) so tokens can only be obtained through the API.
- **Location:** Identity assumptions for `id` (exec/spec: mutex.rs/mutex.spec.rs)
  **Description:** Correctness relies on the ghost `id` being unique per mutex instance, but uniqueness is not enforced. If two mutexes share the same `id` and state, a token from one could satisfy the other’s unlock precondition.
  **Suggested Fix:** Provide a verified allocator for unique IDs, or model identity via a tracked ownership resource that cannot be duplicated, to eliminate the global uniqueness assumption.

### Medium
- **Location:** `Mutex::reference_count`, `MutexInner::unlock_unchecked`, `MutexGuard::drop`, `fmt::Debug` (original: src/kernel/src/pm/sync/mutex.rs)
  **Description:** These public or safety-critical functions lack direct verified counterparts. The model replaces RAII `Drop` with explicit `unlock()` and omits `reference_count()` entirely, so coverage is incomplete.
  **Suggested Fix:** Add verified wrappers or stubs with precise specs for `reference_count()` and `Drop`/`unlock_unchecked()` behavior, and document any deliberate exclusions with a justification in the API mapping.
- **Location:** `Mutex::lock`/`MutexInner::unlock_unchecked` (exec model)
  **Description:** The verified API omits timeout handling and `SleepError`, and does not model `notify_first()` failure or the `warn!()` path in `Drop`. This weakens equivalence and under-specifies observable error behavior.
  **Suggested Fix:** Add result/error variants or abstract error modeling in the spec to reflect the original API’s failure modes, even if the internals are left uninterpreted.

### Low
- **Location:** `Mutex::wf` invariant (spec: mutex.spec.rs)
  **Description:** The invariant only ties `locked` to `token_issued` and does not capture any condvar/waiting-thread state, which limits what can be proven about progress or wakeup behavior.
  **Suggested Fix:** Extend the state view with an abstract wait set or a counter, or explicitly declare that waiting behavior is outside the verified scope.

## Positive Observations
- The spec/proof/exec split is clean, and the documentation clearly explains the intended verification scope and trust assumptions.
- Core sequential protocol properties (lock/unlock transitions, token binding, and identity preservation) are proved with well-structured lemmas.
- No `assume` or `external_body` is used in the mutex module itself, which is good for soundness within the chosen model.

## Summary
The verification provides a solid sequential state-machine model but falls short on the essential concurrent correctness and blocking semantics of the kernel mutex. Strengthening the model to capture contention, atomicity, and token soundness would materially improve coverage and equivalence to the real implementation.
