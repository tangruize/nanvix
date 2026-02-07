# Review: mutex (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

(none)

### High

- **Location:** `lock()` in `mutex.rs` (exec), line 219–235
  - **Description:** The verified `lock()` requires `old(self).spec_is_unlocked()` as a precondition, meaning it can only be called when the mutex is already unlocked. This fundamentally changes the semantics: the original `lock()` is a *blocking* operation that can be called on an *already-locked* mutex (the caller sleeps on the Condvar until the mutex becomes available). The verified version transforms a blocking wait into a guaranteed-success-on-first-try operation. While the documentation explicitly acknowledges this as intentional (sequential model), it means the verification does not prove the correctness of the most important use case — contended locking — which is the primary reason a mutex exists. Any caller in the kernel that calls `lock()` when the mutex *might* be held by another context is outside the verified model's coverage.
  - **Suggested Fix:** This is an inherent limitation of sequential verification of concurrent primitives. Document this prominently as a known gap. Consider adding a proof lemma that at least demonstrates the lock-wait-unlock protocol at the spec level (e.g., modeling the state transition sequence: thread A locks → thread B calls lock → thread A unlocks → thread B acquires).

- **Location:** `try_lock()` in `mutex.rs` (exec), line 181–206
  - **Description:** The original `try_lock()` takes `&self` (shared reference with atomic interior mutability). The verified version takes `&mut self` (exclusive reference). This means the verification cannot express the scenario where two callers simultaneously call `try_lock()` — exactly the contention scenario `try_lock()` is designed for. The postcondition `self.locked` (line 188) is trivially true since after the `if` branch both paths end with the mutex locked, but this does not capture the atomic CAS semantics.
  - **Suggested Fix:** Acknowledge in the review/docs that `&mut self` verification only covers the state machine transitions, not the concurrent correctness that `compare_exchange` provides. This is partially documented but deserves explicit mention in the function's doc comment.

### Medium

- **Location:** `Mutex` struct in `mutex.rs` (exec), line 130–138
  - **Description:** All fields (`locked`, `id`, `token_issued`) are declared `pub`. While the comment says "required by Verus for `pub open spec fn` access," this deviates from the original where `MutexInner.locked` and `MutexInner.sleeping` are private fields. The Nanvix coding standards require struct fields to be private with getter/setter access. While this is a Verus tooling constraint, it weakens the encapsulation guarantee that the original code provides.
  - **Suggested Fix:** Add a comment per the Nanvix convention noting this is a Verus requirement, and verify whether Verus's `spec(checked)` or accessor patterns could avoid the `pub` exposure.

- **Location:** `mutex.rs` (exec) — missing `reference_count()` and `fmt::Debug`
  - **Description:** The original has `Mutex::reference_count()` (returning `Arc::strong_count`) and `fmt::Debug for MutexGuard`. Neither is modeled. While `reference_count()` is Arc-specific and `Debug` is a formatting trait, the API mapping table documents the omission of `reference_count()` but does not mention `Debug`. For completeness, both omissions should be documented.
  - **Suggested Fix:** Add `fmt::Debug for MutexGuard` to the API mapping table noting it is out of scope (display-only, no state mutation).

- **Location:** `mutex.proof.rs`, lemmas overall
  - **Description:** The proof lemmas are largely "definition-unfolding" properties — they assert facts that are trivially true by the definitions of the spec functions. For example, `lemma_state_is_total` asserts `locked || !locked`, which is a tautology. `lemma_try_lock_unlocked_succeeds` asserts `!pre.locked` given `spec_is_unlocked()` which is defined as `!self.locked`. While these serve as regression tests, they don't prove deep properties. The most substantive lemma is `lemma_lock_unlock_roundtrip`, but even that constructs concrete instances rather than reasoning universally over arbitrary state sequences.
  - **Suggested Fix:** Consider adding stronger protocol lemmas: (1) mutual exclusion — two tokens with the same id cannot coexist (follows from wf() + token_issued tracking but worth stating explicitly), (2) no double-unlock — unlock requires a valid token which is consumed, so double-unlock is precondition-blocked (worth a lemma for documentation).

- **Location:** `mutex.spec.rs`, `wf()` predicate, line 72–74
  - **Description:** The well-formedness invariant `self.locked == self.token_issued()` is a biconditional. This is correct for the sequential model, but it means `wf()` is trivially preserved because `locked` and `token_issued` are always set together in the exec code. A stronger invariant (e.g., "at most one token exists per mutex id in the entire system") would be more meaningful but is beyond Verus's current tracked-token model without a global resource algebra.
  - **Suggested Fix:** No code change needed, but document in the spec file that `wf()` is a local (per-mutex) invariant and does not capture the global uniqueness property.

### Low

- **Location:** `mutex.rs` (exec), line 155
  - **Description:** `new()` takes `Ghost(id): Ghost<nat>` requiring callers to supply a unique ghost ID. The original `Mutex::new()` takes no arguments — identity is implicit via `Arc` pointer equality. Trust assumption T1 (ID uniqueness) is documented but there is no mechanism to enforce it. In a kernel with many mutexes, accidental ID reuse could silently invalidate token isolation.
  - **Suggested Fix:** Consider using a ghost global counter or `vstd::state_machine_internal` allocation token to guarantee fresh IDs, if Verus supports it. Otherwise, the documented trust assumption is acceptable.

- **Location:** `mutex.rs` (exec), `unlock()` line 251–268
  - **Description:** The original `unlock_unchecked()` calls `self.sleeping.notify_first()` which can return an error, handled with a `warn!()` log in `Drop`. The verified `unlock()` has no error path — it always succeeds. This is acceptable since the error case is a Condvar notification failure (external dependency), but the divergence should be noted.
  - **Suggested Fix:** Add a note in the API mapping table that the error path from `notify_first()` is not modeled.

- **Location:** `mutex.proof.rs`, `lemma_lock_token_snapshot_is_locked`, line 108–114
  - **Description:** This lemma asserts that a token whose `view.locked` is true has a view equal to `MutexView { locked: true, ... }`. This is a structural identity tautology and provides no additional insight.
  - **Suggested Fix:** Consider replacing with a more useful lemma, or remove to reduce noise.

## Positive Observations

- **No assume/external_body/trusted annotations.** The entire mutex module is fully verified with no trust gaps in the core logic. This is a strong result — `lock()` successfully delegates to `try_lock()` without needing external_body, unlike many concurrent primitive verifications.
- **Excellent documentation.** The module-level doc comment is exceptionally thorough, covering verification model, scope, API mapping, divergences, trust boundaries, and trust assumptions. This is a model for how verified code should be documented.
- **Clean spec/proof/exec separation.** The three-file split is well-organized: `mutex.spec.rs` contains view types and spec functions, `mutex.proof.rs` contains proof lemmas, and `mutex.rs` contains exec code with pre/postconditions. No proof logic leaks into exec code.
- **Sound token-based ownership model.** The `MutexToken` tracked struct correctly models the `MutexGuard` RAII pattern. Token binding via `token.view == self@` ensures instance isolation. The token is consumed by `unlock()`, preventing double-unlock.
- **Well-formedness preserved across all transitions.** Every exec function maintains the `wf()` invariant in its postconditions, establishing a state machine where all reachable states are well-formed.
- **All 24 verification conditions pass** with no errors, in 4 seconds. The verification is stable and efficient.
- **Honest about limitations.** The documentation explicitly lists what is out of scope (concurrency, liveness, Condvar interaction, Arc, Drop, timeout) rather than overclaiming.

## Summary

This is a solid sequential verification of the mutex state machine protocol. The verification successfully proves that lock/unlock transitions are correct, tokens are instance-bound, well-formedness is preserved, and the round-trip property holds — all without any trust gaps (`assume`/`external_body`).

The main limitation is inherent to the approach: verifying a concurrent primitive with `&mut self` (exclusive access) cannot capture the concurrent scenarios that motivate the mutex's existence. The `lock()` precondition requiring `spec_is_unlocked()` means contended locking — the primary use case — is outside the verified model. This is honestly documented and is a reasonable trade-off given Verus's current capabilities for reasoning about atomics and shared-memory concurrency.

The proof lemmas, while numerous (14 lemmas), are mostly definitional unfolding rather than deep protocol properties. Adding lemmas for mutual exclusion (two tokens cannot coexist) and no-double-unlock (precondition-blocked) would strengthen the proof artifact.

**Recommendations:**
1. Add a mutual-exclusion lemma proving that `wf()` + `token_issued` tracking prevents two tokens from coexisting for the same mutex instance.
2. Add `fmt::Debug` and `notify_first()` error path to the API divergence documentation.
3. Consider whether Verus's `state_machine!` macro could provide a more natural model for the lock protocol with explicit state transitions.
