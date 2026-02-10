# Review: kcall_lock_mutex (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Safety preconditions still not enforced in exec model.**
  - **Location:** `spec_lock_mutex_safety_preconditions` (lock_mutex.spec.rs), `lock_mutex_model` (lock_mutex.rs, exec).
  - **Description:** The model still omits `pid`/`tid` and does not add a wrapper or requires clause enforcing `spec_lock_mutex_safety_preconditions`. The updated comments explicitly say these are out of scope, so the original unsafe-call contract remains unverified at this module boundary.
  - **Suggested Fix:** Add a `lock_mutex_model_with_context(pid, tid, ...)` wrapper with a `requires spec_lock_mutex_safety_preconditions(pid, tid)` and prove it delegates to `lock_mutex_model`, or enforce these preconditions in the verified kcall dispatcher.

- **Guard drop/unlock on put_guard failure is only documented, not modeled.**
  - **Location:** `put_mutex_guard_model` (lock_mutex.rs, exec), `lemma_guard_consumed_on_all_paths` (lock_mutex.proof.rs).
  - **Description:** The update adds comments and a guard-token consumption lemma, but there is still no state predicate or postcondition that proves the mutex is unlocked on the `PutGuardError` path. The hook `spec_mutex_unlocked_after_guard_drop` remains commented out and unproven.
  - **Suggested Fix:** Introduce a mutex-state ghost predicate (or reuse one from the mutex module) and add a real postcondition on `put_mutex_guard_model` for the error path that establishes “unlocked after drop,” then connect it to the exec pipeline via a lemma.

### Low
- **Architecture guard still not tied to actual target width.**
  - **Location:** `USIZE_BITS`, `lemma_architecture_guard` (lock_mutex.spec.rs / lock_mutex.proof.rs).
  - **Description:** The guard remains a hardcoded constant with no link to the real `usize` width. The new comments acknowledge this, but there is still no verification-time check that fails on non-32-bit targets.
  - **Suggested Fix:** Add a verified link to `size_of::<usize>() * 8` or a Verus-level target-width assertion that fails if the target changes.

## Positive Observations
- Added guard-token consumption documentation and lemma improves the narrative around ownership flow.
- Pipeline short-circuiting and error propagation remain well-specified and verified.
- Verification still passes cleanly with the updated module.

## Summary
The updates mostly add documentation and a guard-consumption lemma, but the core missing guarantees from the prior review remain: safety preconditions are not enforced and guard-drop unlock semantics are still unmodeled. The architecture-width guard is still a hardcoded assumption. The verification is solid for the pipeline logic, but not complete for the caller safety contract and lock-release semantics.
