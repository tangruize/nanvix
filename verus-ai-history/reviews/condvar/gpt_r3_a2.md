# Review: condvar (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Coverage gap still present:** `reference_count()` and `Drop::drop`/`fmt::Debug` remain unmodeled in exec/spec/proof. The updated docs explicitly mark them out-of-scope, but there is still no verified counterpart or spec capturing their behavior, so the API surface is still partially unverified.
- **Concurrency gap in wait protocol remains:** The proof still assumes no interleavings between enqueue and cleanup. The new T4 trust assumption only documents this limitation; there is still no proof of correctness under concurrent notify/wait interleavings.

### Medium
- **Notify semantics still weakened:** `notify_first`, `notify_process`, `notify_thread`, and `notify_all` continue to be modeled as pure queue removals, with no specification of `ProcessManager::wakeup()` outcomes or `Result`-level error behavior. The new `spec_notify_all_result` bounds are helpful but do not model partial-failure semantics.
- **Wait error behavior still unspecified:** `wait(alarm)` is still reduced to `try_enqueue(..., alarm_expired) -> bool` without any `SleepError` or error path specification, so the original error handling remains unverified.
- **Drop discipline remains a global assumption:** New lemmas show empty queues are drop-safe, but there is still no proof that actual drop sites satisfy `spec_drop_safe()`; the panic-on-drop safety condition remains unverified at call sites.

### Low
- None.

## Positive Observations
- The kernel-pid invariant is now explicitly part of `wf()` via `spec_no_kernel_pid()`, addressing the prior missing global invariant.
- Added lemmas around drop safety (`lemma_empty_is_drop_safe`) and explicit trust assumptions improve clarity.
- The queue protocol proofs remain well-structured and comprehensive for sequential state transitions.

## Summary
The update fixes the missing kernel-pid invariant, but the major verification gaps remain: unmodeled API surface (reference_count/drop/debug), non-concurrent wait protocol reasoning, and weakened error semantics. Documentation improvements do not resolve these correctness gaps. Verification is still incomplete for full API- and concurrency-accurate behavior.
