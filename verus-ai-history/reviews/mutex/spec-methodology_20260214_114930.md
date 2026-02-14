# Review: mutex Spec Methodology (claude-opus-4.6)

## Grade: B+

## Issues Found

### Critical

- None.

### High

- **Public struct fields violate encapsulation (Criterion 4).** `Mutex` has `pub locked`, `pub id`, `pub token_issued` fields (mutex.rs:174-179). The guidelines (Step 3) state: "never talk directly about the fields of a `Self` parameter. For instance, don't say `self.x`; instead say things like `self@.y`." Several public method specs reference `self.locked` directly instead of `self@.locked`:
  - `try_lock` ensures `self.locked` (line 236) and `result.0 == !old(self).locked` (line 233).
  - `lock` ensures `self.locked` (line 276).
  - `unlock` requires `old(self).locked` (line 306), ensures `!self.locked` (line 312).
  - `is_locked` ensures `result == self.locked` (line 331).
  - `new` ensures `!result.locked` (line 199).

  Public method specs should use `self@.locked` (the view field) rather than `self.locked` (the concrete field). While they are equivalent due to the `view()` definition, the methodology requires specs to go through the view to maintain abstraction.

- **`MutexToken.view` is `pub ghost` instead of private (Criterion 4, Trust Assumption T3).** The `MutexToken` tracked struct exposes `pub ghost view: MutexView` (mutex.spec.rs:51). The module header documents this as Trust Assumption T3, acknowledging that external code could theoretically construct a token without calling `lock()`/`try_lock()`. The justification (Verus opaqueness rule requiring `pub open spec fn` bodies to reference only public fields) is documented but represents a known gap in encapsulation.

### Medium

- **`MutexView` uses concrete `bool` instead of abstract type (Criterion 1).** `MutexView.locked` is `bool` and `MutexView.token_issued` is `bool` (mutex.spec.rs:23,27). For booleans this is acceptable since `bool` has no abstract counterpart in Verus, but it is worth noting. The `id` field correctly uses `nat` (abstract) rather than `usize` (concrete), which is good.

- **`view()` is `pub open spec fn` instead of `pub closed spec fn` (Criterion 2).** The `view()` function is declared as `open spec fn` (mutex.spec.rs:113). However, this is justified in the comment (lines 106-109): the Verus `View` trait requires `open`. This is an acceptable deviation.

- **Helper spec functions `spec_is_locked`, `spec_is_unlocked`, `token_issued` are on `Mutex` not `MutexView` (Guideline Step 3).** The methodology states: "You may find yourself repeatedly writing the same subexpressions... create a `pub open spec fn` in `MyTypeView` for each such common expression." These helpers (mutex.spec.rs:60-66, 92-93) are defined on `Mutex` rather than `MutexView`. While functional, placing them on `MutexView` would better follow the methodology and reinforce the abstraction boundary.

- **`spec_is_locked`/`spec_is_unlocked` reference `self.locked` (concrete field) directly.** These spec functions (mutex.spec.rs:60-66) access `self.locked` instead of `self@.locked`. Since they are on `Mutex` (not `MutexView`), this accesses the concrete field. If moved to `MutexView`, they would naturally operate on the abstract type.

- **Proof lemmas reference concrete `Mutex` fields directly.** Multiple proof lemmas construct `Mutex { locked: ..., id: ..., token_issued: ... }` directly (e.g., mutex.proof.rs:82-84, 160-162, 240-241, 308-309). While proofs are internal and the methodology focuses on public specs, this tightly couples proofs to the concrete representation.

### Low

- **`wf()` is named `wf` instead of `inv` (Criterion 3).** The guidelines (Step 2) specify the invariant function should be named `inv`. The implementation uses `wf` (mutex.spec.rs:87). This is a minor naming deviation; the function is correctly `pub closed spec fn` and serves the right purpose. The `wf()` naming is used consistently and is acceptable if it is a project-wide convention.

- **`spec_new_view` is an additional public spec function beyond `view` and `wf` (Guideline Step 3).** The methodology states: "Don't write any further `pub` specification functions in `impl MyType` beyond `inv` and `view`." `spec_new_view` (mutex.spec.rs:97-99) is a public spec function on `Mutex` that goes beyond this. However, it operates on views (returns `MutexView`) and serves as a constructor spec, which is a reasonable extension.

## Verification

- **Verification passes**: 27 verified, 0 errors. No `assume`, `admit`, or unjustified `external_body` found (Criterion 6 satisfied).

## Summary

The mutex verification is well-structured with comprehensive documentation, thorough proof lemmas, and clean verification (27/27 pass, no assumes/admits). The primary methodology gap is that **public method specs reference concrete fields (`self.locked`, `self.id`) rather than going through the view (`self@.locked`, `self@.id`)**, which violates the abstraction boundary prescribed by Step 3 of the methodology. This is the most significant issue because it couples callers to the concrete representation. The `MutexToken` public field is a documented Verus limitation (T3). The `wf()` naming (vs `inv()`) and helper function placement are minor deviations. The `view()` being `open` is correctly justified by Verus trait requirements. Overall, the verification model is sound and well-documented, but the spec methodology adherence could be tightened by consistently using view-based references in public specs.
