# Review: running_thread (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Coverage**: `RunningThread::join_cond` is still omitted, so there is no verified
  model or wake-all semantics for the condvar boundary. **Location:**
  `verus/split/kernel/pm/thread/running.rs` / `running.spec.rs`.
  **Suggested Fix:** add a modeled/trusted `join_cond()` with explicit
  postconditions or a boundary spec for condvar semantics.
- **Soundness**: `RunningThread::thread_state_mut` remains `#[verifier::external]`
  with no machine-checked postconditions, so callers can violate `wf()`/identity
  invariants. **Location:** `running.rs`. **Suggested Fix:** replace with verified
  narrow mutation APIs or add a trusted wrapper with explicit ensures and forbid
  direct mutation except through verified helpers.

### Medium
- **Equivalence**: `put_mutex_guard` still requires `!spec_has_mutex(address@)`
  and `locked_mutex_count < usize::MAX`, which is stronger than the original
  `BTreeMap::insert` behavior; no proof that double-lock cannot occur is provided.
  **Location:** `running.rs`. **Suggested Fix:** model overwrite behavior or prove
  double-lock impossible in all verified callers.
- **Equivalence**: `take_mutex_guard` still requires `spec_has_mutex(address@)` and
  elides the `Option<MutexGuard>` return, so the `None` path is unmodeled.
  **Location:** `running.rs`. **Suggested Fix:** model the `Option` return or
  weaken the precondition and add postconditions for the not-held case.

### Low
- **Equivalence**: `sleep()`, `schedule()`, and `exit()` still omit the
  `*mut ContextInformation` return, so no property about context-pointer
  validity/aliasing is verified. **Location:** `running.rs`. **Suggested Fix:**
  add a ghost spec tying the pointer to the state’s context or explicitly mark
  this as a trusted HAL boundary with stated obligations.
- **Split boundary**: `SleepingThread`, `ReadyThread`, and `ZombieThread` remain
  boundary models whose obligations are only documented, not discharged.
  **Location:** `running.rs` / `running.spec.rs` / `running.proof.rs`.
  **Suggested Fix:** ensure sibling modules prove these postconditions or add
  cross-module proof links.

## Positive Observations
- `from_state` now preserves the full `ThreadStateView` (`result.state@ == state@`),
  fixing the prior weak-spec gap.
- Transition specs and lemmas consistently preserve state view and mutex
  accounting.

## Summary
The prior weak `from_state` spec was fixed, but the major gaps remain:
`join_cond` is still unmodeled and `thread_state_mut` is still an unchecked escape
hatch. The mutex guard APIs are still strengthened beyond the original behavior,
and boundary models remain unproven, so verification is not yet complete or fully
sound.
