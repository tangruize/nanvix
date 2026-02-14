# Review: vmem Spec Methodology (claude-opus-4.6)

## Grade: C

## Issues Found

### Critical

1. **No View type exists for `Vmem` or `PageMapping`**
   - The guidelines (Step 1) require a `VmemView` type using abstract types (`int`, `Seq`, `Set`, `Map`) instead of concrete types (`usize`, fixed-size arrays).
   - The doc comment at line 52 of `vmem.rs` mentions "VmemView is derived from the concrete mappings array" but no `VmemView` struct is defined anywhere.
   - No `view()` function exists (`pub closed spec fn view(&self) -> VmemView`).
   - `PageMapping` similarly lacks a `PageMappingView` with abstract types (e.g., `vaddr` and `frame_addr` should be `int`, not `usize`).
   - **Impact**: Without a View type, all public method specs are forced to reference internal fields directly, violating the abstraction principle.

2. **No `view()` function declared**
   - The guidelines require `pub closed spec fn view(&self) -> VmemView`.
   - No such function exists for `Vmem` or `PageMapping`.
   - All spec functions on `Vmem` (`spec_is_mapped`, `spec_get_frame_addr`, `spec_page_is_mapped`, `spec_user_region_is_mapped`, `has_mapping_capacity`, `has_mappings`) directly access `self.mappings` and `self.mapping_count` instead of working through `self@`.

### High

3. **Public method specs use `self.mapping_count` instead of `self@` abstraction**
   - Per the guidelines (Step 3), public method specifications must not talk directly about fields of a `Self` parameter. Instead of `self.mapping_count`, they should use `self@.field` (shorthand for `self.view().field`).
   - Affected public methods with `self.mapping_count` in requires/ensures:
     - `new()` — line 319: `result.mapping_count == 0`
     - `clone()` — line 397: `result.mapping_count == 0`
     - `map_kpage()` — line 513: `self.mapping_count == old(self).mapping_count`
     - `map()` — line 711: `self.mapping_count == old(self).mapping_count + 1`
     - `unmap()` — lines 817, 826: `self.mapping_count == old(self).mapping_count - 1`
     - `uctrl()` — line 974: `self.mapping_count == old(self).mapping_count`
     - `kctrl()` — line 1045: `self.mapping_count == old(self).mapping_count`
     - `memset()` — line 1273: `self.mapping_count == old(self).mapping_count`

4. **Struct fields are `pub` (should be private with getters/setters)**
   - `Vmem.mappings` and `Vmem.mapping_count` are `pub` (line 283–285).
   - `PageMapping.vaddr`, `PageMapping.frame_addr`, and `PageMapping.valid` are `pub` (lines 262–266).
   - Per coding standards: "Member fields in `struct`s must be private and accessed via getter/setter methods."
   - This directly enables the spec leak of internal fields into public method contracts.

5. **Multiple `pub open spec fn` on `Vmem` beyond `inv` and `view`**
   - The guidelines (Step 3) say: "Don't write any further `pub` specification functions in `impl MyType` beyond `inv` and `view`."
   - `Vmem` has 6 additional `pub open spec fn`: `spec_is_mapped`, `spec_get_frame_addr`, `spec_page_is_mapped`, `spec_user_region_is_mapped`. These should either be on `VmemView` (if a View type is created) or made private.
   - `has_mapping_capacity` and `has_mappings` are `pub closed spec fn` (acceptable as helpers, but per guidelines these should be on the View type or private).

6. **`PageMapping.spec_is_for_vaddr` is `pub open spec fn` exposing internals**
   - Line 26 of `vmem.spec.rs`: `pub open spec fn spec_is_for_vaddr(&self, vaddr: int) -> bool` directly references `self.valid` and `self.vaddr`.
   - This should be on a `PageMappingView` type or made private.

### Medium

7. **`inv()` for `Vmem` references concrete fields directly**
   - This is expected for `inv()` (it's `closed` so internals are hidden), but the invariant references `self.mappings` and `self.mapping_count` which are `pub` fields. Making fields private would improve encapsulation.

8. **`external_body` functions lack full justification documentation**
   - Five `external_body` functions exist: `load()`, `pgdir()`, `map_kpage()`, `kctrl()`, `copy_to_user_unaligned_unchecked()`.
   - While all have documentation explaining *why* they are `external_body`, some justifications could be stronger:
     - `pgdir()` (line 460): Returns a `usize` placeholder — the ensures clause is empty, providing no useful spec constraint.
     - `copy_to_user_unaligned_unchecked()` (line 1228): The ensures clause mirrors the checked version but the body is `unimplemented!()` — acceptable for a spec model but should be documented as needing refinement proof.

9. **`PageMapping.inv()` exists but is never required/ensured in public method contracts**
   - `PageMapping.inv()` is defined (line 17) but never appears in any public method's `requires` or `ensures` clauses. The individual invariant properties are instead captured within `Vmem.inv()`. While not incorrect (they're structurally redundant), it's an unused spec function.

### Low

10. **Free-standing spec functions are `pub open` (appropriate for this use case)**
    - `spec_is_user_addr`, `spec_is_kernel_addr`, `spec_is_user_region`, `spec_is_kernel_region`, `spec_is_physical_region` are all `pub open spec fn` at module scope.
    - These are standalone predicates (not on `Vmem`), so the guidelines about restricting `pub` spec fns to `inv`/`view` don't apply. This is fine.

## Positive Findings

- **No `assume` or `admit` statements** anywhere in the codebase — clean.
- **`inv()` is `pub closed spec fn`** — correctly follows guidelines.
- **`has_mapping_capacity()` and `has_mappings()` are `pub closed spec fn`** — proper encapsulation of helper predicates.
- **Verification passes**: 30 verified, 0 errors.
- **Comprehensive documentation** on all functions including abstraction decisions.
- **Proof file is clean** with useful lemmas about user/kernel space disjointness.
- **`external_body` usage is well-justified** for hardware operations (CR3, TLB, physical memory copy).

## Summary

The vmem module passes verification (30/0) and has no `assume`/`admit` statements, which is good. However, it has a fundamental structural gap: **no View type (`VmemView`) or `view()` function exists**, which is the cornerstone of the spec methodology guidelines. This absence cascades into multiple violations: public method specs directly reference concrete fields (`self.mapping_count`, `self.mappings`), struct fields are `pub` instead of private, and spec functions that should be on the View type are instead `pub open` on `Vmem` itself.

The `external_body` usage is appropriate and well-documented for hardware operations that cannot be modeled. The invariant is well-designed with uniqueness, alignment, and bounds properties.

To bring this module to full compliance, the primary work needed is:
1. Define `VmemView` with abstract types (e.g., `mapping_count: int`, mappings as `Seq` or `Map`).
2. Implement `pub closed spec fn view(&self) -> VmemView`.
3. Move public spec functions to `VmemView`.
4. Rewrite public method specs to use `self@` instead of `self.field`.
5. Make struct fields private.
