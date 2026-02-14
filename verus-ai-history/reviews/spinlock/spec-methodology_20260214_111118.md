# Review: spinlock Spec Methodology (claude-opus-4.6)

## Grade: B+

## Issues Found

### Critical

- None.

### High

1. **View type uses concrete `bool` fields instead of abstract types (Criterion 1).**
   `SpinlockView` (spinlock.spec.rs:21-30) uses `pub locked: bool` and `pub token_issued: bool`. Per the methodology guide, View types should use abstract types (e.g., `int` instead of `i32`). While `bool` has no wider abstract counterpart (unlike `usize` → `nat`), the `bool` fields arguably leak the concrete representation. The `id` field correctly uses `nat` instead of `usize`. Given that `bool` is already maximally abstract for a two-valued domain, this is a minor deviation — but it should be explicitly documented as an intentional exception.

2. **`view()` is `pub open spec fn` instead of `pub closed spec fn` (Criterion 2).**
   The `View` trait implementation (spinlock.spec.rs:105-111) uses `open spec fn view()`. The methodology guide (Step 1) prescribes `pub closed spec fn view()`. The code includes a justification comment (line 101-104) explaining that the Verus `View` trait mandates `open`. This is a valid technical constraint — Verus's `View` trait signature requires `open` — so this is a **justified deviation**, not a bug. The justification is well-documented.

3. **Public method specs use `self.locked` (concrete field) instead of `self@.locked` (Criterion 4).**
   Multiple public methods reference concrete fields directly in their specs:
   - `new()` ensures: `!result.locked` (spinlock.rs:140) — should be `!result@.locked`.
   - `try_lock()` ensures: `result.0 == !old(self).locked`, `self.locked` (spinlock.rs:175, 178) — should use `self@.locked` and `old(self)@.locked`.
   - `lock()` ensures: `self.locked` (spinlock.rs:220) — should be `self@.locked`.
   - `unlock()` requires: `old(self).locked` (spinlock.rs:248); ensures: `!self.locked` (spinlock.rs:253) — should use view accessors.
   - `is_locked()` ensures: `result == self.locked` (spinlock.rs:276) — should be `result == self@.locked`.

   The methodology guide (Step 3) is explicit: "never talk directly about the fields of a `Self` parameter. For instance, don't say `self.x`; instead say things like `self@.y`." The existing `spec_is_locked()` and `spec_is_unlocked()` helpers already exist and should be used instead of `self.locked`.

4. **Public method specs reference `self.token_issued()` which accesses `self@.token_issued` — acceptable but `spec_is_locked`/`spec_is_unlocked` are inconsistently used (Criterion 4).**
   Some ensures clauses mix `self.locked` with `self.spec_is_locked()` in the same method (e.g., `unlock()` at lines 252-254: `old(self).spec_is_locked()` alongside `!self.locked`). This inconsistency makes specs harder to read and maintain. All public specs should uniformly use the view-based predicates.

### Medium

5. **`spec_is_locked` and `spec_is_unlocked` are `pub open` and access `self.locked` (concrete field) directly (Criterion 4).**
   These spec functions (spinlock.spec.rs:61-67) are defined as `pub open spec fn` and reference `self.locked` — the concrete struct field, not `self@.locked`. Per the methodology guide, public spec functions beyond `inv()` and `view()` should be avoided on `impl MyType`. These would be better placed on `SpinlockView` (e.g., `impl SpinlockView { pub open spec fn is_locked(&self) -> bool { self.locked } }`), which would also resolve the concrete-field-access issue since `SpinlockView.locked` is already the abstract representation.

6. **`token_issued()` is `pub open spec fn` on `impl Spinlock` (Criterion 3/4).**
   The methodology guide says "Don't write any further `pub` specification functions in `impl MyType` beyond `inv` and `view`." `token_issued()` (spinlock.spec.rs:87-89) is a public spec function on `Spinlock`. It correctly accesses `self@.token_issued` (via the view), which is good. But per the guide, it should either be private or moved to `impl SpinlockView`.

7. **`spec_new_view()` is `pub open spec fn` on `impl Spinlock` (Criterion 3/4).**
   Same concern as above — `spec_new_view()` (spinlock.spec.rs:92-94) is an additional public spec function beyond `inv` and `view`. This is a constructor-like spec function and could be a standalone spec function or placed on `impl SpinlockView`.

8. **Struct fields are `pub` (spinlock.rs:113-121).**
   All fields of `Spinlock` (`locked`, `id`, `token_issued`) are `pub`. The comment at lines 107-110 explains this is required by Verus for `pub open spec fn` access. However, per the Nanvix coding guidelines, "Member fields in `struct`s must be private and accessed via getter/setter methods." If the spec helpers were moved to `SpinlockView` and `view()` could be made `closed`, the fields could potentially be made private. Given the Verus constraint, this is a known limitation.

### Low

9. **`inv()` is correctly `pub closed spec fn` (Criterion 3) — no issue.** ✓

10. **No `assume`, `admit`, or unjustified `external_body` found (Criterion 6).** ✓

11. **Verification passes: 23 verified, 0 errors (Criterion 7).** ✓

## Summary

The spinlock verification is functionally sound — all 23 verification conditions pass, the invariant is correctly formulated as `pub closed spec fn`, there are no `assume`/`admit`/`external_body` trust gaps, and the `LockToken` ghost protocol correctly models the RAII guard pattern. The documentation is thorough, covering verification scope, trust boundaries, and API divergences.

The main methodology gaps are in **public spec abstraction discipline**. Public method specs frequently reference concrete struct fields (`self.locked`) instead of view-based accessors (`self@.locked` or `self.spec_is_locked()`), violating the methodology guide's core principle of hiding implementation details from public specifications. Additionally, several public spec functions (`spec_is_locked`, `spec_is_unlocked`, `token_issued`, `spec_new_view`) are defined on `impl Spinlock` rather than on `impl SpinlockView`, contrary to the guide's directive to limit `impl MyType` public spec functions to `inv` and `view`. The `view()` being `open` is a justified Verus trait constraint.

These are methodology hygiene issues rather than soundness bugs — the proofs are correct and the invariant is strong. Addressing them would improve the spec's resilience to implementation changes and align with the project's verification methodology.
