# Review: fence (claude-opus-4.6)

## Grade: B+

## Issues Found

### Critical

_None._

### High

- **Location:** `wait()` in exec (`fence.rs:170-180`)
  **Description:** The verified `wait()` requires `self.spec_is_satisfied()` as a precondition, meaning the caller must prove the fence is already satisfied _before_ calling wait. This fundamentally inverts the semantics of the original `wait(&self)`, which _blocks until_ the fence becomes satisfied. The verification proves nothing about wait's core purpose — that it blocks until concurrent signalers deliver enough signals. The precondition essentially makes `wait()` a no-op assertion rather than a synchronization operation. While the documentation acknowledges this honestly, it remains the largest semantic gap in the verification.
  **Suggested Fix:** This is an inherent limitation of the sequential model. A future improvement could use a ghost protocol token or a rely/guarantee framework where `wait` consumes a proof obligation that `total` signal tokens have been issued. Even without full concurrency modeling, a token-passing spec would better capture the intent that wait _depends on_ external signal events rather than requiring satisfaction upfront.

- **Location:** `signal()` in exec (`fence.rs:193-207`)
  **Description:** The original `signal(&self)` uses `&self` with `AtomicUsize::fetch_add`, allowing concurrent callers. The verified `signal(&mut self)` requires `&mut self` (exclusive access), which prevents modeling the primary use case: multiple threads signaling concurrently. This is documented but constitutes a significant model gap for a synchronization primitive whose entire purpose is concurrent access.
  **Suggested Fix:** Consider using Verus `vstd::atomic` types (e.g., `PAtomicUsize` with ghost state) to model the atomic increment, which would allow `&self` and concurrent reasoning. This would close both the `signal` and `wait` model gaps simultaneously.

### Medium

- **Location:** `signal()` precondition strengthening (`fence.rs:193-195`)
  **Description:** The verified `signal()` requires `spec_is_waiting()` (i.e., `count < total`) as a precondition. The original `signal(&self)` has no such guard — it unconditionally calls `fetch_add(1, Release)`. While the documentation explains this as intentional protocol hardening, it means the verified model rejects programs that the original accepts. If over-signaling is actually benign in certain kernel use patterns (e.g., idempotent signal delivery), the strengthened precondition would flag correct code as invalid. This is a specification tightness concern rather than a bug.
  **Suggested Fix:** Audit kernel callers to confirm over-signaling never occurs. If it does, relax the precondition by modeling count as unbounded (or capped at `usize::MAX`) and adjust `wf()` accordingly. If over-signaling is truly a bug in all callers, the strengthening is justified and should be documented as a verified contract that the runtime should enforce.

- **Location:** `new()` in exec (`fence.rs:142-154`)
  **Description:** The original `new` is `const fn`, which the verified version cannot model. This is minor but means the verification does not cover the compile-time evaluation context in which `Fence::new` may be used (e.g., static initialization). The documentation notes this.
  **Suggested Fix:** No action needed unless Verus adds `const fn` support. The runtime behavior is identical.

- **Location:** Proof lemmas — trivial arithmetic (`fence.proof.rs`)
  **Description:** Several lemmas prove trivially true arithmetic facts that Verus auto-discharges (e.g., `lemma_total_signals_satisfies`: `count == total ==> count >= total`; `lemma_satisfaction_is_monotone`: `count >= total ==> count >= total`; `lemma_wait_on_satisfied_is_noop`: `count >= total ==> !(count < total)`). While harmless as regression tests, they inflate the proof surface without adding meaningful assurance. The more valuable lemmas are `lemma_signals_accumulate_to_satisfaction` and `lemma_signal_commutativity`.
  **Suggested Fix:** Consider marking trivial definitional lemmas as such (e.g., grouping under a "definitional" section, which is already partially done) and focusing proof effort on deeper properties like inductive signal sequences or concurrent linearizability.

### Low

- **Location:** `is_satisfied()`, `get_count()`, `get_total()` in exec (`fence.rs:218-260`)
  **Description:** These helper functions do not exist in the original source. While useful for verification and testing, they expand the API surface of the verified model beyond the original. This is cosmetic — they are pure observers.
  **Suggested Fix:** No action needed. They are clearly documented as verification-only accessors.

- **Location:** `FenceView` in spec (`fence.spec.rs:20-25`)
  **Description:** `FenceView` uses `pub` fields, while the original `Fence` has private fields. The view type is spec-only and does not affect exec semantics, but it does expose internal state at the specification level. This is standard Verus practice.
  **Suggested Fix:** No action needed.

- **Location:** `Fence` struct fields (`fence.rs:116-121`)
  **Description:** Both `count` and `total` are `pub` in the verified struct, while the original has private fields. The comment explains this is required by Verus for `pub open spec fn` access, which is correct. However, this means exec code could directly mutate fields, bypassing `signal()`.
  **Suggested Fix:** If Verus supports `pub(crate)` or getter-only access patterns in the future, restrict field visibility. For now, this is an accepted Verus limitation.

## Positive Observations

- **Excellent documentation.** The module-level doc comment is exceptionally thorough, clearly delineating the verification model, trust boundaries, API divergences, and out-of-scope concerns. This is among the best-documented verification efforts I've reviewed — it sets clear expectations and prevents misinterpretation of what the proofs guarantee.
- **No assume/external_body/trusted.** The entire module is verified without any trust assumptions in the Verus sense. All 24 verification conditions pass cleanly.
- **Clean spec/proof/exec separation.** The three-file split (`fence.rs`, `fence.spec.rs`, `fence.proof.rs`) with `include!` is well-organized. Specs are `pub open` for composability, proofs are standalone lemmas, and exec code is minimal.
- **Well-formedness invariant is well-chosen.** `wf()` as `count <= total` is the correct reachable-state invariant, and it is preserved across all transitions (new, signal). The proofs establish this inductively.
- **Signal commutativity lemma** (`lemma_signal_commutativity`) is a valuable spec-level property that bridges toward concurrent reasoning even within the sequential model.
- **Accumulation lemma** (`lemma_signals_accumulate_to_satisfaction`) with the `forall` quantifier over intermediate states is the strongest proof in the module, establishing that every intermediate state during the signal sequence maintains well-formedness.
- **Honest trust boundary documentation.** The `wait()` gap is transparently documented rather than hidden, which is the correct approach for a sequential model of a concurrent primitive.

## Summary

The fence verification is a solid sequential model of a concurrent synchronization primitive. It correctly verifies the state machine protocol: well-formedness is preserved across all transitions, satisfaction is monotone, and exactly `total` signals yield satisfaction. The spec/proof/exec split is clean, documentation is outstanding, and there are no soundness escape hatches.

The primary limitation is inherent to the sequential modeling approach: `wait()` becomes a no-op with a satisfaction precondition (inverting its runtime semantics), and `signal()` requires exclusive access (preventing concurrent modeling). These are the most significant semantic gaps. The `signal()` precondition strengthening (rejecting over-signaling) is a deliberate design choice that should be validated against actual kernel callers.

For a simple data structure like `Fence` (essentially a counter with a threshold), the sequential model captures most of the interesting correctness properties. The grade reflects that the verification is well-executed within its chosen scope, but the scope itself excludes the concurrency aspects that are the raison d'être of the primitive. Advancing to an atomic/token-based model would elevate this to an A-grade verification.
