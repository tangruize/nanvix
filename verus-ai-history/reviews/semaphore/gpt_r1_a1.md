# Review: semaphore (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `Semaphore::down` (exec, `verus/split/kernel/pm/sync/semaphore.rs`)
  - **Description:** The verified `down()` models only the instant-success path (`spec_is_available()` precondition) and omits the blocking loop, sleeping on `Condvar`, and `SleepError` propagation present in the original. This misses core semaphore semantics and liveness behavior.
  - **Suggested Fix:** Model `down()` as a result type that can either succeed immediately or take a blocking step. Introduce ghost waiter tracking tied to the condvar queue and specify/verify the sleep–wake protocol with explicit error propagation.

- **Location:** `Semaphore::down` / `Semaphore::up` (exec, `verus/split/kernel/pm/sync/semaphore.rs`)
  - **Description:** The original functions are `unsafe` with required safety conditions (interrupts disabled, not kernel process, no held resources). These preconditions are not expressed in the verified model, so misuse constraints are not captured.
  - **Suggested Fix:** Add explicit preconditions or a ghost permission model encoding the safety conditions, and require them in `down()`/`up()` specifications.

### Medium
- **Location:** `Semaphore::up` (exec, `verus/split/kernel/pm/sync/semaphore.rs`)
  - **Description:** The verified `up()` always succeeds and omits the `Condvar::notify_first()` error path (`Result<(), Error>` in the original). This is a semantic gap for error propagation.
  - **Suggested Fix:** Return a result type in the verified model, or at least specify a postcondition mapping the condvar notification outcome to the return value and connect it to the condvar spec.

- **Location:** `spec_condvar_wake_after_notify` (spec, `verus/split/kernel/pm/sync/semaphore.spec.rs`)
  - **Description:** The condvar wake assumption is only documented and not linked to the condvar module’s verified interface. It is not used to establish a cross-module contract, so wake correctness is effectively assumed rather than proved.
  - **Suggested Fix:** Import a shared condvar interface spec (or define one) and prove that `notify_first()` establishes the wake transition used by the semaphore spec.

- **Location:** `View` implementation / waiter lemmas (spec/proof)
  - **Description:** `SemaphoreView.waiters` is always `0` in `view()`, so exec code never updates the waiter count. Proofs about waiters and liveness (`lemma_all_waiters_eventually_served`) are disconnected from the exec model and do not reflect actual waiting threads.
  - **Suggested Fix:** Add ghost state updates in `down()`/`up()` (or a wrapped condvar model) so waiters correspond to real blocking behavior, and use that in proofs.

- **Location:** `Semaphore::up` precondition (exec, `verus/split/kernel/pm/sync/semaphore.rs`)
  - **Description:** The verified model assumes `value < usize::MAX` to avoid overflow, but the original `fetch_add` can wrap silently in release builds. Without an invariant that bounds the value, the spec is stronger than the implementation.
  - **Suggested Fix:** Prove a global bound on the semaphore count (e.g., from resource pool size), or model overflow explicitly and reflect it in the spec.

### Low
- **Location:** `Semaphore::try_down` return type (exec, `verus/split/kernel/pm/sync/semaphore.rs`)
  - **Description:** The verified API returns `bool` instead of `Result<(), Error>` with `ErrorCode::TryAgain`. Although behavior is similar, the error code mapping is not specified.
  - **Suggested Fix:** Add a spec lemma documenting the mapping (`false` ≡ `Err(TryAgain)`) or use a result enum mirroring the original signature.

## Positive Observations
- Clear spec/proof separation with a dedicated `SemaphoreView` and `wf()` predicate.
- Good documentation of the verification scope, trust assumptions, and API mapping.
- Core arithmetic safety properties (conservation, monotonicity, mutual exclusion for value=1) are proven.
- No `assume`/`external_body` usage in the semaphore module; proofs are explicit.

## Summary
The verification provides a solid sequential-state model for resource counting, but it omits the blocking semantics, error propagation, and unsafe usage preconditions that are central to the kernel semaphore’s correctness. Connecting the ghost waiter model to real condvar behavior and specifying the sleep/wake protocol would significantly improve coverage and equivalence. Addressing overflow and error-path modeling would close remaining semantic gaps.
