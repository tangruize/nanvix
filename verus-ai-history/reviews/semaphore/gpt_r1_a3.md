# Review: semaphore (gpt-5.2-codex)

## Grade: C+

## Issues Found

### Critical
- None.

### High
- **Location:** `Semaphore::down` (exec, `verus/split/kernel/pm/sync/semaphore.rs`)
  - **Description:** `down()` still requires `spec_is_available()` and only models the instant-success path; the blocking loop, waiter queueing, and `SleepError` behavior of the real semaphore are not modeled. The new ghost `CallerContext` does not address this semantic gap.
  - **Suggested Fix:** Model the blocking path explicitly (e.g., via a spec state transition tied to condvar wait) or introduce a verified wrapper that accounts for both immediate and blocking behavior.

### Medium
- **Location:** `Semaphore::up` (exec, `verus/split/kernel/pm/sync/semaphore.rs`)
  - **Description:** `up()` still returns `()` and ignores the `Condvar::notify_first()` error path present in the original `Result<(), Error>`. No postcondition connects notification outcome to a return value.
  - **Suggested Fix:** Return a result type or add a postcondition that ties condvar notification success/failure to the return value and prove it against the condvar spec.

- **Location:** Waiter tracking (`SemaphoreView`, `View`, blocking lemmas; spec/proof)
  - **Description:** `SemaphoreView.waiters` remains disconnected from exec state: `view()` always returns `waiters = 0`, and no exec function updates it. The new blocking/wake lemmas are still purely ghost and do not reflect actual blocking threads.
  - **Suggested Fix:** Introduce ghost state updates in exec `down()`/`up()` (or a modeled condvar wrapper) so waiters correspond to real blocking behavior, and use that in proofs.

- **Location:** `spec_condvar_wake_after_notify` (spec, `verus/split/kernel/pm/sync/semaphore.spec.rs`)
  - **Description:** The condvar wake assumption remains unlinked to the condvar module’s verified interface (explicitly documented as such). This is still an unproven cross-module contract.
  - **Suggested Fix:** Define/import a shared condvar interface spec and prove `notify_first()` satisfies the wake transition used by the semaphore model.

- **Location:** `Semaphore::up` overflow precondition (exec)
  - **Description:** The precondition `value < usize::MAX` remains stronger than the real implementation (which can wrap). There is still no global invariant establishing this bound.
  - **Suggested Fix:** Prove a global bound from system/resource invariants or explicitly model wraparound in the spec.

### Low
- None.

## Positive Observations
- Unsafe caller obligations are now encoded as explicit ghost preconditions via `CallerContext` and enforced in `down()`/`up()`.
- The `try_down()` result mapping is now formalized (`spec_try_down_result_maps_ok`) with a proof lemma connecting it to the postconditions.

## Summary
Some prior issues were fixed (unsafe preconditions and `try_down` mapping), but core semantic gaps remain: blocking behavior, condvar error propagation, waiter tracking, cross-module condvar contract, and overflow modeling. Verification is still incomplete relative to the real semaphore’s behavior.
