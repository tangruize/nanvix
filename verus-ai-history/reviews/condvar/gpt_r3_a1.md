# Review: condvar (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Coverage gap:** `Condvar::reference_count()` and `CondvarInner::drop`/`fmt::Debug` are not modeled or specified in the verified module (exec/spec/proof). This fails the "all functions have verified versions" criterion; reference-count behavior and drop panic condition are unverified.
- **Concurrency gap in wait protocol:** `lemma_wait_cleanup_restores_state` and `lemma_wait_protocol_preserves_wf` assume no interleavings between enqueue and cleanup, but the original `wait()` sleeps and other threads may call `notify_*()` concurrently. The sequential model does not prove queue correctness under realistic interleavings, so equivalence for the wait/notify protocol is not established (proof).

### Medium
- **Weakened notify semantics:** `notify_first`, `notify_process`, `notify_thread`, and `notify_all` are modeled as pure queue removals (`dequeue_first`, `try_remove_by_*`, `clear`) and ignore `ProcessManager::wakeup()` failures and `Result`/error behavior (exec/spec). This omits the original's partial-failure semantics (e.g., `notify_all()` returning `Err` when zero successes), so specs are too weak for API-level correctness.
- **Wait error behavior not specified:** original `wait(alarm)` returns `SleepError` on alarm-expired or sleep failure and logs details; the model reduces this to `try_enqueue(..., alarm_expired)` returning `bool` without error specs (exec/spec). Intended error semantics are unverified.
- **Drop discipline only assumed:** `spec_drop_safe()` is defined and proved for `new()`/`clear()`, but there is no proof that all drop sites satisfy it (spec/proof). The panic-on-drop safety property remains unverified.

### Low
- **Kernel PID invariant not explicit:** `wf()` does not encode "no kernel pid in queue"; it is only a precondition of `enqueue` (spec/exec). This is likely true by protocol but not stated as a global invariant for downstream proofs.

## Positive Observations
- Thorough FIFO and uniqueness proofs; well-formedness is preserved by all queue operations.
- Clear API mapping with explicit trust boundaries and assumptions, including documentation of scope limits.
- No `assume` or `external_body` usage in the core module; soundness is maintained within stated scope.
- Clean split between exec/spec/proof files with readable, structured lemmas.

## Summary
The verification is strong for sequential queue-state correctness but does not fully cover API surface area or the concurrent wait/notify semantics. Strengthen coverage of missing functions and refine specs for error behavior and interleavings to make the proof match kernel behavior more closely.
