# Review: spinlock (claude-opus-4.6)

## Grade: B+

## Issues Found

### Critical

None.

### High

- **Location:** `try_lock()` preconditions (exec: `spinlock.rs:173-176`)
  - **Description:** The preconditions `old(self).wf()` and `!old(self).token_issued()` together with `wf()` (which is `self.locked == self.token_issued()`) imply `!old(self).locked`. This means the failure path (`else` branch at line 193) is dead code — `try_lock` always succeeds. The function does not model "try" semantics at all; it is functionally identical to `lock()` with a redundant boolean return. The postconditions on the failure path (lines 180, 184) are vacuously true and never exercised. This weakens the verification because the most interesting property of `try_lock` — that it can fail on a contended lock — is never actually proven.
  - **Suggested Fix:** To genuinely model `try_lock`, weaken the precondition to only require `wf()` (drop `!old(self).token_issued()`). This requires the failure path to handle the case where `locked == true && token_issued == true`, which the current `else` branch already does correctly. The postcondition `!result.0 ==> self@ == old(self)@` would then be non-vacuously verified. This would require `lock()` to be reworked (it currently delegates to `try_lock` and assumes success).

- **Location:** `lock()` preconditions (exec: `spinlock.rs:214-217`)
  - **Description:** Requiring `old(self).spec_is_unlocked()` as a precondition means the caller must already know the lock is free before calling `lock()`. This eliminates the defining behavior of a spinlock: spinning until the lock becomes available. The sequential model reduces `lock()` to a simple state flip with no contention possible. While documented in the "Verification Scope" section, this means the verification does not cover the most safety-critical scenario — what happens when `lock()` is called on an already-locked spinlock (deadlock in single-threaded, spin-wait in multi-threaded).
  - **Suggested Fix:** This is a fundamental limitation of the sequential `&mut self` model and cannot be fully resolved without a concurrent verification framework (e.g., Verus's `tokenized_state_machine!` or Iron-style atomic invariants). Consider adding a comment-level argument or a proof lemma showing that the precondition is necessary to prevent deadlock in the sequential model, which would strengthen the documentation of this design choice.

### Medium

- **Location:** Proof lemmas (proof: `spinlock.proof.rs`)
  - **Description:** The majority of proof lemmas (12 of 16) are trivially true by definition unfolding and have empty proof bodies. Examples: `lemma_state_is_total` (a bool is true or false), `lemma_locked_unlocked_complementary` (restates `!` relationship), `lemma_view_reflects_locked` (restates view definition), `lemma_try_lock_unlocked_succeeds` (restates `spec_is_unlocked` definition), `lemma_try_lock_locked_fails` (restates `spec_is_locked` definition). While these serve as documentation of properties, they don't exercise Verus's proof capabilities and inflate the verified property count without adding verification depth.
  - **Suggested Fix:** Retain `lemma_lock_unlock_roundtrip`, `lemma_token_instance_isolation`, `lemma_unlocked_eq_new_view`, and `lemma_new_then_try_lock_succeeds` as the substantive lemmas. Consider consolidating the trivial lemmas into a single "basic properties" lemma or removing them, noting that these properties follow directly from definitions. If retained for documentation, mark them explicitly as definition-unfolding lemmas.

- **Location:** `Spinlock` struct fields (exec: `spinlock.rs:112-122`)
  - **Description:** All struct fields (`locked`, `id`, `token_issued`) are `pub`, breaking encapsulation. The documentation explains this is required by Verus for `pub open spec fn` access, but it means any code with access to a `Spinlock` value can directly read or construct a `Spinlock` bypassing `new()`. Specifically, a caller could construct `Spinlock { locked: true, id: Ghost(0), token_issued: Ghost(false) }` which violates `wf()`, then pass it to `unlock()` — the precondition check would catch this, but it weakens the type-level guarantee.
  - **Suggested Fix:** This is a known Verus limitation. Add a comment on the struct noting that callers should only construct via `new()` and rely on `wf()` preconditions for safety. Alternatively, investigate if Verus's `pub(crate)` visibility suffices for spec function access.

- **Location:** Ghost `id` uniqueness (trust assumption T1, exec: `spinlock.rs:79-82`)
  - **Description:** Token instance isolation (`lemma_token_instance_isolation`) depends on distinct spinlock instances having distinct `id` values. This is a caller obligation with no mechanical enforcement. If two spinlocks are created with the same `id`, a token from one could satisfy the `unlock()` precondition of the other (since `token.view == old(self)@` checks the full view including `id`, but two same-id same-state locks would have equal views).
  - **Suggested Fix:** Document this as a verified-but-trusted assumption. Consider adding a ghost monotonic counter pattern (a global `Ghost<nat>` that increments per `new()` call) to mechanically enforce uniqueness, or use Verus's `instance` tracking to bind tokens to specific allocations.

### Low

- **Location:** `SpinlockGuard` not modeled as a type (exec: `spinlock.rs`)
  - **Description:** The original `SpinlockGuard<'a>(&'a Spinlock)` carries a lifetime-bound reference ensuring the guard cannot outlive the spinlock. The verified `LockToken` carries a ghost `SpinlockView` snapshot but has no lifetime relationship to the `Spinlock` it came from. This means the verification cannot prove that a token is used before its originating spinlock is deallocated.
  - **Suggested Fix:** This is a fundamental modeling gap that Verus's current tracked types cannot fully bridge. The `token.view == old(self)@` check at unlock time partially compensates, since a deallocated/reallocated spinlock would likely have different state. Document this as an explicit trust boundary.

- **Location:** `new()` signature divergence (exec: `spinlock.rs:139`)
  - **Description:** Original `new()` takes no arguments and is `const fn`. Verified `new()` takes `Ghost(id): Ghost<nat>` — a ghost parameter erased at runtime. While semantically zero-cost, this changes the function signature. The original is also `const fn` which the verified version is not (Verus limitation).
  - **Suggested Fix:** No action needed — this is an inherent Verus modeling requirement. The ghost parameter is erased at runtime. Document that `const fn` is not supported in Verus.

- **Location:** `&self` vs `&mut self` for `lock()` (exec: `spinlock.rs:213`)
  - **Description:** The original `lock(&self)` uses interior mutability via `AtomicBool`. The verified version uses `lock(&mut self)` because Verus requires exclusive references for mutation. This is well-documented in the "API Divergence" section but means the verification does not cover the concurrent access pattern that justifies a spinlock's existence over a simple boolean flag.
  - **Suggested Fix:** Already documented. No fix possible within the current Verus sequential model. Consider noting that this divergence means the verified code proves "boolean flag protocol correctness" rather than "spinlock correctness."

## Positive Observations

- **No `assume`, `external_body`, or `trusted` annotations.** The entire module is fully verified with no trust holes in the core logic. This is the gold standard for Verus verification soundness.
- **Excellent documentation.** The module-level doc comment thoroughly explains the verification model, scope limitations, trust boundaries, trust assumptions, and API divergences. This level of transparency is exemplary and allows readers to accurately assess what the verification does and does not cover.
- **Clean spec/proof/exec split.** Specifications, proof lemmas, and executable code are cleanly separated into three files with appropriate concerns in each. The `include!` mechanism keeps them unified for Verus while maintaining readability.
- **Sound well-formedness invariant.** The biconditional `wf()` predicate (`locked == token_issued`) is the right strength — it captures the full reachable state space and is preserved across all transitions. This prevents both double-unlock and locked-without-token anomalies.
- **Token-based RAII modeling.** Using a tracked `LockToken` to model `SpinlockGuard`'s `Drop` obligation is a well-chosen design. Verus's linear type system for `tracked` ensures tokens cannot be duplicated or discarded, providing a mechanical analog to RAII discipline.
- **Instance isolation proven.** `lemma_token_instance_isolation` proves that tokens from different lock instances (with different `id`s) cannot be cross-used, which is a meaningful safety property.
- **21 verified conditions, 0 errors.** Clean verification with no warnings or workarounds.

## Summary

The spinlock verification is a well-crafted sequential model that proves the lock/unlock state machine protocol is correct: all reachable states satisfy the well-formedness invariant, tokens are linearly managed and instance-bound, and the lock-then-unlock round-trip restores the initial state. The absence of any `assume` or `external_body` is commendable.

The primary limitation is inherent to the verification approach: the sequential `&mut self` model cannot capture the concurrent semantics that define a spinlock. The `lock()` precondition requiring `spec_is_unlocked()` and `try_lock()` preconditions making the failure path unreachable mean the verification covers only the uncontended "happy path." The most interesting correctness questions about a spinlock — mutual exclusion under concurrent access, deadlock freedom, fairness — are explicitly out of scope.

The proof lemmas, while numerous (16), are mostly trivial definition unfoldings. The substantive lemmas (`lemma_lock_unlock_roundtrip`, `lemma_token_instance_isolation`, `lemma_unlocked_eq_new_view`) do provide value.

**Recommendations:**
1. Weaken `try_lock` preconditions to exercise the failure path non-vacuously.
2. Consider trimming or consolidating trivial proof lemmas.
3. For a future iteration, investigate Verus's `tokenized_state_machine!` macro for modeling concurrent spinlock semantics, which could verify mutual exclusion as an invariant of the token protocol.
