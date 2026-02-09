# Review: sleeping_process (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `SleepingProcess::wakeup` (exec)
  - **Description:** The verified function takes an exec-level `found` oracle to decide success, rather than searching the sleeping list as in the original. This makes the exec model non-equivalent unless callers always provide the correct oracle, and the search logic itself is unverified.
  - **Suggested Fix:** Eliminate the exec oracle by modeling the sleeping list in exec state (e.g., store IDs in an exec vector) and compute `found` internally, or make `found` a `Ghost<bool>` and convert `wakeup` into a spec/proof function with an external-body exec wrapper.

- **Location:** `SleepingProcess::wakeup_alarm` (exec/spec)
  - **Description:** Alarm handling is entirely delegated to oracle parameters (`has_expired`, `interrupted_ids`, `remaining_ids`). The verification does not connect these to `now` or per-thread alarms, so the timeout logic (`now >= alarm`) is not proven and the exec model is not semantically equivalent.
  - **Suggested Fix:** Model alarms explicitly (e.g., a ghost map `tid -> Option<SystemTime>`) and define expired sets via a spec function; then prove `interrupted_ids` and `remaining_ids` are the stable partition of the sleeping list by this predicate.

### Medium
- **Location:** `SleepingProcess::wakeup_alarm` (spec)
  - **Description:** The spec only enforces a set-partition; it does not preserve the relative order of interrupted vs remaining threads. The original uses queue order and produces a stable partition.
  - **Suggested Fix:** Strengthen the spec to require that `interrupted_ids` and `remaining_ids` are subsequences of the original list in the same order, and that their concatenation is the original list.

- **Location:** `RunnableProcess::wf` / `InterruptedProcess::wf` (spec)
  - **Description:** Boundary invariants are very weak (only non-empty ready/interrupted). They do not enforce uniqueness or disjointness across thread lists, so safety properties about thread ownership are not preserved across state transitions.
  - **Suggested Fix:** Extend wf predicates to include no-duplicates and disjointness across ready/sleeping/zombie/interrupted lists, and update proofs accordingly.

### Low
- **Location:** `state()` / `state_mut()` (exec)
  - **Description:** Both are `external_body` and assume PID and thread lists remain unchanged, but there is no model of how `ProcessState` mutation could violate invariants (e.g., PID change).
  - **Suggested Fix:** Model `ProcessState` identity explicitly (ghost field + invariant) or require/prove `mutation_frame_preserved` at call sites that use `state_mut()`.

- **Location:** `terminate()` / `wakeup_alarm()` (spec)
  - **Description:** `InterruptReason` is elided, so the verification does not prove that `Killed`/`TimedOut` are used as in the original.
  - **Suggested Fix:** Add a ghost reason field to interrupted thread IDs or a separate map `tid -> reason` and specify the expected reasons in postconditions.

## Positive Observations
- All original functions (public and private) have verified counterparts with clear specs.
- `SleepingProcess::wf()` captures key list invariants (non-empty sleeping list, disjointness, no duplicates).
- PID preservation and thread-list conservation are explicitly specified and proved.
- Spec/proof separation is clean and consistent with the split structure.

## Summary
The verification captures structural properties (PID preservation, list conservation) well, but relies heavily on oracles for `wakeup` and especially `wakeup_alarm`, leaving core timeout/search behavior unverified and non-equivalent to the original. Strengthening boundary invariants and specifying stable ordering would materially improve correctness guarantees. Overall the split is clean, but the spec is currently too weak on key behaviors.
