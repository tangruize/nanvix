# Review: sleeping_process (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `wakeup_alarm` (exec, `verus/split/kernel/pm/process/state/sleeping.rs`).
  **Description:** Alarm handling is still entirely modeled by oracle parameters (`has_expired`, `interrupted_ids`, `remaining_ids`). The spec only enforces partition/shaping constraints, so there is no proven link between `now >= alarm` and which threads are interrupted. This leaves the key safety and liveness properties (“only expired alarms interrupt” and “all expired alarms interrupt”) unverified. This is unchanged from the previous review.
  **Suggested Fix:** Add a ghost model of alarm expiration (e.g., `spec_alarm_expired(tid, now)` or ghost map of alarms) and require `interrupted_ids == filter(spec_alarm_expired)` and `remaining_ids == filter(!spec_alarm_expired)`. Alternatively, make the alarm check an `external_body` that returns a ghost predicate and prove the partition matches it.

### Medium
- **Location:** `wakeup` (exec, `verus/split/kernel/pm/process/state/sleeping.rs`).
  **Description:** The `found: bool` oracle remains a runtime branch driver. While it is tied to `spec_seq_contains`, correctness still depends on a caller-provided boolean rather than a verified search. This is unchanged from the previous review.
  **Suggested Fix:** Make `found` ghost-only and derive the branch from a proof, or move the executable search behind `external_body` and verify the correspondence to `spec_seq_contains`.

- **Location:** `wf` (spec, `verus/split/kernel/pm/process/state/sleeping.spec.rs`).
  **Description:** Uniqueness/disjointness of thread IDs is still assumed in `wf()` without being proven in this module or referenced to a verified construction invariant elsewhere. Proofs rely on this assumption (e.g., removal uniqueness), so soundness depends on a global invariant that is not shown here. This is unchanged from the previous review.
  **Suggested Fix:** Prove these invariants in thread/process construction modules and import the lemmas, or relax the postconditions to accommodate possible duplicates and use multiset reasoning.

### Low
- **Location:** `state` / `state_mut` (exec, `verus/split/kernel/pm/process/state/sleeping.rs`).
  **Description:** Both remain `external_body` even though they can be pure ghost accessors. This keeps an unnecessary trusted surface in a core module. Unchanged from the previous review.
  **Suggested Fix:** Implement them as simple ghost returns of `self.spec_pid()` (and frame conditions for `state_mut`).

## Positive Observations
- All original functions are still represented in exec/spec/proof with a clean split and stable view types.
- Structural conservation (PID immutability, list partitioning order, and thread counts) is clearly specified and proved.
- No new regressions or mismatches were introduced compared to the prior version.

## Summary
The updated module does not actually address the prior review’s concerns: the alarm semantics and wakeup search remain oracle-driven, and uniqueness/disjointness is still assumed without proof. Verification remains structurally strong but behaviorally incomplete. Further work is required before the verification can be considered sound and complete.
