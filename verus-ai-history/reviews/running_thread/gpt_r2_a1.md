# Review: running_thread (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Coverage gap: `join_cond()` omitted** (exec/spec/proof: `verus/split/kernel/pm/thread/running.rs`).
  - **Description:** The original `RunningThread::join_cond()` is not modeled or verified at all. This violates the coverage requirement and leaves the join condition-variable API unverified.
  - **Suggested Fix:** Add a boundary model for `Condvar` and a verified `join_cond()` that returns an opaque handle (or mark it `#[verifier::external]` with a spec that it is a pure clone and preserves `wf()`/identity). At minimum, include a stub function so the interface is fully covered.

- **API strengthening for mutex guards** (exec/spec: `verus/split/kernel/pm/thread/running.rs` and `verus/split/kernel/pm/thread/state.rs`).
  - **Description:** `put_mutex_guard()` requires `!spec_has_mutex(address@)` and `take_mutex_guard()` requires `spec_has_mutex(address@)`, while the original uses `BTreeMap::insert/remove` and returns `Option<MutexGuard>` (allowing missing cases and overwrites). This makes the verified API strictly stronger and assumes invariants (T1/T2) outside the verification boundary.
  - **Suggested Fix:** Model the `Option<MutexGuard>` return and allow the not-found/overwrite paths in the spec, or explicitly carry these invariants as assumptions in higher-level specifications and audit all call sites for enforcement.

- **Unspecified external escape hatch** (exec: `verus/split/kernel/pm/thread/running.rs::thread_state_mut`).
  - **Description:** `thread_state_mut()` is `#[verifier::external]` with no machine-checked postconditions, allowing arbitrary mutation of `ThreadState` and potential violation of `wf()`/identity. This is a soundness hole if used beyond trusted callers.
  - **Suggested Fix:** Replace uses with verified forwarding methods (preferred), or wrap with an `external_body` plus explicit trusted postconditions that preserve `wf()` and `spec_id()`; reduce/ban use in verified code paths.

### Low
- None.

## Positive Observations
- Good separation of exec/spec/proof files with explicit trust boundary documentation.
- Specs preserve identity, state view, mutex accounting, and drop-safety across transitions.
- Boundary models for Sleeping/Ready/Zombie clearly state cross-module obligations.

## Summary
The verification is mostly faithful and well-structured, but it omits `join_cond()` entirely and strengthens the mutex-guard APIs via preconditions that are not enforced in the original implementation. The `thread_state_mut()` external escape hatch also leaves a soundness gap unless strictly contained. Addressing these would elevate confidence and bring the coverage and equivalence criteria to an A-level.
