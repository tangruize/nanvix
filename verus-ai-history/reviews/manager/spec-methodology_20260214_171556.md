# Review: manager Spec Methodology (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- **Two `#[verifier::external_body]` functions without full justification**: `alloc_kpages()` (line 470) and `alloc_upages()` (line 522) are both marked `external_body` with `unimplemented!()` bodies. Per the guidelines (Step 5), these should be removed before declaring success. The doc comments note these are "specification-level functions" and suggest using the single-allocation variants in a loop, but neither a loop-based implementation nor a formal justification for leaving them as external_body is provided. These represent unverified trusted specifications.

### Medium
- **Public method specs reference `vmem.mapping_count` directly instead of via view**: Multiple public methods (`alloc_upage`, `unmap_upage`, `ctrl_upage`, `alloc_upages`, `new_vmem`) reference `vmem.mapping_count` directly in their `ensures` clauses (lines 220, 277, 350, 403, 549, 551). Per the guidelines (Step 3), public method specs should avoid direct field access and use `self@.field` or the type's view instead. However, `Vmem` does not currently implement a `View` trait, and `mapping_count` is a `pub` field on `Vmem`. This is a `Vmem`-side design issue that leaks into manager's specs — manager cannot use `vmem@.mapping_count` when `Vmem` has no view. This is noted as medium because the violation originates in `Vmem`'s design, not manager's.
- **`new_vmem()` missing `self.inv()` in ensures**: The `new_vmem()` method takes `&self` and requires `self.inv()`, but does not ensure `self.inv()` in the postcondition (line 218-220). While `&self` is immutable so `inv()` is trivially preserved, the guidelines state that output `Self` parameters should have `inv()` in ensures. For `&self` this is arguably unnecessary but inconsistent with other methods.

### Low
- **`VirtMemoryManagerView` helper functions are `pub open spec fn`**: The functions `has_kpool_capacity`, `has_upool_capacity`, `has_kpool_capacity_for`, `has_upool_capacity_for`, and `pools_valid` on `VirtMemoryManagerView` (lines 39-66 of spec file) are correctly `pub open spec fn` per guidelines Step 3 — common subexpressions on the view type should be `pub open spec fn`. This is correct.
- **`spec_uframe_is_allocated` is `pub closed spec fn` on `VirtMemoryManager`**: This helper (line 113 of spec file) is used in `unmap_upage()` preconditions. Per the guidelines, public methods should avoid calling `Self` spec functions other than `inv` and `view`. However, this function encodes provenance tracking that cannot be expressed through the view alone (it references `self.upool@`). The `closed` visibility correctly hides internals. This is a minor deviation justified by the need to express frame ownership in preconditions.
- **`pools_valid()` defined but not used in `inv()`**: The `VirtMemoryManagerView::pools_valid()` spec function (line 63) captures pool count bounds but is not referenced by `VirtMemoryManager::inv()`. The invariant delegates entirely to sub-pool invariants (`self.kpool.inv()` and `self.upool.inv()`), which likely imply these bounds. Not a bug, but potentially dead code.

## Summary

The manager module follows the spec methodology guidelines well overall. The `VirtMemoryManagerView` uses abstract types (`int`) correctly, `view()` is `pub closed spec fn`, and `inv()` is `pub closed spec fn`. Public method specs properly use `self@.field` for the manager's own state. The main concerns are: (1) two `external_body` functions that represent unverified trusted specs with no implementation, and (2) direct field access to `vmem.mapping_count` in public specs, though this is forced by `Vmem` lacking a `View` type. Verification passes cleanly with 10 verified, 0 errors. No `assume` or `admit` statements are present.
