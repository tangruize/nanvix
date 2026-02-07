# Review: condvar (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Coverage gap still present:** `reference_count()` and `Drop::drop`/`fmt::Debug` are still not modeled in exec/spec/proof. The code and docs explicitly mark them as out-of-scope, but there is no verified counterpart or spec, so the API surface remains partially unverified.
- **Concurrency gap in wait protocol remains:** The proof still relies on the sequential assumption (T4) and does not establish correctness under real interleavings between enqueue and cleanup. This is only documented, not proved.

### Medium
- **Notify semantics still weakened:** `notify_first`, `notify_process`, `notify_thread`, and `notify_all` are still modeled as pure queue removals with no `ProcessManager::wakeup()` error semantics. `spec_notify_all_result` only constrains bounds and does not model partial-failure behavior.
- **Wait error behavior still unspecified:** `wait(alarm)` remains reduced to `try_enqueue(..., alarm_expired) -> bool`, with no `SleepError` or failure-mode specification, so error semantics are still unverified.
- **Drop discipline remains a global assumption:** New/previous drop-safety lemmas only show empty queues are drop-safe; there is still no proof that actual drop sites satisfy `spec_drop_safe()`.

### Low
- None.

## Positive Observations
- The kernel-pid invariant remains encoded in `wf()` via `spec_no_kernel_pid()` and preserved by operations.
- Queue protocol proofs are consistent and well-structured for the sequential model.

## Summary
No material changes addressing the previous gaps were found. The verification still does not cover `reference_count`/drop/debug behavior, does not model concurrency in wait/notify, and leaves error semantics of wait/notify unverified. Verification remains incomplete for full API-accurate behavior.
