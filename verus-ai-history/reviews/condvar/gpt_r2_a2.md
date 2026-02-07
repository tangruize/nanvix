# Review: condvar (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **API coverage still incomplete (documented but not verified).**
  - **Location:** `condvar.rs` (exec), API mapping and scope notes (lines 54-116, 73-86).
  - **Why not fixed:** The update documents that only queue operations are modeled and explicitly leaves `reference_count()` out of scope, and maps `wait()` to `enqueue()` only (no verified full wait protocol or wrappers). This is a documentation change, not a proof of the missing APIs. The verified module still does not provide exec-level wrappers or refinement proofs for `reference_count()`, full `wait()`, or runtime `notify_*()` error semantics.
  - **Evidence:** API mapping explicitly says `wait()` is modeled only as `enqueue` and `reference_count()` is “not modeled” (lines 75-86), and the scope section says error handling and ProcessManager interactions are out of scope (lines 54-65).

- **Uniqueness invariant remains a trust assumption, not enforced by runtime.**
  - **Location:** `condvar.spec.rs` `wf()` includes `spec_all_unique()` (lines 38-41) and `condvar.rs` trust assumption T1 (lines 130-139), `enqueue` precondition (lines 239-244).
  - **Why not fixed:** The model still requires `enqueue` to reject duplicates, but the runtime code does not check for duplicates. The update adds a justification that the protocol prevents duplicates (T1), but this is not proven or enforced—soundness still depends on an external protocol assumption.

- **Concurrency/interleaving still unmodeled; sequential wait protocol assumption remains.**
  - **Location:** `condvar.rs` trust assumption T4 (lines 152-160) and proof notes (lines 28-31, 647-655 in `condvar.proof.rs`).
  - **Why not fixed:** The proof still assumes no concurrent queue modifications between enqueue and cleanup while `sleep()` runs. This is explicitly documented as a limitation but still leaves the actual interleavings of `notify_*()` unverified.

### Medium
- **Notify error semantics still not modeled.**
  - **Location:** `condvar.rs` scope and API divergence (lines 62-64, 105-115).
  - **Why not fixed:** The model still returns `bool`/`usize` and does not represent `Result` error paths or success-count semantics from `notify_first()`/`notify_all()`. The update explicitly marks these as out of scope rather than implementing them.

- **`wait()` behavioral checks still missing.**
  - **Location:** `condvar.rs` scope and API mapping (lines 78-80, 121-126).
  - **Why not fixed:** `wait()` is still modeled solely as queue insertion. Alarm expiration, kernel-process checks, and `ProcessManager::sleep()` error handling are not captured.

- **Search correctness still assumed via ghost inputs.**
  - **Location:** `try_remove_by_pid` / `try_remove_by_tid` preconditions (lines 460-541).
  - **Why not fixed:** The model still requires callers to provide `has_match` and a ghost index consistent with `spec_contains_*`. This assumes the runtime search result without modeling or proving `position()` correctness.

### Low
- **Queue length bound remains a trust assumption.**
  - **Location:** `condvar.rs` trust assumption T3 (lines 145-151) and `enqueue` precondition (lines 239-244).
  - **Why not fixed:** The update only documents the assumed system-level bound. There is no proof or runtime guard enforcing `len < usize::MAX`.

## Positive Observations
- The documentation now clearly states verification scope, API mapping, and trust assumptions (including T1–T6), which makes the model’s intended guarantees explicit (`condvar.rs`, lines 36-171).
- The updated API mapping accurately reflects the model’s intent and highlights divergence from runtime behavior (lines 75-115).
- Proof lemmas for uniqueness and wait-cleanup are now clearly grouped and documented, improving auditability (`condvar.proof.rs`, lines 274-742).

## Summary
The update primarily adds documentation and trust-assumption framing, but does not actually close the original verification gaps. Key behavioral mismatches remain: full API coverage (including `reference_count()` and `wait()`), concurrency interleavings, and error semantics are still unmodeled. Verification is therefore not complete for the runtime condvar API; it remains a sequential queue model with explicit assumptions.
