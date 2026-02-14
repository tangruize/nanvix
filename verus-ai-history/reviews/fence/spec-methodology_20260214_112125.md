# Review: fence Spec Methodology (claude-opus-4.6)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- **Public spec functions beyond `inv` and `view` are `pub open` instead of `pub closed`.**
  The guidelines (Step 3) state: "Don't write any further `pub` specification functions in `impl MyType` beyond `inv` and `view`." However, `spec_is_satisfied`, `spec_is_waiting`, `spec_count`, `spec_total`, `spec_remaining`, and `spec_new_view` are all `pub open spec fn` on `impl Fence` (fence.spec.rs lines 46–80). These expose implementation details (e.g., `self.count as nat`) to callers. Per the guidelines, if common subexpressions are needed in public method specs, they should be placed as `pub open spec fn` on `FenceView`, not on `Fence`. Currently they reference `self.count` and `self.total` (concrete fields) rather than `self@.count` and `self@.total` (view fields), which leaks the concrete representation through the spec layer. The guideline-compliant approach would be to either (a) move these helpers onto `FenceView` and have them reference view fields, or (b) make them private to `Fence` and only expose `inv` and `view` publicly.

- **Public method specs reference `self.count` and `self.total` (concrete fields) directly.**
  The guidelines (Step 3) say: "never talk directly about the fields of a `Self` parameter. For instance, don't say `self.x`; instead say things like `self@.y`." Multiple ensures clauses in public methods violate this:
  - `new()` ensures: `result.count == 0`, `result.total == total` (fence.rs lines 150–151).
  - `wait()` ensures: `self.count as nat >= self.total as nat` (fence.rs line 182).
  - `signal()` ensures: `self.count == old(self).count + 1`, `self.total == old(self).total` (fence.rs lines 204–205).
  - `is_satisfied()` ensures: `result == (self.count as nat >= self.total as nat)` (fence.rs line 229).
  - `get_count()` ensures: `result == self.count` (fence.rs line 245).
  - `get_total()` ensures: `result == self.total` (fence.rs line 261).

  These should use `self@.count` and `self@.total` instead, routing through the `FenceView` abstraction.

### Medium
- **`view()` is `open spec fn` instead of `pub closed spec fn`.**
  The guidelines (Step 1) prescribe `pub closed spec fn view`. The code (fence.spec.rs line 94) uses `open spec fn` with a comment explaining the Verus `View` trait requires `open`. This is a justified divergence — the trait constraint is a genuine Verus limitation — and the comment at line 87–89 documents it. However, since the fields of `Fence` are also `pub` (line 123–127 of fence.rs), the combination of `open view()` + public fields means the abstraction barrier is fully transparent. If Verus were to relax the trait constraint, `view()` should be made `closed`.

- **Struct fields are `pub` instead of private.**
  The `Fence` struct (fence.rs lines 122–127) has `pub count` and `pub total`. The comment says this is "required by Verus for `pub open spec fn` access." This is a Verus limitation workaround, but it undermines the abstraction boundary that the View pattern is designed to provide. If the `pub open` spec fns on `Fence` were moved to `FenceView` or made `closed`, the fields could potentially be made private.

- **`inv()` and `wf()` naming.**
  The guidelines mention both `inv()` and `wf()`. The code uses `inv()` as `pub closed spec fn` (fence.spec.rs line 41), which is correct. There is no separate `wf()` — this is acceptable since `inv()` serves the well-formedness role. ✓

### Low
- **`is_satisfied` and `get_count`/`get_total` are not in the original API.**
  These are verification-only accessors (documented as such). While not a spec methodology issue per se, they expand the public API surface beyond the original `Fence`. This is minor since they are clearly documented.

- **Redundant ensures in `new()`.**
  `new()` has 8 ensures clauses (fence.rs lines 150–157), several of which are redundant given the view (e.g., `result.count == 0` is redundant with `result@ == Fence::spec_new_view(total as nat)`). This is not incorrect but adds maintenance burden.

## Checklist

| # | Criterion | Status | Notes |
|---|-----------|--------|-------|
| 1 | View uses abstract types (int, Seq, Set, Map) | ✅ Pass | `FenceView` uses `nat` for both fields |
| 2 | `view()` is `pub closed spec fn` (or justified) | ⚠️ Justified | `open` required by Verus `View` trait; documented |
| 3 | `inv()`/`wf()` exists as `pub closed spec fn` | ✅ Pass | `inv()` is `pub closed spec fn` |
| 4 | Public method specs avoid `self.field` | ❌ Fail | Multiple ensures use `self.count`, `self.total` directly |
| 5 | Public methods require/ensure `inv()` | ✅ Pass | All public methods include `inv()` in requires/ensures |
| 6 | No remaining assume/admit/unjustified external_body | ✅ Pass | None found |
| 7 | Verification passes | ✅ Pass | 24 verified, 0 errors |

## Summary

The fence verification is solid in terms of correctness — all 24 verification conditions pass, there are no `assume`/`admit`/`external_body` usages, and `inv()` is properly threaded through all public methods. The `FenceView` type correctly uses `nat` (abstract) instead of `usize` (concrete).

The main methodology gap is that public method specifications directly reference concrete struct fields (`self.count`, `self.total`) instead of routing through the view (`self@.count`, `self@.total`). This is compounded by having multiple `pub open spec fn` helpers on `impl Fence` that also reference concrete fields, contrary to the guideline that only `inv()` and `view()` should be public spec functions on the implementation type. The struct fields being `pub` further erodes the abstraction barrier.

These are structural/methodology issues rather than correctness bugs — the proofs are valid and the verified properties are meaningful. Fixing the field-reference issue would require moving the helper spec fns to `FenceView` and rewriting ensures clauses to use `self@.field` syntax, which is a moderate refactor but would bring the code into full compliance with the specification methodology guidelines.
