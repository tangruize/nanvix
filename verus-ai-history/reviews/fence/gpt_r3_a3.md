// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

# Review: fence (gpt-5.2-codex)

## Grade: D

## Issues Found

### Critical
- **Location:** `Fence::wait` (exec `verus/split/kernel/pm/sync/fence.rs`).
  **Description:** `wait()` is still a no-op with a precondition that the fence is already satisfied.
  This fails to verify the actual runtime behavior (blocking until concurrent signalers advance the
  count), so the key safety/liveness contract of `wait()` remains unproved. **Status:** Not fixed.

### High
- **Location:** `Fence::signal` and `Fence::wf` (exec/spec `verus/split/kernel/pm/sync/fence.rs`,
  `verus/split/kernel/pm/sync/fence.spec.rs`).
  **Description:** The model still enforces `count < total` and `count <= total`, forbidding
  over-signaling. The runtime allows `count > total` and a known caller over-signals by one, so the
  verification remains stronger than the actual behavior. **Status:** Not fixed.
- **Location:** `Fence` modeling and `signal()` signature (exec `verus/split/kernel/pm/sync/fence.rs`).
  **Description:** The model continues to replace `AtomicUsize` with `usize` and uses `&mut self`,
  which cannot represent concurrent signals or Acquire/Release ordering. Concurrency safety is still
  unverified. **Status:** Not fixed.

### Medium
- **Location:** Protocol proofs (proof `verus/split/kernel/pm/sync/fence.proof.rs`).
  **Description:** Proofs remain arithmetic lemmas and do not connect to an execution model of the
  wait loop or interleavings with `signal()`. End-to-end properties like “after N signals, wait
  returns” are still not established for the exec model. **Status:** Not fixed.

### Low
- _None._

## Positive Observations
- The documentation accurately states the sequential model limitations.
- Spec predicates remain consistent and well-structured.

## Summary
No substantive changes address the previous issues; the verification is still incomplete and does
not establish soundness for the real concurrent fence behavior.
