# Review: condvar (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **API coverage still incomplete (documented but not verified).**
  - **Location:** `condvar.rs` API mapping and scope notes (lines 54-116, 75-86).
  - **Status:** **Not fixed.** The model still exposes only queue operations (no exec wrappers for `reference_count()` or full `wait()`), and `notify_*()` semantics are documented as out of scope. This is still a documentation-only mitigation, not a refinement proof.
  - **Evidence:** API mapping explicitly states `reference_count()` is “not modeled,” `wait()` is only `enqueue`, and error handling is out of scope.

- **Uniqueness invariant remains a trust assumption, not enforced by runtime.**
  - **Location:** `condvar.spec.rs` `wf()` includes `spec_all_unique()` (lines 38-41) and `condvar.rs` T1 + `enqueue` precondition (lines 130-139, 239-244).
  - **Status:** **Not fixed.** The runtime still does not check for duplicates; the proof relies on a protocol assumption that a thread cannot wait twice. This is not enforced or proved, so reachable runtime states may violate `wf()`.

- **Concurrency/interleaving still unmodeled; sequential wait protocol assumption remains.**
  - **Location:** `condvar.rs` T4 (lines 152-160) and `condvar.proof.rs` wait protocol lemmas (lines 647-709).
  - **Status:** **Not fixed.** The model still assumes no concurrent `notify_*()` calls between enqueue and cleanup. The interleavings that occur while `sleep()` blocks are explicitly out of scope.

### Medium
- **Notify error semantics still not modeled.**
  - **Location:** `condvar.rs` scope and API divergence (lines 62-64, 105-115).
  - **Status:** **Not fixed.** The model continues to return `bool`/`usize` instead of `Result`, and does not capture wakeup error paths or success-count semantics for `notify_first()`/`notify_all()`.

- **`wait()` behavioral checks still missing.**
  - **Location:** `condvar.rs` scope and API mapping (lines 78-80, 121-126).
  - **Status:** **Not fixed.** Alarm expiration, kernel-process checks, and `ProcessManager::sleep()` failure handling are still not modeled.

- **Search correctness still assumed via ghost inputs.**
  - **Location:** `try_remove_by_pid` / `try_remove_by_tid` preconditions (lines 478-541).
  - **Status:** **Not fixed.** The caller must supply `has_match` and a matching ghost index consistent with `spec_contains_*`. This still assumes the runtime search (`position()`) is correct without modeling or proving it.

### Low
- **Queue length bound remains a trust assumption.**
  - **Location:** `condvar.rs` T3 and `enqueue` precondition (lines 145-151, 239-244).
  - **Status:** **Not fixed.** The model still assumes `len < usize::MAX` without a proof or runtime guard.

## Positive Observations
- The documentation continues to clearly delineate trust assumptions (T1–T6) and the verification scope, which helps avoid overstating guarantees.
- No new regressions or unsoundness were introduced in the spec/proof files; the model remains a consistent sequential queue abstraction.

## Summary
The updated files do not materially change the verification coverage: the same high-level gaps remain in API completeness, concurrency reasoning, error semantics, and search correctness. Most changes are documentation or scope framing rather than proofs or implementations. Verification is still incomplete for the runtime condvar API, and soundness still depends on multiple trust assumptions.
