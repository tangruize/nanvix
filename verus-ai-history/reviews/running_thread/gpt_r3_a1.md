# Review: running_thread (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Coverage**: `RunningThread::join_cond` from the original is missing in the verified exec/spec/proof, so there is no verified model of its behavior or the “wake-all” requirement noted in the source comment. **Location:** missing function in `verus/split/kernel/pm/thread/running.rs` / `running.spec.rs` / `running.proof.rs`. **Suggested Fix:** add a boundary model for `Condvar` and a verified `join_cond()` that at least preserves identity and captures any required wake-all semantics (or mark it as a trusted external with explicit postconditions).
- **Soundness**: `RunningThread::thread_state_mut` is `#[verifier::external]` with no machine-checked postconditions, allowing arbitrary mutation that can break `wf()` and `spec_id()` invariants. **Location:** `running.rs` (exec, non-verus impl). **Suggested Fix:** replace with verified, narrow mutation APIs or add a trusted wrapper with explicit ensures and forbid direct mutation except through verified helpers.

### Medium
- **Specs too weak**: `RunningThread::from_state` does not ensure full state equality (e.g., `spec_interrupt_reason`, `spec_user_tda`, `spec_kernel_stack`, `spec_user_stack`). This allows models where those fields change during construction, which is not semantically equivalent to the original. **Location:** `running.rs` (exec/spec). **Suggested Fix:** strengthen ensures with `result.state@ == state@` or add equality ensures for all relevant spec fields.
- **Equivalence**: `take_mutex_guard` is strengthened to require `spec_has_mutex(address@)` and omits the `Option` return, so the `None` path in the original is unmodeled. **Location:** `running.rs` (exec/spec). **Suggested Fix:** model the `Option` return or weaken preconditions and add postconditions for the “not-held” case.
- **Equivalence**: `put_mutex_guard` requires `!spec_has_mutex(address@)` and `locked_mutex_count < usize::MAX`, which is stronger than the original `BTreeMap::insert` behavior. **Location:** `running.rs` (exec/spec). **Suggested Fix:** model overwrite behavior or add an invariant/lemma proving double-lock cannot occur in verified callers.

### Low
- **Equivalence**: `sleep()`, `schedule()`, and `exit()` omit the `*mut ContextInformation` return, so no property about context-pointer validity/aliasing is verified. **Location:** `running.rs` (exec/spec). **Suggested Fix:** add a ghost spec tying the pointer to the state’s context or explicitly mark this as a trusted HAL boundary with stated obligations.
- **Split boundary**: `SleepingThread`, `ReadyThread`, and `ZombieThread` are boundary models whose postconditions are only documented as cross-module obligations. **Location:** `running.rs` (exec/spec/proof). **Suggested Fix:** ensure sibling modules prove these postconditions or add cross-module proof links to discharge these assumptions.

## Positive Observations
- Clear split between exec/spec/proof with detailed module-level documentation and explicit trust boundaries.
- State-transition specs consistently preserve identity, mutex accounting, drop-safety, and `wf()`.
- Proof lemmas cover identity preservation and mutex accounting across transitions, including a non-trivial acquire-then-release lemma.

## Summary
The verification is well-structured and captures core state-transition preservation properties, but it misses full coverage (notably `join_cond`) and relies on a soundness escape hatch (`thread_state_mut`). Strengthening `from_state`’s postconditions and relaxing/representing the mutex guard APIs to match original semantics would materially improve equivalence and spec strength.
