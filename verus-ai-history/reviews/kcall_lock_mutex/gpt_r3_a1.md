# Review: kcall_lock_mutex (gpt-5.2-codex)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Safety preconditions not enforced in exec model.**
  - **Location:** `spec_lock_mutex_safety_preconditions` (lock_mutex.spec.rs), `lock_mutex_model` (lock_mutex.rs, exec).
  - **Description:** The three unsafe-call safety requirements are modeled as uninterpreted predicates, but `lock_mutex_model` does not require them (nor does any wrapper with pid/tid). This makes the verified contract weaker than the real safety contract and allows proofs to ignore critical caller-side conditions.
  - **Suggested Fix:** Add a verified wrapper that includes `pid`/`tid` and a `requires spec_lock_mutex_safety_preconditions(pid, tid)` (or enforce these preconditions in the kcall dispatcher proof), and connect it to `lock_mutex_model` via a lemma.

- **Missing modeling of guard drop/unlock on put-guard failure.**
  - **Location:** `lock_mutex_model` (lock_mutex.rs, exec) and `put_mutex_guard_model` (lock_mutex.rs, exec).
  - **Description:** In the original code, if `put_mutex_guard` fails, the `MutexGuard` is dropped at scope exit, releasing the lock. The model does not capture this state change or prove that the mutex is unlocked on this error path, which can hide lock-leak/deadlock properties.
  - **Suggested Fix:** Extend the model with a ghost mutex-state or guard-ownership predicate that is consumed on any exit path, or add a lemma in the mutex/PM modules that ensures failed `put_mutex_guard` still releases the guard.

### Low
- **Architecture guard is not tied to actual target width.**
  - **Location:** `USIZE_BITS`, `USIZE_MAX_X86_32`, `lemma_architecture_guard` (lock_mutex.spec.rs / lock_mutex.proof.rs).
  - **Description:** The guard is a hardcoded constant; the lemma does not check the real `usize` width. Verification would still pass even if compiled under a 64-bit target, risking unsoundness for the `usize`→`u32` casts.
  - **Suggested Fix:** Add a verified link to `size_of::<usize>() * 8` (or a `#[cfg(target_pointer_width = "32")]` assertion in Verus) so the proof fails if the target changes.

## Positive Observations
- Exec model closely mirrors the original control flow with clean short-circuit proofs and explicit error propagation.
- Timeout parsing is modeled precisely (including the MAX/MAX infinite sentinel) with correct error-code linkage.
- Ownership is tracked via a guard token that ties `lock` success to `put_mutex_guard` preconditions.
- Spec/proof separation is clear, with well-factored helper predicates and lemmas.

## Summary
The verification captures the core pipeline semantics and error propagation well, but it leaves the unsafe-call safety preconditions and the guard-drop side effect on the error path unmodeled. Tightening those contracts and strengthening the architecture guard would make the model more faithful and reduce unsoundness risk.
