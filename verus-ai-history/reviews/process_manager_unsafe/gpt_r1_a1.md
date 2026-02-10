# Review: process_manager_unsafe (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Priority: High**
  - **Location:** `ProcessManagerUnsafeState::wf` (spec) + T9 trust boundary comment (exec/spec).
  - **Description:** The invariant does not relate `current_tid` to `current_pid`'s thread set. This omits the essential safety property that the running thread belongs to the running process, so correctness of `sleep`, `exit_thread`, `wakeup`, and `try_recv` is proved without establishing basic PID/TID consistency.
  - **Suggested Fix:** Extend `ProcessManagerInner` with a ghost map of PID→thread set (or a per-process thread membership predicate) and strengthen `wf()` to require `current_tid` is a member of `current_pid`'s set. Propagate this invariant through `switch`, `exit_thread`, `sleep`, and `wakeup` specs.

- **Priority: High**
  - **Location:** `exit()` and `exit_thread()` (exec).
  - **Description:** The verified model explicitly does not capture divergence (T10). In the real code, successful `exit`/`exit_thread` never return, but the model allows callers to reason about post-call states as if control continues.
  - **Suggested Fix:** Model the non-returning success path explicitly (e.g., use a `-> !`-like spec, split success/error paths with `ensures false` on success, or add a ghost flag that forces caller proof obligations to treat the success path as unreachable).

### Medium
- **Priority: Medium**
  - **Location:** `giveup()` (exec).
  - **Description:** When `remaining_quantum > 1`, the model ignores `new_inner` but only constrains its `running_pid`. This under-specifies the no-switch path and allows callers to provide an arbitrary mutated `new_inner` without proof obligations, weakening equivalence.
  - **Suggested Fix:** Require `new_inner == old(self).inner` when `remaining_quantum > 1`, or split into two functions so the no-switch path takes no `new_inner` parameter.

- **Priority: Medium**
  - **Location:** `join_thread_*` (exec).
  - **Description:** The model splits `join_thread` into harvest/wait/error paths but does not provide a wrapper spec for the loop or a progress/liveness argument. The wait path is explicitly treated as a trust boundary, so blocking/termination behavior is not verified.
  - **Suggested Fix:** Add a top-level `join_thread` spec that sequences the loop steps with a ghost progress measure, or at least specify a liveness assumption (e.g., eventual `notify_all`) in a dedicated lemma.

- **Priority: Medium**
  - **Location:** `try_recv_some/try_recv_none` (exec).
  - **Description:** The model only decrements `number_buffered_messages` and does not relate message existence to the `tid` argument or specify the returned message. This is too weak to prove per-thread message delivery correctness.
  - **Suggested Fix:** Introduce ghost state for per-thread message queues and specify that `try_recv_some` removes exactly one message for `tid`, while `try_recv_none` requires the queue is empty.

- **Priority: Medium**
  - **Location:** `get_mutex`, `put_mutex_guard`, `get_cond`, `put_cond`, `take_mutex_guard` (exec).
  - **Description:** These operations are modeled as boolean success/failure with no state change. In reality they create/lookup mutexes/condvars and transfer ownership of guards; omitting this state makes the specs too weak for synchronization correctness.
  - **Suggested Fix:** Add ghost maps for mutex/condvar tables and guard ownership, and specify how these maps change on success/failure.

### Low
- **Priority: Low**
  - **Location:** `get()` / `get_mut()` (exec).
  - **Description:** Specs assume `wf()` (initialized) and do not model the panic path when the process manager is uninitialized, so failure behavior is not verified.
  - **Suggested Fix:** Add a separate spec/lemma for the uninitialized case or model a two-outcome behavior (panic vs. return) to cover the full API contract.

## Positive Observations
- All original functions appear to have verified counterparts; complex functions are split into path-specific models that make proofs manageable.
- `switch()` correctly models the stale atomic PID/TID read and quantum reset behavior, matching the original control flow.
- No `assume`/`external_body` shortcuts were found in this module; verification is not bypassed.
- Spec/proof separation is clean via dedicated `.spec.rs` and `.proof.rs` includes.

## Summary
The verification provides good structural coverage and a faithful model of the switch/quantum logic, but key safety properties (PID↔TID membership) and non-returning semantics of exit paths are not captured. Several APIs are modeled with minimal success/failure booleans, which is too weak for synchronization and IPC correctness. Strengthening invariants and adding ghost state for queues and synchronization objects would significantly improve the soundness of the model.
