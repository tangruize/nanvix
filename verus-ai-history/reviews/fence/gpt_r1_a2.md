# Review: fence (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `Fence::wait` (exec, `verus/split/kernel/pm/sync/fence.rs`).
  **Description:** Still modeled as a no-op with precondition `spec_is_satisfied`. This does not
  verify the blocking/liveness behavior of the runtime spin-wait loop. The new comments only
  document the gap; they do not introduce an environment/fairness model or proof of eventual
  satisfaction.
  **Suggested Fix:** Add an explicit progress/environment model or a trusted lemma that captures
  eventual signaling, and prove `wait` terminates when enough signals occur.

- **Location:** `Fence::signal` precondition and `Fence::wf` (exec/spec,
  `verus/split/kernel/pm/sync/fence.rs`, `verus/split/kernel/pm/sync/fence.spec.rs`).
  **Description:** The model still enforces `count <= total` and requires `spec_is_waiting` before
  `signal`, forbidding over-signaling. The runtime allows `signal` at any time and permits
  `count > total`, so the verified contract remains stronger than runtime semantics. The new
  documentation labels this as deliberate, but it is still a mismatch.
  **Suggested Fix:** Weaken the spec/precondition to allow over-signaling (bounded only by
  overflow), or modify the runtime to enforce the same protocol.

### Medium
- **Location:** Fence model/proofs (exec/proof, `verus/split/kernel/pm/sync/fence.rs`,
  `verus/split/kernel/pm/sync/fence.proof.rs`).
  **Description:** Verification remains purely sequential (`&mut self` for `signal`) and does not
  model atomicity or memory ordering, so correctness under concurrent interleavings is still
  unproven. Added commentary does not change the proof obligations.
  **Suggested Fix:** Use Verus atomic modeling or a concurrency protocol (rely/guarantee, ghost
  invariants) to connect to the atomic runtime behavior.

### Low
- None.

## Positive Observations
- Added documentation clearly describes verification scope and trust boundaries.
- Proof lemmas remain consistent with the specified sequential protocol.

## Summary
The prior high/medium issues were not fixed; they were documented but the model still omits
concurrency, liveness of `wait`, and allows only protocol-compliant signaling. Verification
therefore remains incomplete with respect to the runtime fence semantics.
