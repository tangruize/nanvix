# Review: kpage Spec Methodology (claude-opus-4.6)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- **`PageAddress` lacks a View type (violates methodology Step 1).** `PageAddress` is a verified type with an `inv()` and spec functions, but has no `PageAddressView` abstraction and no `View` trait implementation. The guidelines require a `MyTypeView` abstraction for each verified type. Currently, its public method specs use `self.spec_raw_value()` and `self.spec_is_aligned()` directly instead of going through `self@`. This is a structural gap that should be addressed for consistency.
- **`PageAddress.raw_addr` is `pub` (violates struct encapsulation).** Per the guidelines, member fields should be private and accessed via getter/setter methods. The `pub raw_addr` field is directly accessed in public method ensures clauses (e.g., `page_address_eq` line 212: `result == (a.raw_addr == b.raw_addr)`), which leaks implementation details.
- **`page_address_eq` ensures clause references `a.raw_addr` directly (violates criterion 4).** Line 212: `result == (a.raw_addr == b.raw_addr)` exposes the internal field in a public function's specification. This should use `a.spec_raw_value()` or (once a View exists) `a@.raw_value()` instead.

### Medium
- **`PageAddress` spec functions are `pub open` but should arguably be `pub closed` or routed through a View.** `spec_raw_value()` (line 19), `spec_is_aligned()` (line 25), `spec_pte_index()` (line 34), and `spec_cmp()` (line 40) are all `pub open spec fn` on `PageAddress` directly. Per methodology Step 3, no further `pub` spec functions beyond `inv` and `view` should exist in `impl MyType`. These should either become methods on a `PageAddressView`, or be made private if only used internally.
- **`PageAddressEqSpec` trait methods are `open spec fn` without `pub`.** Lines 54 and 59: `open spec fn obeys_eq_spec()` and `open spec fn eq_spec()`. These are effectively public due to the trait being `pub`. This is acceptable as a workaround for vstd's PartialEq mechanism but should be documented as such.
- **`KernelPage::spec_pool_id()` is `pub closed spec fn` beyond `inv` and `view`.** Line 162: This is an additional public spec function on `KernelPage` beyond the two permitted by the methodology (`inv` and `view`). However, it delegates to `self@.pool_id()` and is a minor convenience. Consider removing it if unused.

### Low
- **`KernelPageView` fields are `pub`.** Lines 254-258: While View fields being `pub` is common in Verus for usability, the guidelines suggest using accessor methods (which do exist: `page_address()`, `frame_address()`, `pool_id()`). The `inv()` on `KernelPage` directly accesses `self@.page_addr`, `self@.frame_addr`, and `self@.pool_id` (the fields), but this is acceptable inside `inv()` since it's `closed`.
- **`view()` on `KernelPage` is `closed spec fn` but not declared `pub`.** Line 123: The `View` trait's `view()` is declared as `closed spec fn view(&self)` without `pub`. This is correct behavior since the trait makes it effectively public, but the methodology document says "pub closed spec fn". This is a non-issue in practice since the trait controls visibility.

### Info
- **`external_body` on `PageAddress::eq` (line 236) is justified.** The comment thoroughly explains why it's needed (vstd PartialEq trait limitations), and `page_address_eq()` + `lemma_page_address_eq_correct()` provide full verified coverage of the same logic. This is acceptable.
- **No `assume` or `admit` found.** Clean verification with 16 verified, 0 errors.
- **`KernelPageView` uses abstract types correctly.** All fields use `int` rather than concrete types like `usize`. ✓
- **`KernelPage.kframe` is private.** The struct field is not `pub`. ✓
- **All public `KernelPage` methods require `self.inv()` and ensure `result.inv()` where applicable.** ✓

## Summary

The `KernelPage` type follows the spec methodology well: it has a proper `KernelPageView` with abstract types, `view()` is `closed`, `inv()` is `pub closed`, and public method specs use `self@` to access view fields. Verification passes cleanly with 16 verified and 0 errors, no assumes or admits.

The main gaps are around `PageAddress`, which lacks a `View` type entirely, has a `pub` field (`raw_addr`), and has multiple `pub open spec fn` helpers directly on the impl rather than on a View type. The `page_address_eq` function also leaks the internal field `raw_addr` in its ensures clause. These issues don't affect soundness (verification still passes), but they violate the encapsulation and abstraction principles from the methodology guidelines, making future refactoring of `PageAddress` internals harder.

Overall, the `KernelPage` side is well-done (A-level), but `PageAddress` drags the grade down to B+ due to structural methodology violations.
