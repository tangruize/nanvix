# Exec Consistency Fix: kheap

## Summary
- Mismatches fixed: 0 (all 4 are necessary Verus adaptations, documented)
- Missing functions added: 0 (all 3 require Verus-incompatible types, documented)
- Documented equivalences: 11

## Changes

| Function | Action | Justification |
|----------|--------|---------------|
| `from_raw_parts` [from_raw_parts.diff](from_raw_parts.diff) [from_raw_parts_source.rs](from_raw_parts_source.rs) [from_raw_parts_verus.rs](from_raw_parts_verus.rs) | DOCUMENTED | Verus cannot reason about `*mut u8` pointer arithmetic (`heap_start_addr.add()`). Uses `Slab::from_raw_parts_at_offset` with integer offsets instead — semantically identical (both create slabs at `addr + i * slab_size`). Runtime validation checks moved to preconditions (standard Verus pattern). `info!`/`error!` logging omitted (unavailable in Verus). Ghost fields `base_addr`/`total_size` added for specification (no runtime effect). |
| `allocate` [allocate.diff](allocate.diff) [allocate_source.rs](allocate_source.rs) [allocate_verus.rs](allocate_verus.rs) | DOCUMENTED | Parameter changed from `Layout` to `usize` because Verus cannot model `core::alloc::Layout`. Return type changed from `Result<*mut u8, AllocError>` to `Result<usize, Error>` because Verus cannot model raw pointer types or `AllocError`. The `?` operator is replaced by explicit match (Verus limitation). Core dispatch logic (select slab → call `slab.allocate()`) is identical. |
| `deallocate` [deallocate.diff](deallocate.diff) [deallocate_source.rs](deallocate_source.rs) [deallocate_verus.rs](deallocate_verus.rs) | DOCUMENTED | Same adaptations as `allocate`: `(*mut u8, Layout)` → `(usize, usize)`, `AllocError` → `Error`. Core dispatch logic (select slab → call `slab.deallocate(addr)`) is identical. |
| `init` [init.diff](init.diff) [init_source.rs](init_source.rs) [init_verus.rs](init_verus.rs) | DOCUMENTED | Original takes no parameters, reads from `static mut HEAP_STORAGE`, stores result in `static mut HEAP`. Verus cannot model static mutable state. Changed to take explicit `(addr, size)` parameters and return `Result<Kheap, Error>`. The executable logic is identical: delegates to `Kheap::from_raw_parts(addr, size)`. |
| `alloc` [alloc_source.rs](alloc_source.rs) | NOT ADDED | `GlobalAlloc::alloc` on `ArenaAllocator` requires: (1) `GlobalAlloc` trait implementation, (2) `static mut HEAP` access, (3) `*mut u8` return type. None of these can be modeled in Verus. This is a thin wrapper that delegates to `Kheap::allocate`, which IS verified. |
| `dealloc` [dealloc_source.rs](dealloc_source.rs) | NOT ADDED | Same as `alloc` — `GlobalAlloc::dealloc` requires trait implementation + static mutable state + raw pointers. Delegates to `Kheap::deallocate`, which IS verified. |
| `layout_to_allocator` [layout_to_allocator_source.rs](layout_to_allocator_source.rs) | NOT ADDED | Replaced by `layout_to_slab_size`. The match logic is identical (same 8 arms: `1..=8 → Slab8`, ..., `4096 → Slab4096`, `_ → Err`). Only differences: parameter `&Layout` → `usize` (original only uses `layout.size()`), error type `AllocError` → `Error`, function is standalone instead of `impl Kheap` method. Documented equivalence in code comment. |
| `as_usize` [as_usize_verus.rs](as_usize_verus.rs) | KEPT | Justified helper: original uses `SlabSize::Variant as usize` via `#[repr(usize)]` casting. Verus cannot reason about `repr` casts, so this explicit conversion is needed to bridge spec (`spec_as_int`) and exec. |
| `layout_to_slab_size` [layout_to_slab_size_verus.rs](layout_to_slab_size_verus.rs) | KEPT | Justified replacement for `layout_to_allocator` — identical match logic with Verus-compatible types. See `layout_to_allocator` row above. |
| `test_layout_to_slab_size_verified` [test_layout_to_slab_size_verified_verus.rs](test_layout_to_slab_size_verified_verus.rs) | KEPT | Verification test exercising `layout_to_slab_size` for all size categories and boundary values. |
| `test_slab_size_as_usize_verified` [test_slab_size_as_usize_verified_verus.rs](test_slab_size_as_usize_verified_verus.rs) | KEPT | Verification test exercising `SlabSize::as_usize` for all variants. |
| `Kheap` struct | DOCUMENTED | Two ghost fields (`base_addr: Ghost<int>`, `total_size: Ghost<int>`) added for spec. Ghost fields are erased at compile time and have no runtime representation. Original 8 slab fields are preserved unchanged. |
| `ArenaAllocator` struct | NOT ADDED | ZST implementing `GlobalAlloc` trait — cannot be modeled in Verus. |
| `HeapStorage` struct | NOT ADDED | `#[repr(align(4096))]` static buffer used with `static mut` — Verus cannot model static mutable state or repr alignment attributes. |

## Verification: PASS

```
verification results:: 43 verified, 0 errors
Duration: 10s
```

## Notes

All 4 MISMATCH functions and 3 MISSING functions involve Verus-incompatible constructs:
- **Raw pointers** (`*mut u8`, pointer arithmetic via `.add()`): Verus uses integer addresses instead.
- **`core::alloc::Layout`**: Opaque stdlib type with no Verus model. Size is passed as `usize` directly.
- **`core::alloc::AllocError`**: Not available in Verus; `Error` is used instead.
- **`GlobalAlloc` trait**: Requires trait implementation for extern type + `unsafe impl` — not verifiable.
- **Static mutable state** (`static mut HEAP`, `HEAP_STORAGE`, `ALLOCATOR`): Verus cannot model mutable globals.
- **`#[repr(usize)]` casts**: Verus cannot reason about discriminant-to-integer casts; `as_usize()` helper used instead.

The core algorithmic logic verified in Verus (slab selection, allocation dispatch, deallocation dispatch, heap initialization) is semantically equivalent to the original. The unverified portions (`alloc`, `dealloc`, `init`'s static storage) are thin wrappers around the verified core.
