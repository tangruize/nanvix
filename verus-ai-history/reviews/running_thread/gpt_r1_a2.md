# Review: running_thread (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `RunningThread::join_cond` (missing in verus `running.rs` exec/spec).
  **Description:** The public `join_cond()` API is still omitted; there is no modeled `Condvar` or spec relating join semantics, so synchronization behavior remains unverified.
  **Suggested Fix:** Add a modeled `join_cond()` with an opaque/ghost `Condvar` token and a spec that preserves identity (or an explicit trusted boundary with obligations).

- **Location:** `RunningThread::store_mutex_guard` / `RunningThread::take_mutex_guard` (verus `running.rs` exec).
  **Description:** The API is still strengthened: `store_mutex_guard` requires `!spec_has_mutex`, and `take_mutex_guard` requires `spec_has_mutex` and returns unit, removing the `None` path from the original `Option<MutexGuard>`. The note acknowledges this but it remains a spec/behavior mismatch.
  **Suggested Fix:** Model the `Option` return and allow the not-found case, or add a verified wrapper that enforces the stronger preconditions at runtime.

### Medium
- **Location:** `RunningThread::thread_state_mut` (verus `running.rs` non-verus `#[verifier::external]`).
  **Description:** Still an unchecked escape hatch with no machine-checked postconditions; callers can violate `wf()` or `spec_id()` with no proof obligations.
  **Suggested Fix:** Provide a verified wrapper with explicit ensures, or add a trusted lemma/axiom that callers must prove before mutation.

### Low
- **Location:** `RunningThread::sleep` / `schedule` / `exit` (verus `running.rs` exec).
  **Description:** The raw `*mut ContextInformation` return values remain omitted; there is no abstract token relating the returned context pointer to the thread state.
  **Suggested Fix:** Introduce an abstract context token or ghost pointer identity to connect the boundary.

- **Location:** `ReadyThread` boundary model (verus `running.rs` exec).
  **Description:** The `admission_time` field is still omitted, so scheduling fairness/time-ordering properties cannot be expressed.
  **Suggested Fix:** Add an abstract `admission_time` field with minimal spec (e.g., fresh/monotonic) if those properties are needed.

## Positive Observations
- Transition specs now assert `result.state@ == self.state@`, and proof lemmas (`lemma_*_preserves_state_view`) were added, fixing the prior weak-spec gap for preserving full `ThreadStateView` across `sleep/schedule/exit`.
- `thread_state()` now guarantees identity and full state view equality, tightening that API.

## Summary
The update fixes the previously weak transition specs by preserving the full `ThreadState` view, but key gaps remain: missing `join_cond`, strengthened mutex-guard API, and the unchecked `thread_state_mut` escape hatch. Boundary omissions for context pointers and admission time persist. Verification is improved but still incomplete and not fully sound until these issues are addressed.
