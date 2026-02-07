# Review: mutex (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `Mutex::lock` (exec: `verus/split/kernel/pm/sync/mutex.rs`)
  - **Description:** The verified `lock()` requires `spec_is_unlocked()` and `!token_issued()` and then delegates to `try_lock()` once. This excludes the contended, blocking behavior of the original (`loop` + `Condvar::wait()`), omits timeout handling, and does not model `SleepError`. As a result, the verification does not cover the essential blocking semantics or any liveness/progress properties of the real mutex.
  - **Suggested Fix:** Model the contended case with a loop and a ghost model of `Condvar` wait/notify (or link to the condvar spec) and include timeout/`SleepError` in the spec. If liveness is out of scope, at least specify and prove a blocking contract that relates `lock()` to `Condvar::wait()` and eventual acquisition under fairness assumptions.

- **Location:** `Mutex::try_lock` / module model (exec: `verus/split/kernel/pm/sync/mutex.rs`)
  - **Description:** The sequential `&mut self` model does not capture atomicity, interleavings, or linearizability of the CAS-based `AtomicBool` operations in the real implementation. Mutual exclusion is only proven in a single-threaded state machine, which is insufficient for an OS kernel mutex.
  - **Suggested Fix:** Provide a concurrent model or refinement proof that relates the `AtomicBool` CAS to a linearization point (e.g., using Verus `atomic` specs or a shared-state invariant). Prove mutual exclusion and absence of lost wakeups under concurrent execution.

### Medium
- **Location:** Coverage gaps (exec/spec/proof)
  - **Description:** The verified split omits several functions from the original: `Mutex::reference_count`, `MutexInner::unlock_unchecked`, `MutexGuard::drop`, and `fmt::Debug` for `MutexGuard`. `unlock()` is a model of drop-based release but does not capture the `notify_first()` call or its error path. This fails the “all functions covered” criterion and leaves behavior unverified.
  - **Suggested Fix:** Add spec stubs or verified wrappers for these functions (e.g., `reference_count` as a spec-opaque value with minimal guarantees, `unlock_unchecked`/`drop` modeled as `unlock` plus a `Condvar::notify_first` effect, and `Debug` as a pure formatting function).

- **Location:** `Mutex` identity assumptions (exec/spec: `verus/split/kernel/pm/sync/mutex.rs`, `mutex.spec.rs`)
  - **Description:** Correctness relies on a caller-provided ghost `id` being globally unique, but there is no enforced global invariant. If IDs collide, token isolation and instance separation proofs become unsound. The invariant `wf()` is purely local and does not prevent cross-instance misuse.
  - **Suggested Fix:** Introduce a global ghost allocator or registry of IDs (e.g., tracked `Set<nat>`) and require uniqueness at `new()` construction. Alternatively, connect IDs to an abstract `Arc` identity model to match runtime uniqueness.

### Low
- **Location:** `Mutex::unlock` postcondition (exec: `verus/split/kernel/pm/sync/mutex.rs`)
  - **Description:** The postcondition resets the view to `spec_new_view`, but the original mutex may still have waiters queued in `Condvar`. The model ignores any queue state and the wakeup effect, which may be acceptable in isolation but is an incomplete behavioral match.
  - **Suggested Fix:** Extend the model with a ghost waiter queue or a link to condvar invariants, and specify that unlock performs a notify action.

## Positive Observations
- The split is clean: exec code is separated from spec and proof files, and the verified module is self-contained.
- The state-machine protocol (locked/unlocked, token tracking, no double-unlock) is clearly specified and mechanically verified.
- No `assume` or `external_body` appears in the mutex module, and the verification script passes (`./verus-ai/scripts/verify.sh mutex`).

## Summary
The verification provides a solid sequential state-machine proof but omits the core concurrent and blocking semantics that make a kernel mutex correct and useful. To raise confidence, the model should be strengthened to cover contention, waiting/wakeup, and atomicity/linearizability, and the missing functions should be brought under verification (or explicitly modeled with precise specs). As-is, the proof is helpful but incomplete for an OS-kernel correctness bar.
