# Review: sleeping_process (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Location:** `SleepingProcess::wakeup_alarm` (exec/spec). **Description:** Alarm handling is modeled purely by oracle partitions (`interrupted_ids`/`remaining_ids`) with only structural constraints; there is no link to actual per-thread alarms or `now`, so the verification does not prove that expired alarms are interrupted or that unexpired alarms remain sleeping. This leaves the core time-based correctness/liveness property unverified. **Suggested Fix:** Model alarm timestamps in ghost state (e.g., `Map<tid, Option<SystemTime>>`) and define an `alarm_expired(tid, now)` predicate; require `interrupted_ids` to equal the set (and stable subsequence) of expired alarms and prove the partition from the implementation.

### Medium
- **Location:** `RunnableProcess::wf` and `InterruptedProcess::wf` (spec). **Description:** Well-formedness only checks non-empty ready/interrupted lists and omits uniqueness/disjointness across thread lists, weakening ownership invariants that the original Rust types implicitly guarantee. This allows verified transitions to produce states that violate global thread-identity invariants. **Suggested Fix:** Extend boundary `wf()` predicates to include no-duplicates and disjointness across all thread lists, and strengthen postconditions in `terminate`, `wakeup`, and `add_thread` accordingly.
- **Location:** `find_thread` / `find_thread_mut` (exec/spec). **Description:** These are modeled as ghost-only list tags and do not capture reference semantics or the effects of mutation through `&mut ThreadRef`, so equivalence with the original API is partial. **Suggested Fix:** Introduce an abstract thread-state model (even if ghost-only) and require that any mutation preserves thread identity/list membership and `wf()`; alternatively, wrap with `external_body` plus explicit frame/ownership constraints.
- **Location:** `terminate` and `wakeup_alarm` (exec/spec). **Description:** The interrupt reason (`Killed` vs `TimedOut`) is elided, so the verification does not establish that the correct reason is recorded for later logic. **Suggested Fix:** Extend the interrupted-thread model to include `(tid, reason)` pairs or add a ghost mapping from thread IDs to interrupt reasons and prove the correct reason assignment.

### Low
- **Location:** `SleepingProcess::add_thread` (exec/spec). **Description:** The spec adds a precondition that the added `ready_tid` is not already in sleeping/zombie lists; this is stronger than the local code unless a global uniqueness invariant is proven elsewhere. **Suggested Fix:** Either prove this precondition from a global invariant or move uniqueness into the module-wide `wf()` assumptions so callers are not over-constrained.
- **Location:** `state` / `state_mut` (exec). **Description:** Both are `external_body`, so correctness relies on a trust boundary without modeling `ProcessState` beyond PID immutability. **Suggested Fix:** Model the relevant `ProcessState` fields or provide a verified wrapper that enforces PID immutability and any required invariants.

## Positive Observations
- All original functions are represented in the verified module, and the split between exec/spec/proof is clean and well-documented.
- The model captures key structural invariants for sleeping/zombie lists (non-empty, no duplicates, disjointness) and preserves ordering via subsequence constraints in `wakeup_alarm`.
- `wakeup` and `terminate` preserve PID and thread conservation properties with explicit postconditions.

## Summary
The verification is structurally solid and covers the API surface, but the most important behavioral gap is the unmodeled alarm logic in `wakeup_alarm`, which leaves time-based correctness and liveness unproven. Strengthening boundary invariants and modeling reference semantics/reasons would improve equivalence and soundness. Addressing these gaps would likely move the grade into the A range.
