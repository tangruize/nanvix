# Review: fence (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `Fence::wait` (exec, `verus/split/kernel/pm/sync/fence.rs`).
  **Description:** The verified `wait` is a no-op with a precondition that the fence is already
  satisfied (`spec_is_satisfied`). This eliminates the core behavior of blocking until concurrent
  `signal` calls occur and does not prove the fundamental correctness/liveness property of the
  original spin-wait loop.
  **Suggested Fix:** Model `wait` with an explicit environment/progress assumption (e.g., ghost
  state or an atomic model with fairness) and prove that the loop terminates once enough signals
  occur. Alternatively, isolate the concurrency assumption in a clearly marked trusted lemma with
  a precise contract that matches the runtime behavior.

- **Location:** `Fence::signal` precondition and `Fence::wf` (exec/spec, `fence.rs` and `fence.spec.rs`).
  **Description:** The spec enforces `count <= total` and requires `spec_is_waiting` before
  `signal`, which forbids over-signaling. The original implementation allows `signal` at any time
  and permits `count > total`, so the verified API contract is stronger than runtime behavior and
  not semantically equivalent.
  **Suggested Fix:** Weaken the invariant and precondition to match the runtime: allow `count`
  to exceed `total`, and specify `signal` as `count := count + 1` with only a wrapping/overflow
  condition consistent with `AtomicUsize::fetch_add` (or add a runtime check in the original if
  the stronger contract is intended).

### Medium
- **Location:** `Fence` model and proofs (exec/proof, `fence.rs`, `fence.proof.rs`).
  **Description:** The verified model is purely sequential (`&mut self` for `signal`) and does
  not model atomicity or Acquire/Release ordering. As a result, the verification does not establish
  correctness under concurrent interleavings, which is the primary use case for the fence.
  **Suggested Fix:** Use Verus atomic modeling (`vstd::atomic*`) or a concurrency-aware protocol
  (e.g., rely/guarantee or ghost invariants) to capture concurrent signals and the ordering
  guarantees required by the spin-wait loop.

### Low
- None.

## Positive Observations
- All original functions (`new`, `wait`, `signal`) are present with verified counterparts.
- Spec/proof/exec separation is clean and documentation clearly states scope and trust boundaries.
- Arithmetic properties (remaining signals, monotonicity) are captured with clear lemmas.

## Summary
The verification cleanly models a sequential protocol, but it does not capture the core blocking
behavior of `wait` or the permissive signaling semantics of the runtime. Strengthening the model
with a concurrency-aware atomic specification (or explicitly justifying trusted assumptions) would
bring it closer to the essential correctness properties expected for this kernel synchronization
primitive.
