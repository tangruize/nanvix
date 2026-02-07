# Review: fence (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- **Location:** `Fence::wait` (exec `verus/split/kernel/pm/sync/fence.rs`).
  **Description:** `wait()` is modeled as a no-op with a precondition that the fence is already satisfied. This does not capture the core runtime behavior (blocking until concurrent signalers advance `count`) and therefore does not verify the key safety/liveness contract of `wait()`.
  **Suggested Fix:** Model `wait()` as a loop that is justified by a concurrent/atomic model (e.g., Verus atomics or a rely/guarantee framework), or introduce an explicit ghost/abstract progress assumption and prove that `wait()` returns once `signal()` has been invoked `total` times.

### High
- **Location:** `Fence::signal` and `Fence::wf` (exec/spec `verus/split/kernel/pm/sync/fence.rs` and `fence.spec.rs`).
  **Description:** The verified model enforces `count < total` precondition and invariant `count <= total`, which forbids over-signaling. The runtime implementation allows `count` to exceed `total` (and a known caller over-signals by one), so the verification is strictly stronger than the real behavior and not semantically equivalent.
  **Suggested Fix:** Either (a) relax `wf()` and `signal()` preconditions to allow `count > total` (while still proving `wait()` correctness), or (b) update the runtime protocol to forbid over-signaling and enforce the precondition at call sites.
- **Location:** `Fence` modeling and `signal()` signature (exec `verus/split/kernel/pm/sync/fence.rs`).
  **Description:** The model replaces `AtomicUsize` with a plain `usize` and uses `&mut self`, which cannot represent concurrent `signal()` calls or the Acquire/Release ordering in the original. Concurrency safety (atomicity, non-tearing, and interleavings) is therefore unverified.
  **Suggested Fix:** Use Verus atomic modeling (`vstd::atomic*`) or a concurrent state machine model to represent multiple `signal()` callers and the memory-ordering contract.

### Medium
- **Location:** Protocol properties in `fence.proof.rs`.
  **Description:** The proofs are primarily arithmetic lemmas and do not connect to an execution model that represents the spin loop or interleaving with `signal()`. As a result, key end-to-end properties (e.g., “after N signals, `wait()` returns”) are not established for the exec model.
  **Suggested Fix:** Add a proof that links the exec model to the protocol lemmas (e.g., an inductive invariant over a modeled loop or a rely/guarantee argument for concurrent progress).

### Low
- _None._

## Positive Observations
- All original functions (`new`, `wait`, `signal`) have verified counterparts, and the split between exec/spec/proof is clean.
- The specification defines clear predicates (`wf`, `spec_is_satisfied`, `spec_is_waiting`) and documents trust boundaries explicitly.
- Proof files provide useful arithmetic lemmas and monotonicity facts that can serve as a base for richer concurrent proofs.

## Summary
The verification provides a well-documented sequential protocol model but does not capture the essential concurrent waiting behavior or the runtime-allowed over-signaling, so semantic equivalence is not achieved. Strengthening the model to handle concurrency (or aligning runtime behavior with the stricter spec) is required to make the verification reflect the real fence semantics.
