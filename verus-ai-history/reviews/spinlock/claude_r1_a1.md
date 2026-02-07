# Review: spinlock (claude-opus-4.6)

## Grade: B+

## Issues Found

### Critical

_(none)_

### High

- **H1: `SpinlockGuard` and `Drop` not modeled**
  - **Location:** exec (`spinlock.rs`), entire module
  - **Description:** The original spinlock's core safety mechanism is RAII-based automatic
    unlock via `SpinlockGuard<'a>` and its `Drop` implementation. This pattern structurally
    guarantees that every successful `lock()` is paired with an `unlock` — forgetting to unlock
    is impossible at the type level. The verified code replaces this with an explicit `unlock()`
    method, which loses this guarantee. A caller can now `lock()` and never call `unlock()`,
    and Verus will not flag it. This is the most important safety property of the original
    design and it is entirely unverified. While the documentation acknowledges this as a trust
    boundary ("Drop-based reasoning requires lifetime-aware resource tracking beyond Verus's
    current scope"), the absence still represents a significant gap.
  - **Suggested Fix:** At minimum, add a proof lemma or spec-level protocol that models the
    lock-must-be-released obligation (e.g., a tracked ghost token or a `spec fn
    lock_release_obligation` that callers must discharge). If Verus cannot model Drop directly,
    document a manual audit checklist for all call sites. Consider adding a `SpinlockGuard`
    stub that carries the unlock obligation in its spec.

- **H2: `lock()` missing sequential-model precondition**
  - **Location:** exec (`spinlock.rs:122-129`)
  - **Description:** `lock()` is `external_body` with no `requires` clause. In the sequential
    model used for verification, calling `lock()` on an already-locked spinlock is an infinite
    loop (deadlock). The postcondition `ensures self.locked` is trivially satisfiable by a
    no-op on an already-locked spinlock, so Verus will happily accept code that deadlocks in
    the sequential model. The original `lock(&self)` avoids this issue because it relies on
    concurrent unlock by another thread, but the verified `lock(&mut self)` operates
    sequentially where that rescue is impossible.
  - **Suggested Fix:** Add `requires old(self).spec_is_unlocked()` to the sequential model
    of `lock()`. This faithfully captures the sequential invariant: you may only call `lock()`
    when the lock is available. Callers that violate this represent concurrent access patterns
    that are outside the sequential verification scope anyway.

### Medium

- **M1: API signature divergence — `lock(&self) -> SpinlockGuard` vs `lock(&mut self)`**
  - **Location:** exec (`spinlock.rs:123`)
  - **Description:** The original `lock()` takes `&self` (interior mutability via `AtomicBool`)
    and returns `SpinlockGuard`. The verified version takes `&mut self` and returns nothing.
    This changes the aliasing semantics: with `&self`, multiple call sites can hold references
    to the same spinlock simultaneously (the whole point of a spinlock for concurrent access).
    With `&mut self`, Rust's borrow checker enforces exclusive access, making the spinlock
    redundant. While this is a necessary consequence of the sequential verification model,
    it means the verified code cannot be used as a drop-in replacement for the original and
    callers would need different patterns.
  - **Suggested Fix:** Document this divergence more prominently. In the module doc header,
    explicitly state that the verification covers the *state machine protocol* (lock/unlock
    transitions) but not the *concurrent access pattern* that motivates the spinlock's
    existence. Consider adding a spec-level note: "In the concurrent original, `&self` access
    is safe due to `AtomicBool` interior mutability."

- **M2: `try_lock()` and `is_locked()` are not in the original source**
  - **Location:** exec (`spinlock.rs:98-109`, `spinlock.rs:157-163`)
  - **Description:** `try_lock()` and `is_locked()` are added functions with no corresponding
    implementation in the original `spinlock.rs`. While `try_lock()` usefully decomposes the
    single CAS operation from `lock()`'s loop body, and `is_locked()` is a natural observer
    method, these are new API surface that should be clearly marked as verification helpers
    rather than verified originals. Introducing unverified-against-original functions could
    create confusion about what is "verified original behavior" vs. "verification scaffold."
  - **Suggested Fix:** Annotate `try_lock()` and `is_locked()` with a doc comment:
    `/// NOTE: Verification helper — not present in original source.` This makes the
    distinction clear for reviewers and future maintainers.

- **M3: `try_lock()` postcondition obscures failure semantics**
  - **Location:** exec (`spinlock.rs:99-101`)
  - **Description:** The postcondition `ensures result == !old(self).locked, self.locked`
    is technically correct (both branches end with `self.locked == true`), but it does not
    explicitly capture "state unchanged on failure." The ensures clause `self.locked` reads
    as "the lock is always locked after try_lock" — which may surprise callers who expect a
    failed try_lock to be a no-op. While mathematically equivalent (if already locked, locked
    stays true), a postcondition like `!result ==> self@ == old(self)@` would be more
    expressive of the intended semantics.
  - **Suggested Fix:** Add an additional ensures clause:
    `!result ==> self@ == old(self)@` to explicitly state that a failed try_lock does not
    mutate the spinlock's abstract state. This strengthens documentation without affecting
    proof obligations.

### Low

- **L1: `wf()` is trivially true**
  - **Location:** spec (`spinlock.spec.rs:46-48`)
  - **Description:** The well-formedness predicate `wf()` returns `true` unconditionally.
    While this is technically correct for a single-boolean struct, it adds no verification
    value — any `Spinlock` value satisfies `wf()`, so requiring `wf()` in a precondition
    gains nothing. It appears to be included for API consistency with other verified modules
    that have meaningful `wf()` predicates.
  - **Suggested Fix:** No code change needed; add a brief doc comment: `/// Trivially true
    for Spinlock (no structural invariants beyond a valid bool).` to prevent future
    maintainers from assuming this is an oversight.

- **L2: Proof lemmas are mostly trivial**
  - **Location:** proof (`spinlock.proof.rs`)
  - **Description:** All 9 proof lemmas are automatically discharged by Verus without any
    proof body — they are essentially tautologies over boolean logic (e.g.,
    `lemma_state_is_total`: `locked || !locked`; `lemma_locked_unlocked_complementary`:
    `locked == !(!locked)`). While they serve as documentation of intended properties,
    they don't prove anything non-trivial. The most interesting lemma
    (`lemma_lock_unlock_roundtrip`) constructs explicit states rather than quantifying
    over arbitrary pre/post states.
  - **Suggested Fix:** Consider adding a more substantive lemma that proves a protocol
    property, e.g., "after `new()` followed by `try_lock()`, the result is always `true`"
    or a two-phase commit-style lemma that ties `lock`/`unlock` to a ghost resource.

- **L3: `SpinlockView` adds minimal abstraction**
  - **Location:** spec (`spinlock.spec.rs:19-22`)
  - **Description:** `SpinlockView` has a single `locked: bool` field, identical to the
    exec `Spinlock` struct. The `View` implementation is a trivial identity mapping
    (`SpinlockView { locked: self.locked }`). For the current single-field struct, the
    abstraction layer provides no information hiding — the spec view is identical to the
    exec representation. The view exists for consistency with Verus conventions and would
    become useful if the exec struct gained additional fields.
  - **Suggested Fix:** No change needed. This follows Verus conventions and is forward-
    compatible.

- **L4: `unlock()` postcondition could reference `old(self)`**
  - **Location:** exec (`spinlock.rs:141-148`)
  - **Description:** The `unlock()` postcondition includes `!self.locked`,
    `self.spec_is_unlocked()`, and `self@ == Spinlock::spec_new_view()`. While correct, it
    doesn't assert anything about the state transition relative to `old(self)` (e.g.,
    `old(self).locked && !self.locked`). This makes it harder to reason about the transition
    at call sites — callers know the post-state but not that a transition occurred.
  - **Suggested Fix:** Add `old(self).spec_is_locked()` to the ensures (redundant with
    the `requires` clause, but makes postconditions self-contained for callers who read
    only the ensures).

## Positive Observations

- **Well-documented trust boundaries.** The module header clearly explains why `lock()` uses
  `external_body`, why `SpinlockGuard`/`Drop` are not modeled, and the relationship between
  the sequential model and the concurrent original. This is excellent practice.

- **Clean spec/proof/exec separation.** The three-file split is well-organized: specs define
  abstract predicates and views, proofs establish lemmas, exec implements verified functions.
  The `include!` mechanism works cleanly.

- **Correct state machine verification.** The verified state transitions (new → unlocked,
  try_lock → locked, unlock → unlocked) correctly capture the sequential lock protocol.
  Postconditions are precise and the `try_lock` implementation is correct.

- **Verification passes cleanly.** 14 verified, 0 errors. No `assume` statements. Only one
  justified `external_body` for the inherently unverifiable spin loop.

- **`try_lock()` is a good decomposition.** Extracting the single CAS attempt as a verifiable
  function is a sound strategy — it allows the core state transition to be fully verified while
  isolating the unverifiable spin loop to `external_body`.

- **Consistent coding style.** Copyright headers, section separators, doc comments, and naming
  conventions all follow the Nanvix project standards.

## Summary

The spinlock verification is a competent sequential model of a fundamentally concurrent
primitive. It correctly verifies the lock state machine protocol (new/try_lock/unlock
transitions) with clean specifications and well-documented trust boundaries.

The main weakness is the absence of `SpinlockGuard`/`Drop` verification (High), which is the
original's primary safety mechanism ensuring locks are always released. The `lock()` function's
`external_body` without a sequential precondition (High) could mask deadlock bugs in
sequential reasoning. Both are documented as known limitations, which is good practice.

The verification adds functions not in the original (`try_lock`, `is_locked`) and changes
the API signature (`&self` → `&mut self`), which are reasonable modeling choices but should
be more prominently documented. The proof lemmas are trivially true and serve more as
documentation than deep verification.

**Recommendations for improvement (priority order):**
1. Add `requires old(self).spec_is_unlocked()` to `lock()` for sequential-model safety.
2. Add a ghost-token or obligation-based model for the lock-release guarantee.
3. Annotate `try_lock()`/`is_locked()` as verification helpers not in the original.
4. Add `!result ==> self@ == old(self)@` to `try_lock()` ensures for clarity.
