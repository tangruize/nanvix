# Review: runnable (gemini-3-pro-preview)

## Grade: A

## Issues Found

### Medium
- **Function:** `RunnableProcess::wf` (spec)
- **Description:** The well-formedness predicate (`wf`) does not enforce pairwise disjointness of thread IDs across the four thread lists (ready, interrupted, sleeping, zombie). While the original Rust type system enforces this via ownership, the verification model uses `Seq<int>` which allows duplicates or overlaps. This means a verified `RunnableProcess` could theoretically represent an invalid state (e.g., a thread being both ready and sleeping), relying entirely on the "trust assumption" that callers provide disjoint lists.
- **Suggested Fix:** Incorporate the `spec_ids_disjoint` predicate into `wf()`. Since these are ghost fields, the performance cost is negligible for verification (it increases proof burden slightly but improves model soundness). At minimum, `from_state` and `new` should require `spec_ids_disjoint`.

### Low
- **Function:** `EXIT_STATUS_INTERRUPTED` (spec)
- **Description:** The exit status for interrupted processes is hardcoded to `4` (EINTR). If the definition of `ErrorCode::Interrupted` changes in the codebase, this verification spec will become incorrect without warning.
- **Suggested Fix:** As noted in the TODO, add a cross-module assertion or CI check to validate that `EXIT_STATUS_INTERRUPTED() == ErrorCode::Interrupted as int`.

- **Function:** `find_thread` / `find_thread_mut` (exec)
- **Description:** These functions are omitted from the verified executable code because they return complex reference types (`ThreadRef`) that are difficult to model in Verus. This leaves the burden of verifying these lookups to the callers.
- **Suggested Fix:** No immediate fix required as this is a known limitation. Future work could introduce `external_body` wrappers or ghost-returning versions to assist callers in verifying the lookup logic.

## Positive Observations
- **Oracle Removal:** The use of explicit execution-level counters (`interrupted_count`, `sleeping_count`) tied to ghost sequence lengths via `wf()` is an excellent pattern. It allows `terminate()` and `wakeup()` to be verified without requiring oracle parameters for branch decisions, making the code much closer to the original executable.
- **Fairness Proof:** The `run()` function's specification and proof correctly capture the scheduling policy (earliest admission time) using `spec_earliest_ready_index` and `lemma_earliest_ready_index_bounds`. The proof that a minimum exists in a non-empty sequence is rigorous.
- **Isomorphism:** The modeling of `NonEmptyVecDeque` as `Seq<int>` with `len() >= 1` enforced by `wf()` is sound and clean.
- **Separation of Concerns:** The split between `runnable.rs` (exec), `runnable.spec.rs` (definitions), and `runnable.proof.rs` (lemmas) is clean and follows best practices.

## Summary
The verification of `RunnableProcess` is high-quality and complete. It effectively models the complex state transitions of a runnable process while abstracting away lower-level details (HAL, Context). The soundness of the verification is strong, with the only notable gap being the reliance on Rust ownership for thread ID uniqueness, which is a reasonable trade-off but could be strengthened. The removal of oracles via ghost-counter coupling is particularly praiseworthy.
