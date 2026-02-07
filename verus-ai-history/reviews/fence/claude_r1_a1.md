# Review: fence (claude-opus-4.6)

## Grade: B+

## Issues Found

### Critical

_None._

### High

- **Location:** `wait()` in fence.rs (exec), line 143
- **Description:** The `wait()` function requires `self.spec_is_satisfied()` as a precondition, which means the caller must prove the fence is already satisfied *before* calling `wait()`. In the original code, `wait()` is the *blocking mechanism* that achieves satisfaction—it spin-loops until `count >= total`. The verified model transforms a blocking operation into a no-op with a precondition, making `wait()` vacuously correct. Any caller that can prove `spec_is_satisfied()` has no reason to call `wait()` at all. This means the verified `wait()` does not model the original's semantics: in the original, `wait()` is called *before* the fence is satisfied (by the calling thread's perspective), and it blocks until concurrent signalers fulfill it. The verification does not capture the fundamental purpose of `wait()`.
- **Suggested Fix:** This is an inherent limitation of sequential verification for a concurrent primitive. Document this more prominently as a verification gap. Alternatively, model `wait()` with only `self.wf()` as a precondition (not `spec_is_satisfied`) and use `ensures self.spec_is_satisfied()` with an `assume` on the concurrency assumption, making the trust boundary explicit rather than hidden in the precondition.

### Medium

- **Location:** `signal()` in fence.rs (exec), line 166
- **Description:** The original `signal(&self)` uses `&self` with `AtomicUsize::fetch_add`. The verified `signal(&mut self)` uses `&mut self`. This is a necessary modeling choice for Verus, but it changes the API contract significantly: `&mut self` implies exclusive access, which means the verified model cannot represent the concurrent case where multiple threads call `signal()` simultaneously on the same fence. The original code explicitly permits this via atomic operations. The documentation acknowledges this but the implication is that the verification does not cover the actual concurrent signaling pattern.
- **Suggested Fix:** No code change needed, but this should be tracked as a known limitation. Consider adding a proof lemma that explicitly states the commutativity property of concurrent signals (i.e., the order of signal calls doesn't affect the final state), even if expressed at the spec level over `nat` arithmetic.

- **Location:** `spec_remaining()` in fence.spec.rs, line 66-68
- **Description:** The spec function `spec_remaining` computes `(self.total as nat - self.count as nat) as nat`. When `wf()` holds (`count <= total`), this is fine. However, without `wf()` as a precondition on callers, `spec_remaining` could underflow in the nat domain (though Verus nat subtraction saturates to 0). The function's semantics may be surprising: if `count > total` (violating `wf()`), remaining would be 0, not negative, which could mask bugs in specs that forget the `wf()` precondition.
- **Suggested Fix:** Add a comment noting that `spec_remaining` assumes `wf()` for meaningful results, or add a recommends clause: `recommends self.wf()`.

- **Location:** `lemma_total_signals_satisfies` in fence.proof.rs, line 117-120
- **Description:** This lemma claims to prove "liveness: if we start at count == 0 and signal `total` times, the fence becomes satisfied." However, the actual ensures clause is simply `total >= total`, which is a trivial tautology that does not actually express the liveness property. A meaningful liveness lemma would show that starting from a fence with `count == 0` and `total == t`, after exactly `t` applications of `signal`, `spec_is_satisfied()` holds. The current lemma proves nothing beyond `x >= x`.
- **Suggested Fix:** Replace with a lemma that takes a `Fence` (or count/total pair) and proves that `count == total ==> spec_is_satisfied()`, or better yet, an inductive lemma over signal count. For example:
  ```rust
  pub proof fn lemma_total_signals_satisfies(total: nat)
      ensures
          forall|count: nat| count == total ==> count >= total,
  {}
  ```
  Or more meaningfully, prove inductively that after `total` signal operations from `count == 0`, the fence is satisfied.

### Low

- **Location:** `is_satisfied()`, `get_count()`, `get_total()` in fence.rs (exec), lines 187-221
- **Description:** These three functions do not exist in the original `fence.rs`. They are utility accessors added for the verification model. While they are harmless and useful for the verification, they represent API additions not present in the original code. This is a minor equivalence divergence.
- **Suggested Fix:** Add a brief comment or section in the module doc noting these are verification-only accessors not present in the original runtime code.

- **Location:** `new()` in fence.rs (exec), line 115
- **Description:** The original `new` is `const fn`; the verified version is a plain `fn`. This is a minor divergence—Verus may not support `const fn`, so this is likely unavoidable.
- **Suggested Fix:** Document this as a known limitation if `const fn` is not supported in Verus.

- **Location:** Proof lemmas in fence.proof.rs
- **Description:** Several proof lemmas (`lemma_satisfaction_is_monotone`, `lemma_wait_on_satisfied_is_noop`, `lemma_signal_preserves_wf`, `lemma_last_signal_satisfies`) have trivially true ensures clauses that Verus automatically discharges. While they serve as documentation and regression tests, they don't exercise the prover in a meaningful way. For example, `lemma_satisfaction_is_monotone` proves `count >= total ==> count >= total` and `count + 1 >= total`, both of which are arithmetic tautologies.
- **Suggested Fix:** Consider consolidating trivial lemmas into fewer, more meaningful combined lemmas, or label them explicitly as "regression/documentation lemmas" in a section comment (which is partially done already).

## Positive Observations

- **Clean split structure:** The spec/proof/exec separation is well-organized. Specs in `fence.spec.rs` are clean and focused, proofs are in `fence.proof.rs`, and exec code in `fence.rs` is minimal.
- **No trust assumptions in code:** There are zero `assume`, `external_body`, `trusted`, or `#[verifier::external]` annotations. The verification is fully machine-checked with no escape hatches.
- **Thorough documentation:** The module-level documentation in `fence.rs` is excellent—it clearly explains the verification model, trust boundaries, API divergence, and scope limitations. This is a model for how to document verified code.
- **Complete verification:** All 22 verification conditions pass. The `wf()` invariant is preserved across all state transitions (`new` establishes it, `signal` preserves it).
- **Well-formedness invariant:** The `wf()` predicate (`count <= total`) correctly captures the reachable state invariant and is established by `new()` and preserved by `signal()`.
- **Complementarity proof:** The proof that `spec_is_satisfied` and `spec_is_waiting` are complementary under `wf()` is a good property to verify.
- **View type:** The `FenceView` abstraction with the `View` trait implementation provides a clean abstract view of the fence state.

## Summary

The Verus verification of `Fence` is a solid sequential model that correctly verifies the state machine protocol: `new()` establishes `wf()`, `signal()` preserves it while incrementing count, and the satisfaction/waiting predicates are complementary. The verification has no trust assumptions (no `assume`/`external_body`), which is commendable.

The main limitation is the modeling of `wait()`: by requiring `spec_is_satisfied()` as a precondition, `wait()` becomes a vacuous no-op that doesn't capture the original's blocking semantics. This is the most significant gap—the whole *point* of a fence is that `wait()` blocks until satisfaction is achieved by concurrent signalers. The sequential model necessarily cannot express this, but the current approach of making `wait()` require satisfaction as a precondition (rather than establishing it as a postcondition via an explicit concurrency assumption) hides the trust boundary.

The `lemma_total_signals_satisfies` is a tautology (`total >= total`) that doesn't prove the liveness property it claims. Several other lemmas are also trivial arithmetic facts, though they serve as regression tests.

Overall, this is a competent verification of a simple data structure's sequential protocol, with excellent documentation of its limitations. The grade reflects that the core functions are covered, specs are reasonable, and no unsound escape hatches are used, but the `wait()` modeling gap and trivial liveness lemma prevent a higher grade.
