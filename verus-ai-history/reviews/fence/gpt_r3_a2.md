// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

# Review: fence (gpt-5.2-codex)

## Grade: D

## Issues Found

### Critical
- **Location:** `Fence::wait` (exec `verus/split/kernel/pm/sync/fence.rs`).
  **Description:** `wait()` is still modeled as a no-op with a precondition that the fence is already
  satisfied. This does not verify the core runtime behavior (blocking until concurrent signals
  advance the count), so the fundamental correctness/liveness property of `wait()` remains unproved.
  **Status:** Not fixed.

### High
- **Location:** `Fence::signal` and `Fence::wf` (exec/spec `verus/split/kernel/pm/sync/fence.rs`,
  `verus/split/kernel/pm/sync/fence.spec.rs`).
  **Description:** The model still enforces `count < total` and invariant `count <= total`, which
  forbids over-signaling. The runtime allows `count > total` and a known caller over-signals by one,
  so the verification remains strictly stronger than the real behavior. **Status:** Not fixed.
- **Location:** `Fence` modeling and `signal()` signature (exec `verus/split/kernel/pm/sync/fence.rs`).
  **Description:** The model still replaces `AtomicUsize` with a plain `usize` and uses `&mut self`,
  which cannot represent concurrent signals or Acquire/Release ordering. Concurrency safety remains
  unverified. **Status:** Not fixed.

### Medium
- **Location:** Protocol proofs (proof `verus/split/kernel/pm/sync/fence.proof.rs`).
  **Description:** Proofs remain arithmetic lemmas that do not connect to an execution model of the
  wait loop or interleavings with `signal()`. End-to-end properties like “after N signals, wait
  returns” are still not established for the exec model. **Status:** Not fixed.

### Low
- _None._

## Positive Observations
- Documentation clearly describes the sequential model and its limitations.
- Spec predicates are consistent and use `wf()` to relate satisfaction/waiting states.

## Summary
The prior critical and high issues remain; the changes add documentation but do not address the
semantic mismatch with runtime concurrency, over-signaling, or the missing wait-loop proof. The
verification is still incomplete and does not establish soundness for the real fence behavior.
