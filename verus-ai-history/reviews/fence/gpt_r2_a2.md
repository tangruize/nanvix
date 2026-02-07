# Review: fence (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `wait` (exec: `verus/split/kernel/pm/sync/fence.rs:159-183`).
  **Description:** `wait()` is still modeled as a no-op with the precondition `spec_is_satisfied()`. This does not capture the runtime's blocking behavior or the liveness guarantee that the fence eventually becomes satisfied due to concurrent signals. The issue from the previous review remains unfixed; the update only documents the gap.

- **Location:** `signal` contract vs. runtime usage (exec: `verus/split/kernel/pm/sync/fence.rs:185-210`, spec: `verus/split/kernel/pm/sync/fence.spec.rs:31-43`, runtime: `src/kernel/src/kmain.rs:345-349` and `src/kernel/src/kmain.rs:487-490`).
  **Description:** The model still forbids over-signaling (`spec_is_waiting()` / `count < total`), while the runtime `signal()` has no guard. The new documentation claims callers never oversignal, but the startup fence initializes with `startup::init(ncores - 1)` and each application core calls `startup::signal()`, which can oversignal by 1 when multiple cores are present. This is concrete evidence the strengthened precondition is not justified by current runtime usage.

### Medium
- **Location:** Concurrency model (exec: `verus/split/kernel/pm/sync/fence.rs:20-55`).
  **Description:** The proof still uses a sequential `&mut self` model and does not establish atomicity or memory ordering correctness for concurrent `signal()`/`wait()`. The text now acknowledges this limitation, but the core verification gap remains unresolved.

### Low
- **Location:** Documentation claim about caller audit (exec docstring: `verus/split/kernel/pm/sync/fence.rs:64-68`).
  **Description:** The statement that callers signal exactly once and never oversignal is not supported by the current runtime code path in `kmain.rs` (see the high-severity issue above). This is misleading documentation unless the runtime logic is corrected.

## Positive Observations
- The updated documentation makes the trust boundaries explicit and clearer.
- The proof module now includes additional spec-level lemmas that improve readability.

## Summary
The changes add useful documentation but do not fix the substantive issues: `wait()` still does not model blocking/liveness, the `signal()` precondition still diverges from runtime behavior (with evidence of actual oversignaling), and concurrency correctness remains unverified. Verification is therefore not complete or sound with respect to the runtime fence behavior.
