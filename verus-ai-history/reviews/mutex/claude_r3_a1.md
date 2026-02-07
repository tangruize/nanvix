# Review: mutex (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

(none)

### High

- **H1: `lock()` precondition eliminates the interesting case**
  - **Location:** `lock()` in `mutex.rs` (exec, line 243–258)
  - **Description:** The original `lock()` is designed to be called on a *possibly-locked* mutex: it loops calling `try_lock()` and sleeps on a `Condvar` when the mutex is contended. The verified `lock()` requires `old(self).spec_is_unlocked() && !old(self).token_issued()`, which means the caller must already know the mutex is free. This reduces `lock()` to a trivially-successful `try_lock()`, eliminating verification of the retry/sleep loop that is the entire purpose of `lock()`. While the documentation openly acknowledges this limitation and the sequential model cannot fully capture it, the result is that the most safety-critical code path (contended locking) is unverified.
  - **Suggested Fix:** Document this as a known verification gap in a dedicated "Limitations" section. Consider a future iteration that models contention via a ghost sequence of events or a state-machine trace to at least prove that the loop invariant is sound assuming fair scheduling.

- **H2: `MutexToken` is publicly constructible (Trust Assumption T3)**
  - **Location:** `MutexToken` in `mutex.spec.rs` (spec, line 49–52)
  - **Description:** `MutexToken` is a `pub tracked struct` with a `pub ghost view` field. Any external module can construct `MutexToken { view: ... }` without going through `lock()`/`try_lock()`. This means the mutual exclusion guarantee (only one token per mutex) is an assumption, not a proven property. An external caller could forge a token and call `unlock()` on a mutex it never locked, or hold two tokens simultaneously. The module header documents this as Trust Assumption T3, but it weakens the formal guarantee significantly.
  - **Suggested Fix:** Investigate Verus's `proof fn` token-creation patterns or `sealed`/`mod`-private approaches to restrict token construction. If Verus limitations prevent this, add a dedicated proof lemma asserting the intended invariant with a clear `// TRUST:` comment, and track as a known limitation.

### Medium

- **M1: `reference_count()` not modeled**
  - **Location:** Original `Mutex::reference_count()` (line 120), not present in verified code
  - **Description:** The original exposes `Arc::strong_count()` which is used elsewhere to determine when a mutex can be safely destroyed. Not modeling this means the verification cannot reason about mutex lifetime safety. The API mapping table documents this omission.
  - **Suggested Fix:** If `reference_count()` is used in safety-critical paths (e.g., deciding when to deallocate), add a ghost refcount field and model increment/decrement. If it is only used for diagnostics, the omission is acceptable as-is.

- **M2: `try_lock()` error semantics diverge from original**
  - **Location:** `try_lock()` in `mutex.rs` (exec, line 203–228)
  - **Description:** The original returns `Result<MutexGuard, ()>` — a standard Rust Result. The verified version returns `(bool, Tracked<Option<MutexToken>>)`. While functionally equivalent in terms of state machine transitions, this changes the API contract: callers pattern-match differently, and the `Result` error-handling conventions (e.g., `?` operator) do not apply. This is a minor semantic divergence but could cause confusion when reasoning about code that uses `try_lock()` in error-handling chains.
  - **Suggested Fix:** Consider returning a ghost `Result`-like enum or documenting the mapping more prominently for reviewers comparing original and verified code.

- **M3: Condvar notification error path not modeled in `unlock()`**
  - **Location:** `unlock()` in `mutex.rs` (exec, line 277–294)
  - **Description:** The original `unlock_unchecked()` calls `self.sleeping.notify_first()` which returns `Result<_, Error>`. If notification fails, the original logs a warning. The verified `unlock()` has no error path, meaning a failure to wake a sleeping thread (which could cause a thread to remain blocked indefinitely) is not captured. This is a liveness concern.
  - **Suggested Fix:** Add a comment in `unlock()` noting that condvar notification failure is a liveness concern delegated to the condvar module's verification. If the condvar module does not verify liveness guarantees either, flag this as an end-to-end verification gap.

- **M4: `&mut self` vs `&self` semantic gap not formally bounded**
  - **Location:** All exec functions in `mutex.rs`
  - **Description:** The original uses `&self` with `AtomicBool` interior mutability to allow concurrent access. The verified model uses `&mut self`, which enforces exclusive access at the Rust type level. The documentation thoroughly explains this divergence, but there is no formal argument bounding the gap — i.e., no proof that "if the sequential model is correct AND the atomic operations are linearizable, THEN the concurrent implementation is correct." This is inherent to the verification approach but should be noted.
  - **Suggested Fix:** Add a brief "Refinement Argument" section to the module documentation sketching the informal argument for why sequential correctness implies concurrent correctness under linearizability of `compare_exchange`.

### Low

- **L1: `fmt::Debug for MutexGuard` not modeled**
  - **Location:** Original lines 189–198
  - **Description:** The `Debug` implementation is display-only with no state mutation, so not modeling it is correct. Documented in the API mapping table.
  - **Suggested Fix:** None needed.

- **L2: `Drop for MutexGuard` modeled as explicit `unlock()` — adequate but different mechanism**
  - **Location:** `unlock()` in `mutex.rs` (exec, line 277–294)
  - **Description:** The original uses RAII via `Drop` to guarantee unlock on scope exit. The verified model uses explicit token consumption. This is a standard verification pattern and is well-documented, but it shifts the burden of ensuring "unlock always happens" from the compiler (Drop) to the caller (must call `unlock()`). The module header explains this clearly.
  - **Suggested Fix:** None needed beyond the existing documentation. Consider adding a proof lemma that states "if a token exists and the mutex is well-formed, then unlock can be called" to strengthen the argument that the obligation is always dischargeable.

- **L3: Proof lemmas are mostly definitional unfoldings**
  - **Location:** `mutex.proof.rs` (proof, lines 17–141)
  - **Description:** The first 14 lemmas in the "Definitional Properties" section are trivially discharged by Verus — they unfold definitions and assert obvious consequences. While they serve as documentation and regression tests, they do not prove deep properties. The more interesting "Protocol Properties" section (lines 146–315) does prove meaningful round-trip, isolation, and contention lemmas.
  - **Suggested Fix:** Consider annotating the definitional lemmas with a comment like `// Regression test:` to distinguish them from the substantive protocol proofs, improving readability for reviewers.

## Positive Observations

- **Excellent documentation.** The module header is exceptionally thorough, with clear sections on verification model, scope, API mapping, API divergence, trust boundaries, and trust assumptions. This is among the best-documented verification modules I have reviewed.
- **Clean verification.** 27 verified, 0 errors, with no `assume`, `external_body`, or `trusted` annotations in any of the three files. The verification is fully self-contained.
- **Sound well-formedness invariant.** The `wf()` predicate (`locked == token_issued`) is a biconditional that precisely captures the reachable-state invariant. It is neither too weak (would allow invalid states) nor too strong (would reject valid states).
- **Token-based ownership model.** The `MutexToken` tracked struct cleanly models the `MutexGuard` RAII pattern. The token carries a view snapshot that binds it to a specific mutex instance, and `unlock()` requires an exact view match.
- **Meaningful protocol proofs.** The proof file includes substantive lemmas: round-trip restoration (`lemma_lock_unlock_roundtrip`), token instance isolation (`lemma_token_instance_isolation`), mutual exclusion (`lemma_mutual_exclusion`), no double-unlock (`lemma_no_double_unlock`), contention resolution (`lemma_contention_resolution_protocol`), and relockability (`lemma_unlock_enables_relock`).
- **Proper split quality.** Spec, proof, and exec are cleanly separated into three files with `include!()` composition. Spec functions are `pub open spec fn` for transparency. Proof lemmas are isolated and do not leak into exec code.
- **Honest trust boundaries.** The documentation explicitly lists what is assumed (ID uniqueness, Arc lifetime, token construction) rather than hiding limitations. This builds confidence in the verification.

## Summary

The mutex verification is a well-executed sequential model of a concurrent primitive. It achieves clean verification (27/0) with no trust escapes, and the documentation is exemplary in its transparency about scope and limitations.

The main weakness is inherent to the approach: the sequential `&mut self` model cannot verify the concurrent contention scenario that is the mutex's raison d'être. The `lock()` precondition (`spec_is_unlocked`) eliminates the retry loop entirely, and the publicly constructible `MutexToken` means mutual exclusion is an assumption rather than a theorem. These are acknowledged limitations, not oversights.

The verification successfully proves: (1) the state machine protocol is correct (lock/unlock transitions preserve `wf()`), (2) tokens are instance-bound (cross-mutex token reuse is precondition-blocked), (3) double-unlock is prevented, (4) round-trip semantics are preserved, and (5) the mutex is relockable after unlock. These properties are valuable for the sequential correctness of the lock protocol.

**Recommendation:** Accept the verification as-is for the sequential model. For a future iteration, prioritize restricting `MutexToken` construction (H2) and sketching a refinement argument connecting the sequential model to the concurrent implementation (M4).
