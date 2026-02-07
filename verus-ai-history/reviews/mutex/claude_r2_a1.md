# Review: mutex (claude-opus-4.6)

## Grade: B+

## Issues Found

### Critical

(none)

### High

- **Location:** `Mutex::lock()` (exec — `mutex.rs:234`)
  - **Description:** The `lock()` precondition requires `old(self).spec_is_unlocked()` and `!old(self).token_issued()`, meaning it can only be called on an already-unlocked mutex. In the original implementation, `lock()` is specifically designed to handle the contended case — it loops calling `try_lock()` and sleeps on a `Condvar` when the mutex is already held. By requiring the mutex to be unlocked, the verified `lock()` collapses to a single `try_lock()` call that always succeeds, and the entire blocking/retry protocol (the primary reason a mutex exists) is unverified. The documentation acknowledges this, but it is the largest functional gap in the verification.
  - **Suggested Fix:** This is inherently difficult in a sequential `&mut self` model. Consider adding a proof-level lemma that explicitly models the loop invariant: "if the mutex eventually becomes unlocked (fairness assumption), then `lock()` terminates with the mutex held." Even without a full concurrent model, a termination argument conditioned on a fairness ghost predicate would strengthen the verification. Alternatively, model `lock()` as an `external_body` with a weaker postcondition that honestly states what is proven vs. assumed about blocking.

### Medium

- **Location:** `Mutex::wf()` (spec — `mutex.spec.rs:77`)
  - **Description:** The well-formedness invariant `self.locked == self.token_issued()` is a local (per-instance) biconditional. It does not capture the global uniqueness property that at most one `MutexToken` exists per mutex `id` across the entire system. Since `MutexToken` is a `tracked struct` with a `pub ghost view` field, nothing structurally prevents constructing a second token with the same view outside of the module boundary. The spec comment (line 72–76) acknowledges this, but it means the mutual exclusion guarantee is only as strong as the trust assumption that tokens are created solely via `try_lock()`/`lock()`.
  - **Suggested Fix:** Document this as a trust assumption (T3) in the module header, parallel to T1 (ID uniqueness) and T2 (Arc lifetime). In a future iteration, consider using Verus's `tracked` ghost resource algebra or a singleton token pattern to enforce global uniqueness at the type level.

- **Location:** `Mutex::unlock()` (exec — `mutex.rs:266`)
  - **Description:** The original `MutexInner::unlock_unchecked()` calls `self.sleeping.notify_first()` which returns `Result<(), Error>`. The `Drop` impl handles this error with a `warn!()` log. The verified `unlock()` has no error path — it always succeeds. This means the condvar notification failure scenario is unmodeled, and the verification does not capture the (admittedly defensive) error-handling behavior of the original.
  - **Suggested Fix:** Since the condvar module is separately verified and `notify_first()` failure is an edge case tied to external process manager state, this is acceptable as-is. Add a brief comment in the `unlock()` doc explaining that condvar notification failure is handled in the original but out of scope for the mutex state machine model.

- **Location:** `Mutex::try_lock()` return type (exec — `mutex.rs:194`)
  - **Description:** The return type `(bool, Tracked<Option<MutexToken>>)` diverges from the original's `Result<MutexGuard, ()>`. While functionally equivalent, the `Result`-based API provides stronger ergonomic guarantees: the `MutexGuard` is only available on the `Ok` path, whereas the verified version returns a tuple where the caller must manually correlate the `bool` with the `Option`. This weakens the verified API's ability to prevent misuse (e.g., ignoring the bool and unwrapping the `None`).
  - **Suggested Fix:** Consider returning `Result<Tracked<MutexToken>, ()>` to match the original's `Result` pattern. The token would only be available on the `Ok` path, structurally preventing misuse.

### Low

- **Location:** `Mutex::is_locked()` (exec — `mutex.rs:290`)
  - **Description:** `is_locked()` exists in the verified code but has no counterpart in the original source. While harmless (it's a simple accessor), it expands the verified API surface beyond the original.
  - **Suggested Fix:** No action required. Consider noting it as a verification-only helper in the API mapping table.

- **Location:** `Mutex::reference_count()` — not modeled
  - **Description:** `reference_count()` is not modeled in the verified code. This is documented and justified (Arc-specific), but it means any code relying on reference count semantics for correctness is outside verification scope.
  - **Suggested Fix:** No action required. Documented appropriately.

- **Location:** `fmt::Debug for MutexGuard` — not modeled
  - **Description:** The `Debug` implementation is not modeled. It reads `locked` state via `Ordering::Relaxed`, which has no effect on mutex correctness.
  - **Suggested Fix:** No action required. Display-only trait impls are reasonably excluded.

- **Location:** Proof lemmas `lemma_try_lock_unlocked_succeeds` / `lemma_try_lock_locked_fails` (proof — `mutex.proof.rs:62,71`)
  - **Description:** These lemmas are trivially discharged by definition unfolding — they merely restate that `spec_is_unlocked()` implies `!locked` and vice versa. They don't actually prove anything about `try_lock()` behavior (e.g., they don't invoke or reason about the function's postconditions). Their names overstate their contribution.
  - **Suggested Fix:** Rename to `lemma_unlocked_implies_not_locked` / `lemma_locked_implies_locked` for accuracy, or strengthen them to reason about `try_lock()` postconditions applied to a specific pre-state.

## Positive Observations

- **Zero `assume`/`external_body`/`trusted` usage.** All 27 verification conditions are fully discharged by the solver with no escape hatches. This is the gold standard for Verus verification soundness.
- **Exceptionally thorough documentation.** The module header (lines 1–108) provides a comprehensive API mapping table, explicit divergence documentation, trust boundaries, trust assumptions, and verification scope. This is among the best-documented Verus verification modules I have reviewed.
- **Clean spec/proof/exec separation.** The three-file split is well-organized: view types and spec functions in `mutex.spec.rs`, proof lemmas in `mutex.proof.rs`, and executable code in `mutex.rs`. The `include!` mechanism is used correctly.
- **Strong token-based ownership model.** The `MutexToken` ghost type cleanly models the `MutexGuard`/`Drop` RAII pattern. The token is bound to the mutex instance via view identity, and `unlock()` consumes it — preventing double-unlock at the type level.
- **Comprehensive proof coverage.** The 17 proof lemmas cover definitional properties, protocol properties (round-trip, relockability), safety properties (no double-unlock, mutual exclusion), and instance isolation. The `lemma_contention_resolution_protocol` is a particularly nice spec-level demonstration of the contention sequence.
- **Honest about limitations.** The "Verification Scope" section explicitly enumerates what is out of scope (concurrency, liveness, condvar, Arc, Drop, timeout). This prevents users from overestimating the verification's guarantees.

## Summary

The mutex verification is a well-crafted sequential state machine model that proves the lock/unlock protocol correct within its domain. The verification passes cleanly (27/0) with no escape hatches, and the documentation is exemplary in its honesty about the gap between the sequential model and the concurrent original.

The primary limitation is that the `lock()` function — the core blocking acquisition path that justifies the mutex's existence — is verified only for the trivial uncontended case. The contended case (retry loop + condvar sleep) is entirely unmodeled due to the `&mut self` sequential model. This is an inherent limitation of verifying concurrent primitives with sequential reasoning, not a flaw in the verification itself, but it does mean the most safety-critical aspect of mutex behavior (correct blocking under contention) remains unverified.

Recommendations for improvement:
1. Strengthen `lock()` with a fairness-conditioned termination argument or mark it `external_body` with a weaker postcondition that separates proven from assumed properties.
2. Consider returning `Result<Tracked<MutexToken>, ()>` from `try_lock()` to match the original's API pattern.
3. Add a T3 trust assumption for token uniqueness alongside the existing T1/T2 assumptions.
4. Rename or strengthen the trivial proof lemmas to avoid overstating their contribution.
