# Review: process_manager_unsafe (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Priority: High**
  - **Location:** `ProcessManagerUnsafeState::wf` (spec) + T9 trust boundary (exec/spec).
  - **Description:** The PID↔TID membership invariant is still absent. The updated files only add broader trust-boundary commentary; `wf()` remains unchanged and does not require that `current_tid` belongs to `current_pid`'s thread set. This leaves key safety properties (sleep/exit_thread/wakeup/try_recv) unproven with respect to thread ownership.
  - **Suggested Fix:** Extend `ProcessManagerInner` with a ghost PID→thread-set map (or equivalent predicate) and strengthen `wf()` with `current_tid ∈ threads(current_pid)`. Propagate this across switch/sleep/exit_thread/wakeup specs.

- **Priority: High**
  - **Location:** `exit()` and `exit_thread()` (exec).
  - **Description:** Divergence is still not modeled. The new scope/limitations text asserts Verus cannot express non-returning behavior, but the spec still allows callers to reason about post-call states as if control continues in the exiting thread.
  - **Suggested Fix:** Add a formal non-returning contract (e.g., split success/error specs with an unreachable-success branch or a ghost flag forbidding continuation) so post-call reasoning on the success path is impossible.

### Medium
- **Priority: Medium**
  - **Location:** `join_thread_*` (exec).
  - **Description:** The wait path remains a trust boundary and there is still no wrapper spec for the loop or any progress argument. The new limitations section does not provide a formal liveness/termination guarantee.
  - **Suggested Fix:** Add a top-level `join_thread` spec that sequences the loop with a ghost progress measure or explicitly assume eventual `notify_all` in a dedicated lemma.

- **Priority: Medium**
  - **Location:** `try_recv_some` / `try_recv_none` (exec).
  - **Description:** Still only models a decrement of `number_buffered_messages` with no per-thread queue state or message identity. The added comments acknowledge the gap but do not fix it.
  - **Suggested Fix:** Add ghost per-thread message queues in the inner model and specify that `try_recv_some` removes exactly one message for `tid` while `try_recv_none` requires the queue empty.

- **Priority: Medium**
  - **Location:** `get_mutex`, `put_mutex_guard`, `get_cond`, `put_cond`, `take_mutex_guard` (exec).
  - **Description:** These functions still return only a boolean success indicator and model no state changes, despite the real code mutating mutex/condvar tables and guard ownership. The extended trust-boundary text is not a formal link to inner-state effects.
  - **Suggested Fix:** Model synchronization tables/ownership explicitly (ghost maps) or thread the updated inner state through these APIs to reflect mutations.

### Low
- None.

## Positive Observations
- The giveup no-switch fix remains in place (`new_inner == old(self).inner` when `remaining_quantum > 1`).
- No `assume`/`external_body` shortcuts were introduced, and verification still passes.
- The module’s scope and trust boundaries are now clearly documented.

## Summary
The update mostly expands documentation and scope statements without making the prior formal gaps go away. The key safety invariant (PID↔TID membership), divergence semantics of exit paths, and weak IPC/synchronization modeling remain unresolved. Verification is still partial and not yet sound for these critical properties.
