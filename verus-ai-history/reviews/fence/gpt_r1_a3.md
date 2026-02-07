# Review: fence (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `Fence::wait` (exec, `verus/split/kernel/pm/sync/fence.rs`).
  **Description:** Still modeled as a no-op with precondition `spec_is_satisfied`. This does not
  verify the blocking/liveness behavior of the runtime spin-wait loop. The added proof lemma does
  not introduce any environment/fairness model or termination argument.
  **Suggested Fix:** Add an explicit progress/environment model or a trusted lemma that captures
  eventual signaling, and prove `wait` terminates when enough signals occur.

- **Location:** `Fence::signal` precondition and `Fence::wf` (exec/spec,
  `verus/split/kernel/pm/sync/fence.rs`, `verus/split/kernel/pm/sync/fence.spec.rs`).
  **Description:** The model still enforces `count <= total` and requires `spec_is_waiting` before
  `signal`, forbidding over-signaling. The runtime allows `signal` at any time and permits
  `count > total`, so the verified contract remains stronger than runtime semantics.
  **Suggested Fix:** Weaken the spec/precondition to allow over-signaling (bounded only by
  overflow), or modify the runtime to enforce the same protocol.

### Medium
- **Location:** Fence model/proofs (exec/proof, `verus/split/kernel/pm/sync/fence.rs`,
  `verus/split/kernel/pm/sync/fence.proof.rs`).
  **Description:** Verification remains purely sequential (`&mut self` for `signal`) and does not
  model atomicity or memory ordering, so correctness under concurrent interleavings is still
  unproven. The new `lemma_signals_accumulate_to_satisfaction` is purely arithmetic and does not
  reason about state transitions or concurrency, so it does not close this gap.
  **Suggested Fix:** Use Verus atomic modeling or a concurrency protocol (rely/guarantee, ghost
  invariants) to connect to the atomic runtime behavior.

### Low
- None.

## Positive Observations
- Added lemma is consistent with the arithmetic model and does not weaken any invariants.

## Summary
The prior issues remain: `wait` liveness is still assumed, signaling remains over-constrained, and
concurrent correctness is not modeled. The new proof lemma does not change the verification scope,
so verification is still incomplete relative to runtime semantics.
