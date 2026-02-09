# Review: sleeping_process (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Location:** `wakeup_alarm` (exec, `verus/split/kernel/pm/process/state/sleeping.rs`).
  **Description:** Alarm handling is modeled entirely by oracle parameters (`has_expired`, `interrupted_ids`, `remaining_ids`). The spec only enforces length/content/subsequence constraints, so any arbitrary partition can be justified, and there is no proven link between `now >= alarm` and the set of interrupted threads. This misses the key safety property “only threads with expired alarms are interrupted” and the liveness property “all expired alarms are interrupted.”
  **Suggested Fix:** Introduce a ghost model of per-thread alarm expiration (e.g., `spec_alarm_expired(tid, now)` or a ghost map of alarm times) and require `interrupted_ids == filter(spec_alarm_expired)` and `remaining_ids == filter(!spec_alarm_expired)`. Alternatively, add a trusted external_body that computes expiration and then prove that the partition corresponds to that predicate.

### Medium
- **Location:** `wakeup` (exec, `verus/split/kernel/pm/process/state/sleeping.rs`).
  **Description:** The `found: bool` oracle parameter replaces the internal search in the original code. Although tied to `spec_seq_contains`, the implementation branches solely on this caller-provided value, so semantic equivalence to `remove_if` depends on an external assumption rather than verified logic.
  **Suggested Fix:** Make `found` ghost-only and derive the branch from a proof (or move the executable portion behind `external_body` and prove a lemma that matches `spec_seq_contains`). This removes reliance on a runtime oracle.

- **Location:** `wf` (spec, `verus/split/kernel/pm/process/state/sleeping.spec.rs`).
  **Description:** The model assumes no duplicate thread IDs and disjoint sleeping/zombie lists. The original module does not enforce these conditions locally, so correctness proofs (e.g., “removed thread no longer present”) become conditional on an unproven global invariant.
  **Suggested Fix:** Either prove these uniqueness/disjointness invariants in the process/thread construction modules and reference those lemmas here, or relax `wf`/postconditions to permit duplicates (and adjust proofs to use multiset reasoning).

### Low
- **Location:** `state` / `state_mut` (exec, `verus/split/kernel/pm/process/state/sleeping.rs`).
  **Description:** Both are `external_body` despite being representable as pure ghost accessors. This introduces unnecessary trusted surface in a core module.
  **Suggested Fix:** Implement these bodies as `Ghost(self.spec_pid())` (and `Ghost(old(self).spec_pid())` for the mutable variant) to avoid an unverified boundary.

## Positive Observations
- All original functions are represented in exec/spec/proof with clear mapping and documented trust boundaries.
- The specs capture key structural properties: PID immutability, thread conservation, and stable ordering in `wakeup_alarm` via subsequence constraints.
- Proofs establish wf preservation and provide helper lemmas for list operations, supporting cross-module composition via View types.

## Summary
The verification is well-structured and largely faithful for structural properties, but the alarm semantics are left as an oracle and the wakeup search relies on an external boolean, leaving key correctness properties unproven. Strengthening the alarm model and removing runtime oracles would substantially improve soundness and equivalence. Overall, the split quality is good, but core behavioral guarantees are currently incomplete.
