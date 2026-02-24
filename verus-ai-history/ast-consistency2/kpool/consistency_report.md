# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/mm/phys/kpool.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/mm/phys/kpool.rs`

## Summary

- Functions matched: 0/10
- Functions mismatched: 5
- Missing in Verus: 5
- Extra in Verus: 9
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `alloc` [alloc.diff](alloc.diff) | [alloc_source.rs](alloc_source.rs) | [alloc_verus.rs](alloc_verus.rs) | MISMATCH | 207-215 | 371-424 |
| `alloc_many` [alloc_many.diff](alloc_many.diff) | [alloc_many_source.rs](alloc_many_source.rs) | [alloc_many_verus.rs](alloc_many_verus.rs) | MISSING_IN_VERUS | 232-248 |  |
| `alloc_range` [alloc_range.diff](alloc_range.diff) | [alloc_range_source.rs](alloc_range_source.rs) | [alloc_range_verus.rs](alloc_range_verus.rs) | MISMATCH | 82-104 | 460-496 |
| `base` [base.diff](base.diff) | [base_source.rs](base_source.rs) | [base_verus.rs](base_verus.rs) | MISMATCH | 135-137 | 198-205 |
| `clear` [clear_source.rs](clear_source.rs) | MISSING_IN_VERUS | 144-148 |  |
| `deref` [deref_source.rs](deref_source.rs) | MISSING_IN_VERUS | 154-158 |  |
| `deref_mut` [deref_mut_source.rs](deref_mut_source.rs) | MISSING_IN_VERUS | 162-166 |  |
| `drop` [drop_source.rs](drop_source.rs) | MISSING_IN_VERUS | 170-174 |  |
| `free` [free.diff](free.diff) | [free_source.rs](free_source.rs) | [free_verus.rs](free_verus.rs) | MISMATCH | 107-117 | 808-834 |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISMATCH | 188-192 | 298-315 |
| `address` [address_verus.rs](address_verus.rs) | EXTRA_IN_VERUS |  | 169-176 |
| `alloc_contiguous` [alloc_contiguous_verus.rs](alloc_contiguous_verus.rs) | EXTRA_IN_VERUS |  | 531-599 |
| `alloc_noncontiguous` [alloc_noncontiguous_verus.rs](alloc_noncontiguous_verus.rs) | EXTRA_IN_VERUS |  | 636-782 |
| `capacity` [capacity_verus.rs](capacity_verus.rs) | EXTRA_IN_VERUS |  | 319-324 |
| `free_contiguous` [free_contiguous_verus.rs](free_contiguous_verus.rs) | EXTRA_IN_VERUS |  | 928-966 |
| `free_range` [free_range_verus.rs](free_range_verus.rs) | EXTRA_IN_VERUS |  | 857-893 |
| `get_pool_id` [get_pool_id_verus.rs](get_pool_id_verus.rs) | EXTRA_IN_VERUS |  | 328-332 |
| `new_internal` [new_internal_verus.rs](new_internal_verus.rs) | EXTRA_IN_VERUS |  | 146-160 |
| `pool_id` [pool_id_verus.rs](pool_id_verus.rs) | EXTRA_IN_VERUS |  | 184-190 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `alloc` [alloc.diff](alloc.diff) | [alloc_source.rs](alloc_source.rs) | [alloc_verus.rs](alloc_verus.rs) | MISMATCH | ❌ |
| `alloc_many` [alloc_many.diff](alloc_many.diff) | [alloc_many_source.rs](alloc_many_source.rs) | [alloc_many_verus.rs](alloc_many_verus.rs) | MISSING_IN_VERUS | ❌ |
| `alloc_range` [alloc_range.diff](alloc_range.diff) | [alloc_range_source.rs](alloc_range_source.rs) | [alloc_range_verus.rs](alloc_range_verus.rs) | MISMATCH | ❌ |
| `base` [base.diff](base.diff) | [base_source.rs](base_source.rs) | [base_verus.rs](base_verus.rs) | MISMATCH | ❌ |
| `clear` [clear_source.rs](clear_source.rs) | MISSING_IN_VERUS | ❌ |
| `deref` [deref_source.rs](deref_source.rs) | MISSING_IN_VERUS | ❌ |
| `deref_mut` [deref_mut_source.rs](deref_mut_source.rs) | MISSING_IN_VERUS | ❌ |
| `drop` [drop_source.rs](drop_source.rs) | MISSING_IN_VERUS | ❌ |
| `free` [free.diff](free.diff) | [free_source.rs](free_source.rs) | [free_verus.rs](free_verus.rs) | MISMATCH | ❌ |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISMATCH | ❌ |
| `address` [address_verus.rs](address_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `alloc_contiguous` [alloc_contiguous_verus.rs](alloc_contiguous_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `alloc_noncontiguous` [alloc_noncontiguous_verus.rs](alloc_noncontiguous_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `capacity` [capacity_verus.rs](capacity_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `free_contiguous` [free_contiguous_verus.rs](free_contiguous_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `free_range` [free_range_verus.rs](free_range_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `get_pool_id` [get_pool_id_verus.rs](get_pool_id_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `new_internal` [new_internal_verus.rs](new_internal_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `pool_id` [pool_id_verus.rs](pool_id_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `KernelFrame` [struct_KernelFrame.diff](struct_KernelFrame.diff) | [struct_KernelFrame_source.rs](struct_KernelFrame_source.rs) | [struct_KernelFrame_verus.rs](struct_KernelFrame_verus.rs): MISMATCH
- `Kpool` [struct_Kpool.diff](struct_Kpool.diff) | [struct_Kpool_source.rs](struct_Kpool_source.rs) | [struct_Kpool_verus.rs](struct_Kpool_verus.rs): MISMATCH
- `KpoolInner` [struct_KpoolInner_source.rs](struct_KpoolInner_source.rs): MISSING_IN_VERUS
