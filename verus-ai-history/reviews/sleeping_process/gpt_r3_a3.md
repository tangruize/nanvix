# Review: sleeping_process (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `wakeup_alarm` (exec, `verus/split/kernel/pm/process/state/sleeping.rs`).
  **Description:** Alarm handling is still modeled entirely by oracle parameters (`has_expired`, `interrupted_ids`, `remaining_ids`). The spec only constrains the partition shape and ordering, so there is no verified link between `now >= alarm` and which threads are interrupted. This leaves the safety/liveness properties (“only expired alarms interrupt” and “all expired alarms interrupt”) unproven. No evidence of a fix in the updated files.
  **Suggested Fix:** Add a ghost model of alarm expiration (predicate or map) and require the partition equals filtering by that predicate, or add a trusted alarm-checking `external_body` and prove its correspondence.

### Medium
- **Location:** `wakeup` (exec, `verus/split/kernel/pm/process/state/sleeping.rs`).
  **Description:** The `found: bool` oracle remains a runtime branch driver; the function still depends on caller-supplied truth rather than a verified search. This is unchanged, so semantic equivalence to `remove_if` is still assumption-based.
  **Suggested Fix:** Make `found` ghost-only and derive the branch from a proof, or implement a verified/`external_body` search tied to `spec_seq_contains`.

- **Location:** `wf` (spec, `verus/split/kernel/pm/process/state/sleeping.spec.rs`).
  **Description:** Uniqueness/disjointness of thread IDs is still assumed in `wf()` without proof in this module or a referenced invariant from construction code. Proofs rely on this assumption, so soundness is conditional on an unverified global invariant. No new evidence of a fix.
  **Suggested Fix:** Prove these invariants in construction modules and import the lemmas, or relax the specs and use multiset reasoning.

### Low
- **Location:** `state` / `state_mut` (exec, `verus/split/kernel/pm/process/state/sleeping.rs`).
  **Description:** Both remain `external_body` even though they can be pure ghost accessors. This keeps unnecessary trusted surface area. No change observed.
  **Suggested Fix:** Implement as direct ghost returns with frame conditions.

## Positive Observations
- Coverage remains complete across exec/spec/proof with a clean split and view types.
- Structural properties (PID immutability, conservation, ordering constraints) are explicitly captured and proved.
- No new regressions or mismatches were introduced compared to the prior version.

## Summary
The updated files do not address the prior issues; all major concerns remain unchanged. Verification is structurally solid but behaviorally incomplete and still relies on oracles and unproven invariants. Further fixes are required before the verification can be considered sound and complete.
