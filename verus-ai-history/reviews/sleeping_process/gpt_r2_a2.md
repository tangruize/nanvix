# Review: sleeping_process (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `SleepingProcess::wakeup_alarm` (exec/spec). **Description:** The alarm logic is still modeled purely by oracle partitions (`interrupted_ids`/`remaining_ids`) with only structural constraints. There is no modeled per-thread alarm state or link to `now`, so the proof still does not establish that expired alarms are interrupted or that unexpired alarms remain sleeping. This is the same core behavioral gap as before. **Suggested Fix:** Add ghost alarm metadata (e.g., `Map<tid, Option<SystemTime>>`), define `alarm_expired(tid, now)`, and require `interrupted_ids` to be exactly the stable subsequence of expired alarms computed from the model.

### Medium
- **Location:** `RunnableProcess::wf` and `InterruptedProcess::wf` (spec). **Description:** Well-formedness remains a non-empty check only, with no uniqueness/disjointness across ready/interrupted/sleeping/zombie lists. This still allows states that violate ownership invariants enforced by the original Rust types. **Suggested Fix:** Extend boundary `wf()` predicates to enforce no-duplicates and disjointness across all thread lists, then strengthen postconditions in `terminate`, `wakeup`, and `add_thread`.
- **Location:** `find_thread` / `find_thread_mut` (exec/spec). **Description:** These remain ghost-only list tags and do not model reference semantics or mutation effects; equivalence to the original APIs is still partial. **Suggested Fix:** Introduce an abstract thread-state model with identity and list-membership invariants, and require any mutation through `find_thread_mut` to preserve `wf()`.
- **Location:** `terminate` and `wakeup_alarm` (exec/spec). **Description:** Interrupt reasons (`Killed` vs `TimedOut`) are still elided, so correctness of reason propagation is unproven. **Suggested Fix:** Model interrupted threads as `(tid, reason)` pairs or add a ghost map from thread IDs to interrupt reasons and prove the correct assignment.

### Low
- **Location:** `SleepingProcess::add_thread` (exec/spec). **Description:** The precondition forbidding `ready_tid` collisions remains stricter than the local source unless justified by a global invariant. **Suggested Fix:** Either prove the precondition from a global uniqueness invariant or encode it in `wf()` so callers are not over-constrained.
- **Location:** `state` / `state_mut` (exec). **Description:** Both are still `external_body`, leaving `ProcessState` behavior unmodeled beyond PID immutability. **Suggested Fix:** Model the relevant `ProcessState` fields or add a verified wrapper that enforces PID immutability and invariants.

## Positive Observations
- Coverage is still complete: all original functions have verified counterparts with clear specs.
- The split between exec/spec/proof remains clean and readable with explicit invariants and helper lemmas.
- `wakeup` and `terminate` preserve PID and thread conservation properties with explicit postconditions.

## Summary
I do not see substantive fixes to the prior issues; the core time-based correctness in `wakeup_alarm` remains unmodeled and boundary invariants are still weak. No new regressions were introduced, but the verification is not yet complete or fully sound for the intended behavior. Addressing the alarm modeling and boundary invariants is still necessary to reach an A-level result.
