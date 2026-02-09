# Review: sleeping_process (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `SleepingProcess::wakeup_alarm` (exec/spec). **Description:** The alarm logic is still modeled via oracle partitions only, with no modeled per-thread alarm state or `now` comparison, so the verification does not prove that expired alarms are interrupted or that unexpired alarms remain sleeping. This core behavioral gap remains unaddressed. **Suggested Fix:** Add ghost alarm metadata (e.g., `Map<tid, Option<SystemTime>>`), define `alarm_expired(tid, now)`, and constrain `interrupted_ids` to be exactly the stable subsequence of expired alarms computed from that model.

### Medium
- **Location:** `RunnableProcess::wf` and `InterruptedProcess::wf` (spec). **Description:** Well-formedness remains only a non-empty check, with no uniqueness/disjointness across all thread lists, allowing states that violate ownership invariants guaranteed by the original Rust types. **Suggested Fix:** Extend boundary `wf()` predicates to enforce no-duplicates and disjointness across all thread lists, then strengthen postconditions in `terminate`, `wakeup`, and `add_thread`.
- **Location:** `find_thread` / `find_thread_mut` (exec/spec). **Description:** These still return only ghost list tags and do not model reference semantics or mutation effects; equivalence to the original API remains partial. **Suggested Fix:** Introduce an abstract thread-state model and require mutation through `find_thread_mut` to preserve identity/list membership and `wf()`.
- **Location:** `terminate` and `wakeup_alarm` (exec/spec). **Description:** Interrupt reasons (`Killed` vs `TimedOut`) are still elided, so correctness of reason propagation is unproven. **Suggested Fix:** Model interrupted threads as `(tid, reason)` pairs or maintain a ghost map of interrupt reasons.

### Low
- **Location:** `SleepingProcess::add_thread` (exec/spec). **Description:** The precondition forbidding `ready_tid` collisions is still stronger than the local source unless justified by a global invariant. **Suggested Fix:** Prove the precondition from a global uniqueness invariant or encode it into `wf()`.
- **Location:** `state` / `state_mut` (exec). **Description:** These remain `external_body`, so `ProcessState` behavior is unmodeled beyond PID immutability. **Suggested Fix:** Model the relevant `ProcessState` fields or add a verified wrapper enforcing PID immutability and invariants.

## Positive Observations
- Coverage remains complete and the exec/spec/proof split is clean and well-structured.
- Structural invariants on sleeping/zombie lists and conservation properties are clear and preserved.
- `wakeup` and `terminate` maintain PID and thread conservation with explicit postconditions.

## Summary
I do not see substantive fixes to the prior issues; the most important behavioral gaps persist, especially the unmodeled alarm semantics and weak boundary invariants. No new regressions were introduced, but the verification is not yet complete or fully sound for the intended behavior. Addressing the alarm modeling and boundary invariants is still required for an A-level result.
