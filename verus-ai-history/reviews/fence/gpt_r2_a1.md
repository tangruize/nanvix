# Review: fence (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `wait` (exec: `verus/split/kernel/pm/sync/fence.rs`).
  **Description:** The verified `wait()` is a no-op with the precondition `spec_is_satisfied()`, so it does not model the core blocking behavior of the runtime fence (spinning until signals arrive). This makes the spec too strong for callers and omits the key liveness property that `wait()` returns once enough `signal()`s occur.
  **Suggested Fix:** Model `wait()` with a loop over an atomic count (e.g., `vstd::atomic`) or add a rely/guarantee-style assumption/ghost state that captures eventual signaling, then prove that `wait()` terminates when the count reaches `total`.

- **Location:** `signal` (exec: `verus/split/kernel/pm/sync/fence.rs`, spec: `wf()` in `fence.spec.rs`).
  **Description:** `signal()` requires `count < total`, and `wf()` enforces `count <= total`, but the runtime `signal()` permits over-signaling with no guard. This strengthens the contract and makes the verified model non-equivalent if a caller signals more than `total` times (the runtime would continue, the model forbids it).
  **Suggested Fix:** Either (a) align the runtime with this contract by adding a guard/assert and documenting the precondition, or (b) weaken the spec to allow `count > total` and adjust invariants/proofs accordingly (e.g., `wf()` could be `true` and use `min(count,total)` for satisfaction).

### Medium
- **Location:** `signal`/`wait` concurrency model (exec: `verus/split/kernel/pm/sync/fence.rs`).
  **Description:** The sequential `&mut self` model does not capture concurrent `fetch_add`/`load` semantics or memory-ordering effects (`Acquire`/`Release`). As a result, the proof does not establish thread-safety or correctness under concurrent access—the primary use case for this fence.
  **Suggested Fix:** Use Verus atomic primitives (`vstd::atomic*`) or a concurrency proof pattern (e.g., atomic invariants) to model concurrent `signal()`/`wait()` and to justify memory-ordering requirements.

### Low
- None.

## Positive Observations
- All original API functions (`new`, `wait`, `signal`) are covered in the verified exec module, with clear documentation of scope and trust boundaries.
- Specs and proofs are cleanly separated, and key safety properties (well-formedness, satisfaction monotonicity, remaining signals) are stated and proven.
- The verification explicitly documents model limitations and intent, making the proof obligations transparent.

## Summary
The verification cleanly captures a sequential state-machine model and key safety invariants, but it omits the core concurrent blocking behavior and strengthens the `signal()` contract relative to the runtime implementation. To be an essential correctness proof for a fence, it should model concurrency (or explicitly capture rely/guarantee assumptions) and align the runtime contract with the verification preconditions.
