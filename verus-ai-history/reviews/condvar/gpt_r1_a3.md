# Review: condvar (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `Condvar` API coverage (exec) — `verus/split/kernel/pm/sync/condvar.rs`.
  **Description:** The model still does not provide verified wrappers that correspond to the public `wait`, `notify_*`, or `reference_count` APIs. The new lemmas document parts of the protocol, but there is still no exec-level API (or refinement lemma) that captures observable return/error behavior for these methods.
  **Evidence:** The API mapping still lists `reference_count()` as “not modeled” and maps `wait()`/`notify_*()` to queue-only operations, with only a proof lemma for the wait cleanup path.

### Medium
- **Location:** `wait()` semantics — `verus/split/kernel/pm/sync/condvar.rs` / `condvar.proof.rs`.
  **Description:** The new wait-cleanup lemmas only cover the enqueue + retain cleanup path. The alarm-expired branch, panic path, and `sleep()` error handling remain unmodeled, so the verified behavior is still incomplete relative to the original control flow.
  **Evidence:** Verification scope still excludes alarm/timer handling and `ProcessManager::sleep()` effects; no wrapper or lemma encodes those branches.

- **Location:** `notify_first` / `notify_all` return semantics — `verus/split/kernel/pm/sync/condvar.rs`.
  **Description:** The model continues to return `bool`/`usize` and does not capture the `Result`-based error behavior or partial-failure semantics, so observable API behavior is not verified.
  **Evidence:** The verification scope and API divergence sections explicitly exclude error handling and keep queue-only return types.

- **Location:** Drop discipline — `verus/split/kernel/pm/sync/condvar.rs` / `condvar.spec.rs`.
  **Description:** The new `spec_drop_safe()` predicate and lemmas are a good formalization, but the code still does not enforce or preserve drop safety across all operations; it remains a trust assumption rather than a verified obligation at call sites.
  **Evidence:** `spec_drop_safe()` is defined and proved for `new()`/`clear()`, but no operation requires or ensures drop safety before drop, and the trust assumptions section notes whole-program reasoning is needed.

### Low
- None.

## Positive Observations
- **Fixed:** The wait-cleanup path is now captured by `lemma_wait_cleanup_restores_state` and `lemma_wait_protocol_preserves_wf`, so the specific missing cleanup lemma is addressed.
- **Improved:** Drop safety is now formalized with `spec_drop_safe()` and supported by basic lemmas (`new`/`clear`), which is an improvement over purely narrative assumptions.
- Uniqueness preservation and queue-order proofs remain consistent.

## Summary
The prover added targeted wait-cleanup and drop-safety formalizations, which partially address earlier concerns, but the main verification gap persists: public API behavior (`wait`/`notify_*`/`reference_count`) and error semantics remain unverified. Verification is still incomplete and should not be considered fully sound with respect to observable condvar behavior.
