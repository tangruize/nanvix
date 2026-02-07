# Review: condvar (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `Condvar` API coverage (exec) — `verus/split/kernel/pm/sync/condvar.rs`.
  **Description:** The verified model omits `reference_count()` entirely and does not provide a verified `wait()` or `notify_*()` that mirrors the original return types and error paths. The model only exposes queue operations (`enqueue`, `dequeue_first`, `remove_*`, `clear`) which leaves major parts of the public API uncovered, violating the coverage requirement and weakening equivalence.
  **Suggested Fix:** Add verified wrappers that correspond 1:1 with `wait`, `notify_first`, `notify_process`, `notify_thread`, `notify_all`, and `reference_count` (even if some are `external_body` with explicit specs), and prove/refine their queue effects and return/error behavior.

- **Location:** `remove_by_pid` / `remove_by_tid` (exec) — `verus/split/kernel/pm/sync/condvar.rs`.
  **Description:** The model requires a ghost index for a *matching* entry and therefore cannot represent the valid “not found” case where the original implementation performs a no-op and returns `Ok(())`. This makes the spec too strong and not semantically equivalent for callers that may have no matching entry.
  **Suggested Fix:** Add a wrapper that takes an `Option<Ghost<int>>` (or returns a bool) to model both “found” and “not found” cases, with postconditions that preserve state when no match exists.

- **Location:** Uniqueness invariant (spec/proof) — `condvar.spec.rs` / `condvar.proof.rs`.
  **Description:** Uniqueness (`spec_all_unique`) is documented as a trust assumption but is not part of `wf()` and is not required/ensured by the exec methods. As a result, proofs that rely on uniqueness (e.g., removal implies absence) are not applicable to the executable API, leaving an important safety property unconnected to the actual operations.
  **Suggested Fix:** Strengthen `wf()` to include uniqueness or add `requires/ensures` on mutators to preserve `spec_all_unique`, then use those in higher-level specs.

### Medium
- **Location:** `wait()` semantics (original) vs. model — `src/kernel/src/pm/sync/condvar.rs` vs `verus/split/kernel/pm/sync/condvar.rs`.
  **Description:** The model does not capture critical control-flow behavior: kernel-process panic, alarm-expired early error, and the guarantee that the entry is removed from the queue when `ProcessManager::sleep()` fails. These are essential for correctness and resource safety but are not specified as a single verified operation.
  **Suggested Fix:** Add a verified `wait_model` that encodes the branch structure (alarm check, enqueue, sleep, cleanup on error) and connect it to `enqueue/remove_entry` with clear pre/postconditions.

- **Location:** `notify_first` / `notify_all` return semantics (exec) — `verus/split/kernel/pm/sync/condvar.rs`.
  **Description:** The model only returns a bool/usize and ignores the original `Result`-based error behavior and partial-failure semantics (e.g., `notify_all` returns `Err` only when no wakeups succeeded). This makes the spec too weak to reason about observable behavior.
  **Suggested Fix:** Specify and verify the return-value/error behavior (possibly via an abstract error model), or explicitly add a refinement lemma that states the model only captures queue effects and does not verify error semantics.

- **Location:** Drop discipline (spec doc) — `verus/split/kernel/pm/sync/condvar.rs`.
  **Description:** The requirement that the queue is empty on drop is only a documented trust assumption (T2) and is not enforced or proved. This is a safety property of the original runtime behavior.
  **Suggested Fix:** Add a spec-level obligation (e.g., a `drop_ok` predicate) and prove it for all public operations, or provide a justification for why it is enforced elsewhere.

### Low
- None.

## Positive Observations
- Clear spec/proof separation with `condvar.spec.rs` and `condvar.proof.rs`, and no use of `assume`/`external_body` in the core model.
- Strong FIFO and structural queue properties are proved (enqueue/dequeue ordering, remove_at correctness, and clear semantics).
- The documentation explicitly spells out modeling scope, trust assumptions, and the mapping between original APIs and the verified model.

## Summary
The verification provides a solid sequential queue model, but it does not cover major parts of the public API or observable error/timeout behaviors, making the equivalence and coverage incomplete for an OS condvar. Strengthening the model with verified wrappers for `wait`/`notify_*`, incorporating the not-found cases, and tying uniqueness and drop discipline into invariants would substantially improve correctness coverage.
