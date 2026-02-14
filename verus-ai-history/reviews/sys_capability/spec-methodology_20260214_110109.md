# Review: sys_capability Spec Methodology (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

- None.

### High

- **`CapabilityView.value` field is `pub`**: The `value` field on `CapabilityView` (capability.spec.rs:23) is declared as `pub value: int`. Per the methodology guidelines (Step 1), internal fields that don't need user exposure should be hidden. For View types, `pub` fields are acceptable **only** if they are part of the intended abstract interface. For a simple enum discriminant this is borderline acceptable, but the methodology document recommends using `pub open spec fn` helpers on the View type for common expressions rather than exposing raw fields. Since users reference `self@.value` in specs, this is functionally fine, but documenting the deliberate choice would strengthen the design. **Severity: Low-High** — no correctness impact, minor encapsulation concern.

- **`inv()` uses `self.spec_discriminant()` instead of `self@.value`**: The `inv()` function (capability.spec.rs:111) is defined as `Self::spec_is_valid_discriminant(self.spec_discriminant())`. Since `inv()` is `closed`, this is technically acceptable — the implementation detail is hidden from callers. However, the methodology (Step 3) recommends specs avoid `self.field` in favor of `self@.field`. A more methodology-aligned formulation would be `self@.value >= 0 && self@.value <= 4` or `CapabilityView::is_valid_discriminant(self@.value)`. Since `inv()` is specifically a bridge between impl and spec domains and is `closed`, this is a minor style issue. **Severity: Low-High** — no correctness impact but deviates from methodology spirit.

### Medium

- **Redundant spec functions**: `CapabilityView::is_valid_discriminant(v)` and `Capability::spec_is_valid_discriminant(value)` (capability.spec.rs:37-39, 59-61) are identical functions on different types. Per the methodology (Step 3), common spec expressions should be consolidated on the View type. `Capability::spec_is_valid_discriminant` could be removed in favor of `CapabilityView::is_valid_discriminant`. Similarly, `CapabilityView::is_valid(&self)` and `CapabilityView::is_valid_discriminant(v)` overlap — `is_valid` could be defined in terms of `is_valid_discriminant(self.value)`.

- **Spec functions on `impl Capability` are `pub open`**: The methodology (Step 3) says "Don't write any further `pub` specification functions in `impl MyType` beyond `inv` and `view`." The file declares `spec_discriminant`, `spec_is_valid_discriminant`, and `spec_from_discriminant` as `pub open spec fn` on `impl Capability` (capability.spec.rs:48, 59, 66). These are used in public method `ensures` clauses. While `spec_from_discriminant` provides genuinely useful expressiveness in postconditions, `spec_discriminant` and `spec_is_valid_discriminant` should ideally be private or moved to the View type.

### Low

- **`view()` lacks `pub` keyword**: The `view()` function (capability.spec.rs:93) is declared as `closed spec fn view(...)` without `pub`. The methodology says it should be `pub closed spec fn`. Since this is a trait implementation (`impl View for Capability`), the visibility may be inherited from the trait, but explicitly marking it `pub` would better match the methodology guidance.

- **`CapabilityView` lacks `#[verifier::reject_recursive_types_in_ground_args]`**: This is a simple struct so there's no recursion issue, but some Verus conventions add this annotation. Not a real concern here.

## Verification Status

✅ **PASSED**: 14 verified, 0 errors.

## Trust Boundary

✅ No `assume()`, `admit()`, or unjustified `external_body` found in any of the three files.

## Criteria Checklist

| # | Criterion | Status | Notes |
|---|-----------|--------|-------|
| 1 | View type uses abstract types | ✅ Pass | `CapabilityView` uses `int` not `u32` |
| 2 | `view()` is `pub closed spec fn` | ⚠️ Minor | `closed spec fn` but missing explicit `pub` (inherited from trait) |
| 3 | `inv()` exists as `pub closed spec fn` | ✅ Pass | Correctly declared |
| 4 | Public specs use `self@.field` not `self.field` | ✅ Pass | `try_from_u32` and `to_u32` use `self@.value`; `inv()` uses `self.spec_discriminant()` but is `closed` |
| 5 | Public methods require/ensure `inv()` | ✅ Pass | `to_u32` requires `self.inv()`; `try_from_u32` ensures `result->Ok_0.inv()` |
| 6 | No remaining assume/admit/external_body | ✅ Pass | None found |
| 7 | Verification passes | ✅ Pass | 14 verified, 0 errors |

## Summary

The `sys_capability` verified split is well-structured and fully passing verification with 14 lemmas and no trust gaps. The View type correctly uses abstract `int`, `view()` is `closed`, `inv()` is `pub closed`, and public method specs properly use `self@.value`. The module is a clean example of the methodology applied to a simple enum type.

The main areas for improvement are minor: (1) consolidating the redundant `is_valid_discriminant` functions between `CapabilityView` and `Capability`, (2) making the extra `pub open spec fn` functions on `impl Capability` either private or moving them to the View type per Step 3 guidance, and (3) using `self@.value` inside `inv()` for consistency. None of these affect correctness or the trust boundary.
