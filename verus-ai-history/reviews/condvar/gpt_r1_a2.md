# Review: condvar (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `Condvar` API coverage (exec) — `verus/split/kernel/pm/sync/condvar.rs`.
  **Description:** The model still omits verified wrappers that correspond to the public `wait`, `notify_*`, and `reference_count` APIs; it only exposes queue operations. The new documentation explicitly says these behaviors are out of scope, but no wrapper or refinement lemma is added to connect observable return/error behavior to the model, so the coverage gap remains.
  **Evidence:** The API mapping table still lists `reference_count()` as “not modeled” and maps `wait()`/`notify_*()` to queue-only operations without a verified wrapper that encodes return types or error paths (see “API Mapping” section).

### Medium
- **Location:** `wait()` semantics — `verus/split/kernel/pm/sync/condvar.rs`.
  **Description:** The model still does not encode the control-flow/cleanup behavior of the original `wait()` (alarm check, sleep failure cleanup, and panic paths). This is explicitly documented as out of scope but remains an unverifed behavioral gap.
  **Evidence:** The verification scope explicitly excludes alarm/timer handling and `ProcessManager::sleep()` interaction, and there is no `wait`-style wrapper or lemma that sequences enqueue + cleanup.

- **Location:** `notify_first` / `notify_all` return semantics — `verus/split/kernel/pm/sync/condvar.rs`.
  **Description:** The model still returns `bool`/`usize` and does not model the original `Result`-based error behavior (including partial failures in `notify_all`). This weakens reasoning about observable behavior.
  **Evidence:** The “Verification Scope” and “API Divergence” sections state error handling is out of scope and no refinement lemma connects queue state to `Result` behavior.

- **Location:** Drop discipline — `verus/split/kernel/pm/sync/condvar.rs`.
  **Description:** The requirement that the queue is empty on drop remains a trust assumption (T2) with no enforced predicate or proof obligations.
  **Evidence:** The trust assumptions section still documents drop discipline as unverified and there is no `drop_ok`-style predicate or proof.

### Low
- None.

## Positive Observations
- **Fixed:** `wf()` now includes `spec_all_unique()` in `condvar.spec.rs`, and the exec methods preserve it, addressing the prior uniqueness-invariant disconnect.
- **Fixed:** `try_remove_by_pid` / `try_remove_by_tid` now model the not-found case with state-preserving behavior, addressing the previous over-strong precondition for notify-by-id operations.
- The queue-operation proofs remain consistent and are expanded with uniqueness-preservation lemmas.

## Summary
Two of the prior high issues (uniqueness in `wf()` and not-found removal cases) are genuinely fixed, but the core coverage gap remains: there are still no verified wrappers or refinement lemmas that capture the public `wait`/`notify_*`/`reference_count` behaviors or their return/error semantics. The verification is therefore still incomplete with respect to observable API behavior and drop safety, despite improved internal invariants.
