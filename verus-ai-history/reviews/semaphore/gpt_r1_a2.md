# Review: semaphore (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `Semaphore::down` (exec, `verus/split/kernel/pm/sync/semaphore.rs`)
  - **Description:** `down()` still requires `spec_is_available()` and only models the instant-success path; the blocking loop, waiter queueing, and `SleepError` behavior of the real semaphore are not modeled. The new documentation acknowledges this but does not fix the semantic gap.
  - **Suggested Fix:** Model the blocking path explicitly (e.g., a result enum or a ghost state transition tied to condvar wait), or add a verified wrapper that covers both immediate and blocking behaviors.

- **Location:** `Semaphore::down` / `Semaphore::up` (exec, `verus/split/kernel/pm/sync/semaphore.rs`)
  - **Description:** The unsafe caller obligations (interrupts disabled, non-kernel process, no held resources/PM ref) are still not expressed as preconditions; they are only documented as trust assumptions. This leaves the verified API usable in states that are unsafe in the kernel.
  - **Suggested Fix:** Encode these conditions as explicit preconditions or a ghost capability/permission token that must be provided to call `down()`/`up()`.

### Medium
- **Location:** `Semaphore::up` (exec, `verus/split/kernel/pm/sync/semaphore.rs`)
  - **Description:** `up()` still returns `()` and ignores the `Condvar::notify_first()` error path present in the original `Result<(), Error>`. No formal mapping to the condvar result is specified.
  - **Suggested Fix:** Return a result type or add a postcondition that connects the condvar notification outcome to the return value and prove it against the condvar spec.

- **Location:** Waiter tracking (`SemaphoreView`, `View`, blocking lemmas; spec/proof)
  - **Description:** `SemaphoreView.waiters` remains disconnected from exec state (the `view()` always sets `waiters = 0`, and no exec function updates it). The new `spec_down_blocking`/`spec_wake` lemmas operate only on ghost views, so proofs about waiters and wakeup behavior still do not reflect actual exec behavior.
  - **Suggested Fix:** Introduce ghost state updates in exec `down()`/`up()` (or a modeled condvar wrapper) so that waiters correspond to real blocking behavior, and use that in the proofs.

- **Location:** `spec_condvar_wake_after_notify` (spec, `verus/split/kernel/pm/sync/semaphore.spec.rs`)
  - **Description:** The condvar wake assumption is explicitly documented as not linked to the condvar module's verified interface. This remains an unproven cross-module contract.
  - **Suggested Fix:** Define/import a shared condvar interface spec and prove `notify_first()` satisfies the wake transition used by the semaphore model.

- **Location:** `Semaphore::up` precondition (exec, `verus/split/kernel/pm/sync/semaphore.rs`)
  - **Description:** The overflow precondition `value < usize::MAX` is still stronger than the real implementation (which can wrap). There is no global invariant establishing this bound.
  - **Suggested Fix:** Prove a bound from system/resource invariants or explicitly model wraparound in the spec.

### Low
- **Location:** `Semaphore::try_down` API mapping (exec/spec)
  - **Description:** The mapping from `bool` to `Err(TryAgain)` is now documented, but it is still not formalized as a spec lemma or result type. The verification does not enforce the error-code correspondence.
  - **Suggested Fix:** Add a spec lemma documenting the mapping or use a small result enum mirroring the original signature.

## Positive Observations
- The verification scope and trust assumptions are now clearly documented and make the limitations explicit.
- The spec-level blocking/wake transitions and lemmas are internally consistent and better organized.

## Summary
Most prior issues remain unresolved; the updates are primarily documentation and ghost-spec additions without tying them to exec semantics. Verification is still incomplete and unsound with respect to blocking behavior, unsafe preconditions, and condvar interaction.
