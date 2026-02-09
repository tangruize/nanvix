# Review: sleeping_process (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `SleepingProcess::wakeup` (exec)
  - **Description:** Still uses an exec-level `found` oracle instead of searching the sleeping list. The core search logic remains unverified and the exec model is non-equivalent unless the caller provides the correct oracle.
  - **Suggested Fix:** Remove the exec oracle by modeling the sleeping list at exec level and computing membership, or move `found` to `Ghost<bool>` and wrap an exec `external_body` that performs the real search.

- **Location:** `SleepingProcess::wakeup_alarm` (exec/spec)
  - **Description:** Timeout behavior is still driven entirely by oracle parameters (`has_expired`, `interrupted_ids`, `remaining_ids`). There is no modeled relationship to `now` or per-thread alarms, so correctness of alarm expiration is unproven.
  - **Suggested Fix:** Model alarms (e.g., ghost map `tid -> Option<SystemTime>`) and define expired threads via a spec predicate tied to `now`; prove the partition matches that predicate.

### Medium
- **Location:** `RunnableProcess::wf` / `InterruptedProcess::wf` (spec)
  - **Description:** Boundary well-formedness remains too weak (only non-empty ready/interrupted lists). It does not enforce uniqueness or disjointness across thread lists, so ownership safety across transitions is not preserved.
  - **Suggested Fix:** Strengthen boundary `wf()` predicates to include no-duplicates and cross-list disjointness, then update proofs accordingly.

### Low
- **Location:** `state()` / `state_mut()` (exec)
  - **Description:** Both remain `external_body` with frame conditions but no modeling of how `ProcessState` mutation can affect invariants. PID immutability is assumed but not enforced at call sites.
  - **Suggested Fix:** Require/prove `mutation_frame_preserved` at call sites or model PID in `ProcessState` with an invariant preventing changes.

- **Location:** `terminate()` / `wakeup_alarm()` (spec)
  - **Description:** `InterruptReason` is still elided, so the proof does not establish that `Killed`/`TimedOut` reasons are used as in the original.
  - **Suggested Fix:** Add a ghost reason map or extend interrupted thread IDs with reason tags and specify them in postconditions.

## Positive Observations
- Stable-ordering constraints for `wakeup_alarm` remain in place via `spec_is_subsequence`.
- Sleeping-process invariants (non-empty sleeping list, no duplicates, disjointness) are explicit and preserved.
- PID preservation and thread-list conservation are specified.

## Summary
No substantive changes address the remaining high-risk issues: both `wakeup` and `wakeup_alarm` still rely on oracles, leaving core behavior unverified and non-equivalent. Boundary invariants remain weak and interrupt reasons are not modeled. Verification is still incomplete and not fully sound for the intended behavior.
