# Review: sleeping_process (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `SleepingProcess::wakeup` (exec)
  - **Description:** The function still takes an exec-level `found` oracle and does not perform the sleeping-list search. This leaves the core search logic unverified and makes the exec model non-equivalent unless callers provide the correct oracle.
  - **Suggested Fix:** Remove the exec oracle by modeling the sleeping list at exec level and computing membership, or move `found` to `Ghost<bool>` and wrap an exec `external_body` that performs the real search.

- **Location:** `SleepingProcess::wakeup_alarm` (exec/spec)
  - **Description:** Alarm expiry is still entirely controlled by oracle parameters (`has_expired`, `interrupted_ids`, `remaining_ids`). The proof does not relate these to `now` or per-thread alarms, so timeout behavior is not verified and the model remains non-equivalent.
  - **Suggested Fix:** Model per-thread alarms (e.g., ghost map `tid -> Option<SystemTime>`) and define expired threads via a spec predicate tied to `now`; prove the partition matches that predicate.

### Medium
- **Location:** `RunnableProcess::wf` / `InterruptedProcess::wf` (spec)
  - **Description:** Boundary well-formedness still only enforces non-empty ready/interrupted lists. It does not capture uniqueness or disjointness across thread lists, so ownership safety across state transitions is not preserved.
  - **Suggested Fix:** Strengthen boundary `wf()` predicates to include no-duplicates and cross-list disjointness, then update proofs accordingly.

### Low
- **Location:** `state()` / `state_mut()` (exec)
  - **Description:** Both remain `external_body` with frame conditions but no modeling of `ProcessState` mutation effects. PID immutability is asserted but not enforced at call sites.
  - **Suggested Fix:** Introduce an explicit `mutation_frame_preserved` requirement for `state_mut()` callers, or model PID in `ProcessState` with an invariant preventing changes.

- **Location:** `terminate()` / `wakeup_alarm()` (spec)
  - **Description:** `InterruptReason` is still elided, so the proof does not establish that `Killed`/`TimedOut` are used as in the original implementation.
  - **Suggested Fix:** Add a ghost reason map or extend interrupted thread IDs with reason tags and specify them in postconditions.

## Positive Observations
- The stable ordering issue in `wakeup_alarm` was addressed by adding a subsequence predicate and corresponding pre/postconditions.
- Sleeping-process invariants (non-empty sleeping list, no duplicates, disjointness) remain explicit and preserved.
- PID preservation and thread-list conservation are still clearly specified.

## Summary
The rework improves ordering guarantees for `wakeup_alarm`, but the core high-risk issues remain: `wakeup` and `wakeup_alarm` still rely on exec-level oracles, leaving key behavior unverified and non-equivalent. Boundary invariants are still weak, and interrupt reasons remain unmodeled. Verification is improved but not yet complete or sound for the full intended behavior.
