# Exec Consistency Fix: kpool

## Summary
- Mismatches fixed: 0 (all 5 are documented Verus limitations)
- Missing functions added: 1 (`alloc_many` [alloc_many.diff](alloc_many.diff) | [alloc_many_source.rs](alloc_many_source.rs) | [alloc_many_verus.rs](alloc_many_verus.rs))
- Documented equivalences: 14 (5 mismatches + 5 missing + 4 extra groups)

## Changes

| Function | Action | Justification |
|----------|--------|---------------|
| `KernelFrame` [struct_KernelFrame.diff](struct_KernelFrame.diff) | [struct_KernelFrame_source.rs](struct_KernelFrame_source.rs) | [struct_KernelFrame_verus.rs](struct_KernelFrame_verus.rs) struct | Documented equivalence | Field `base` [base.diff](base.diff) | [base_source.rs](base_source.rs) | [base_verus.rs](base_verus.rs) renamed to `addr`; `Rc<RefCell<KpoolInner>>` replaced by `pool_id: usize` because Verus cannot model interior mutability or reference-counted shared ownership. |
| `Kpool` [struct_Kpool.diff](struct_Kpool.diff) | [struct_Kpool_source.rs](struct_Kpool_source.rs) | [struct_Kpool_verus.rs](struct_Kpool_verus.rs) struct | Documented equivalence | `Rc<RefCell<KpoolInner>>` replaced by `FrameAllocator` + `pool_id` [pool_id_verus.rs](pool_id_verus.rs) because Verus cannot model `Rc<RefCell>`. `KpoolInner` [struct_KpoolInner_source.rs](struct_KpoolInner_source.rs)'s `Bitmap` logic is absorbed into `FrameAllocator`. |
| `KpoolInner` [struct_KpoolInner_source.rs](struct_KpoolInner_source.rs) struct | Documented (MISSING) | Absorbed into `FrameAllocator`. `TruncatedMemoryRegion` + `Bitmap` are abstracted away. Verus cannot model `Rc<RefCell<KpoolInner>>`. |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | Documented equivalence | Original takes `TruncatedMemoryRegion<PhysicalAddress>`, creates `Bitmap` + `Rc<RefCell<KpoolInner>>`. Verus takes `FrameAllocator` + `pool_id` [pool_id_verus.rs](pool_id_verus.rs) (pre-constructed). Same bitmap-based tracking; only construction interface differs. |
| `alloc` [alloc.diff](alloc.diff) | [alloc_source.rs](alloc_source.rs) | [alloc_verus.rs](alloc_verus.rs) | Documented equivalence | Original has `clear: bool` param that calls `kframe.clear()` via unsafe `DerefMut`. Verus omits it: clearing requires `unsafe { from_raw_parts_mut }` which Verus cannot verify. Allocation logic (bitmap alloc + KernelFrame wrap) is identical. |
| `alloc_range` [alloc_range.diff](alloc_range.diff) | [alloc_range_source.rs](alloc_range_source.rs) | [alloc_range_verus.rs](alloc_range_verus.rs) | Documented equivalence | Original `KpoolInner::alloc_range(count)` searches + allocates via `bitmap.alloc_range`. Verus splits into `alloc_contiguous(count)` (search) + `alloc_range(start, count)` (book). Combined behavior of `alloc_contiguous` [alloc_contiguous_verus.rs](alloc_contiguous_verus.rs) is functionally equivalent. |
| `base` [base.diff](base.diff) | [base_source.rs](base_source.rs) | [base_verus.rs](base_verus.rs) | Documented equivalence | Original: `self.base`. Verus: `self.addr`. Field renamed in struct; function semantics identical (returns frame physical address). |
| `free` [free.diff](free.diff) | [free_source.rs](free_source.rs) | [free_verus.rs](free_verus.rs) | Documented equivalence | Original `KpoolInner::free(addr)` computes index and calls `bitmap.clear`. Verus `Kpool::free(kframe)` takes `KernelFrame` [struct_KernelFrame.diff](struct_KernelFrame.diff) | [struct_KernelFrame_source.rs](struct_KernelFrame_source.rs) | [struct_KernelFrame_verus.rs](struct_KernelFrame_verus.rs) for provenance checking. Core logic identical (mark frame free in bitmap). Original achieves pool association via `Rc<RefCell>` identity; Verus uses explicit `pool_id` [pool_id_verus.rs](pool_id_verus.rs) match. |
| `alloc_many` [alloc_many.diff](alloc_many.diff) | [alloc_many_source.rs](alloc_many_source.rs) | [alloc_many_verus.rs](alloc_many_verus.rs) | **Added** | Added as wrapper around `alloc_contiguous()`. Matches original `Kpool::alloc_many()` contiguous allocation semantics. `clear` [clear_source.rs](clear_source.rs) param omitted (Verus limitation: requires unsafe `DerefMut`). Returns `usize` start index instead of `Vec<KernelFrame>` (Verus limitation: cannot model `Rc<RefCell>` cloning). |
| `clear` [clear_source.rs](clear_source.rs) | Documented (MISSING) | Requires `DerefMut` which uses `unsafe { from_raw_parts_mut }`. Verus cannot verify unsafe code. Clearing is orthogonal to allocation safety. |
| `deref` [deref_source.rs](deref_source.rs) | Documented (MISSING) | Requires `unsafe { core::slice::from_raw_parts }`. Verus cannot verify unsafe code. Memory content access is beyond allocation safety scope. |
| `deref_mut` [deref_mut_source.rs](deref_mut_source.rs) | Documented (MISSING) | Requires `unsafe { core::slice::from_raw_parts_mut }`. Verus cannot verify unsafe code. |
| `drop` [drop_source.rs](drop_source.rs) | Documented (MISSING) | Requires `Rc<RefCell<KpoolInner>>` for RAII deallocation. Verus cannot model `Rc<RefCell>`. Explicit `free()` replaces `Drop`. |
| `new_internal` [new_internal_verus.rs](new_internal_verus.rs) | Justified extra | KernelFrame constructor extracted for verification. Needed because `KernelFrame::new` in original takes `Rc<RefCell<KpoolInner>>` which Verus cannot model. |
| `address` [address_verus.rs](address_verus.rs), `pool_id` [pool_id_verus.rs](pool_id_verus.rs) | Justified extras | Getters for new struct fields (`addr`, `pool_id` [pool_id_verus.rs](pool_id_verus.rs)) replacing `Rc<RefCell>` identity. |
| `capacity` [capacity_verus.rs](capacity_verus.rs), `get_pool_id` [get_pool_id_verus.rs](get_pool_id_verus.rs) | Justified extras | Pool accessors needed for verification specs. Original accesses these via `KpoolInner` [struct_KpoolInner_source.rs](struct_KpoolInner_source.rs) through `Rc<RefCell>`. |
| `alloc_contiguous` [alloc_contiguous_verus.rs](alloc_contiguous_verus.rs) | Justified extra | Implements the search-and-allocate logic from original `KpoolInner::alloc_range()`. `alloc_many` [alloc_many.diff](alloc_many.diff) | [alloc_many_source.rs](alloc_many_source.rs) | [alloc_many_verus.rs](alloc_many_verus.rs) wrapper provides original API name. |
| `alloc_noncontiguous` [alloc_noncontiguous_verus.rs](alloc_noncontiguous_verus.rs) | Justified extra | Additional verification API for non-contiguous allocation. Not present in original but provides useful verified primitive. |
| `free_range` [free_range_verus.rs](free_range_verus.rs) | Justified extra | Batch deallocation helper. Original relies on `Drop` for each `KernelFrame` [struct_KernelFrame.diff](struct_KernelFrame.diff) | [struct_KernelFrame_source.rs](struct_KernelFrame_source.rs) | [struct_KernelFrame_verus.rs](struct_KernelFrame_verus.rs) in `Vec`. This replaces RAII batch cleanup. |
| `free_contiguous` [free_contiguous_verus.rs](free_contiguous_verus.rs) | Justified extra | Extends `free_range` [free_range_verus.rs](free_range_verus.rs) with ghost index validation for bridging `alloc_noncontiguous` [alloc_noncontiguous_verus.rs](alloc_noncontiguous_verus.rs) results. |

## Verus Limitations Summary

The following Rust features used in the original cannot be modeled in Verus:

1. **`Rc<RefCell<T>>`**: Interior mutability with reference counting. Used for shared pool access and RAII `Drop`. Replaced by explicit `pool_id` [pool_id_verus.rs](pool_id_verus.rs) provenance tracking and explicit `free()` calls.
2. **`unsafe` blocks**: `from_raw_parts` / `from_raw_parts_mut` for byte-level memory access. Required by `Deref`, `DerefMut`, and `clear` [clear_source.rs](clear_source.rs). Orthogonal to allocation safety.
3. **`Drop` trait with shared state**: RAII deallocation requires `Rc<RefCell>`. Replaced by explicit `free()` with provenance precondition.
4. **`TruncatedMemoryRegion`**: Region type abstracted into `FrameAllocator` constructor.

## Verification: PASS
- Before: 33 verified, 0 errors
- After: 34 verified, 0 errors (+1 from added `alloc_many` [alloc_many.diff](alloc_many.diff) | [alloc_many_source.rs](alloc_many_source.rs) | [alloc_many_verus.rs](alloc_many_verus.rs))
