# Review: pid Spec Methodology (claude-opus-4.6)

## Grade: A

## Issues Found

### Critical
- None.

### High
- **`value` field is `pub` in exec struct (pid.rs:66):** The `ProcessIdentifier.value` field is declared `pub` in the Verus exec code. The methodology guidelines (specifying-and-proving-types.md) say the View type should "hide internal fields" and public method specs should "never talk directly about the fields of a `Self` parameter." While the comment at line 59-61 explains this is done "for Verus spec reasoning," the original source uses a private tuple field `ProcessIdentifier(i32)`. Making the field `pub` weakens encapsulation — callers can bypass the API and access `self.value` directly in exec code instead of going through `self@.value` in specs. This is mitigated by the fact that all public method specs properly use `self@.value` (the view), but ideally the field would be private or `pub(crate)`.

### Medium
- **`ProcessIdentifierView` spec helper functions are `pub open` (pid.spec.rs:32-61):** Functions like `is_non_negative()`, `is_kernel()`, `is_initd()`, `in_i32_range()`, and `in_non_negative_i32_range()` are all `pub open spec fn` on the View type. Per the methodology (Step 3), common subexpressions on `MyTypeView` should indeed be `pub open spec fn`, so this is correct. No issue here — just noting these are appropriately open since they are on the View type, not the impl type.

### Low
- **Trivial `inv()` (pid.spec.rs:88-90):** The invariant is `true` — any i32 is a valid PID. This is correctly documented and justified: there are no structural invariants to maintain. While trivial, it follows the methodology correctly by declaring `pub closed spec fn inv()`.
- **`view()` visibility (pid.spec.rs:71):** The `view()` function is `closed spec fn` but lacks the `pub` keyword explicitly (it gets `pub` implicitly from the `View` trait). This is fine — it satisfies the "pub closed" requirement via the trait.

## Checklist Assessment

| # | Criterion | Status | Notes |
|---|-----------|--------|-------|
| 1 | View type uses abstract types (`int`) | ✅ Pass | `ProcessIdentifierView.value: int` — correctly uses `int` not `i32` |
| 2 | `view()` is `pub closed spec fn` | ✅ Pass | Declared as `closed spec fn` in `View` trait impl; public via trait |
| 3 | `inv()` exists and is `pub closed spec fn` | ✅ Pass | `pub closed spec fn inv(&self) -> bool` at pid.spec.rs:88 |
| 4 | Public method specs use `self@.field` not `self.field` | ✅ Pass | All ensures/requires use `self@.value`, `other@.value`, etc. |
| 5 | Public methods require/ensure `inv()` for self params | ✅ Pass | All public methods with `self` have `self.inv()` in requires and/or ensures; factory methods ensure `result.inv()` |
| 6 | No remaining assume/admit/unjustified external_body | ✅ Pass | All `external_body` uses are justified: byte serialization (3), layout assertions (2) — all with clear documentation of trust boundaries |
| 7 | Verification passes | ✅ Pass | 38 verified, 0 errors |

## Summary

The `ProcessIdentifier` Verus specification is well-structured and closely follows the methodology guidelines. The View type correctly abstracts `i32` to `int`, `view()` and `inv()` are properly `closed`, public method specs consistently use `self@.value` instead of `self.value`, and all public methods maintain the invariant. The five `external_body` annotations are well-justified trust boundaries for byte-level operations and layout assertions. The only notable issue is the `pub` visibility of the `value` field in the exec struct, which weakens encapsulation compared to the original private tuple field — though this is partially mitigated by correct spec-level usage. Verification passes cleanly with 38 items verified and 0 errors.
