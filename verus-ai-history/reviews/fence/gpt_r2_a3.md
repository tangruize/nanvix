# Review: fence (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `wait` (exec: `verus/split/kernel/pm/sync/fence.rs:161-185`).
  **Description:** `wait()` is still a no-op with precondition `spec_is_satisfied()`. The core blocking/liveness behavior of the runtime fence remains unmodeled; this issue is only documented, not fixed.

- **Location:** `signal` contract vs. runtime usage (exec: `verus/split/kernel/pm/sync/fence.rs:187-212`, spec: `verus/split/kernel/pm/sync/fence.spec.rs:31-43`, runtime: `src/kernel/src/kmain.rs:345-349` and `src/kernel/src/kmain.rs:487-490`).
  **Description:** The spec still forbids over-signaling (`count < total` / `wf()`), but runtime `signal()` has no guard. The new documentation explicitly acknowledges that startup oversignals by one; this confirms the mismatch rather than fixing it. Verification therefore remains unsound with respect to runtime behavior.

### Medium
- **Location:** Concurrency model (exec: `verus/split/kernel/pm/sync/fence.rs:20-55`).
  **Description:** Still a sequential `&mut self` model with no atomicity/memory-ordering proof. Acknowledging the gap does not provide concurrency correctness.

### Low
- None. (The previous misleading documentation about callers never oversignaling is corrected.)

## Positive Observations
- Documentation now explicitly states the startup oversignal and the intentional divergence from runtime behavior.
- Trust boundaries are clearer and more honest about what is and is not verified.

## Summary
The update improves documentation but does not resolve the main correctness gaps: `wait()` liveness is still unmodeled, the `signal()` precondition still contradicts runtime usage (and now explicitly admits oversignal), and concurrency correctness is still unverified. Verification remains incomplete and unsound relative to runtime fence behavior.
