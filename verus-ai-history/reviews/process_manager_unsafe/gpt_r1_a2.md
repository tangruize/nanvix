# Review: process_manager_unsafe (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Priority: High**
  - **Location:** `ProcessManagerUnsafeState::wf` (spec) + T9 trust boundary (exec/spec).
  - **Description:** The PID↔TID membership invariant is still absent. The updated files only add a trust-boundary comment; `wf()` remains unchanged and does not require that `current_tid` belongs to `current_pid`'s thread set. This leaves key safety properties (sleep/exit_thread/wakeup/try_recv) unproven with respect to thread ownership.
  - **Suggested Fix:** Extend `ProcessManagerInner` with a ghost PID→thread-set map (or equivalent predicate) and strengthen `wf()` with `current_tid ∈ threads(current_pid)`. Propagate this across switch/sleep/exit_thread/wakeup specs.

- **Priority: High**
  - **Location:** `exit()` and `exit_thread()` (exec).
  - **Description:** Divergence is still not modeled. The new T10 comments explain the limitation but do not provide a formal non-returning spec, so callers can still reason about post-call states as if control continues in the exiting thread.
  - **Suggested Fix:** Introduce an explicit non-returning contract (e.g., separate success/error specs with an unreachable-success branch or a ghost “returned” flag that is always false on success) to prevent post-call reasoning on the success path.

### Medium
- **Priority: Medium**
  - **Location:** `join_thread_*` (exec).
  - **Description:** The wait path remains a trust boundary and there is still no wrapper spec for the loop or any progress argument. The updated comments do not provide a formal liveness or termination guarantee.
  - **Suggested Fix:** Add a top-level `join_thread` spec that sequences the loop with a ghost progress measure or formally assumes eventual `notify_all` in a dedicated lemma.

- **Priority: Medium**
  - **Location:** `try_recv_some` / `try_recv_none` (exec).
  - **Description:** Still only models a decrement of `number_buffered_messages` with no per-thread queue state or message identity. The added T11 comment acknowledges the gap but does not fix it.
  - **Suggested Fix:** Add ghost per-thread message queues in the inner model and specify that `try_recv_some` removes exactly one message for `tid` while `try_recv_none` requires the queue empty.

- **Priority: Medium**
  - **Location:** `get_mutex`, `put_mutex_guard`, `get_cond`, `put_cond`, `take_mutex_guard` (exec).
  - **Description:** These functions still return only a boolean success indicator and model no state changes, despite the real code mutating mutex/condvar tables and guard ownership. The T12 comment does not provide a formal link to inner-state effects.
  - **Suggested Fix:** Model synchronization tables/ownership explicitly (ghost maps) or thread the updated inner state through these APIs to reflect mutations.

### Low
- None.

## Positive Observations
- The previous giveup no-switch under-specification is fixed: `giveup` now requires `new_inner == old(self).inner` when `remaining_quantum > 1`.
- Coverage remains complete and there are no `assume`/`external_body` shortcuts in this module.
- Spec/proof separation remains clean and verification still passes.

## Summary
The update mostly adds trust-boundary commentary rather than formal fixes. The giveup no-switch issue is resolved, but the key safety invariant (PID↔TID membership), divergence semantics of exit paths, and the weak modeling of IPC and synchronization remain. Verification is still partial and not yet sound for these critical properties.
