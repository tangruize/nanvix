# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/mm/phys/upool.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/mm/phys/upool.rs`

## Summary

- Functions matched: 0/5
- Functions mismatched: 5
- Missing in Verus: 0
- Extra in Verus: 2
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `address` [address.diff](address.diff) [address_source.rs](address_source.rs) [address_verus.rs](address_verus.rs) | MISMATCH | 150-152 | 138-142 |
| `alloc` [alloc.diff](alloc.diff) [alloc_source.rs](alloc_source.rs) [alloc_verus.rs](alloc_verus.rs) | MISMATCH | 201-205 | 250-311 |
| `alloc_many` [alloc_many.diff](alloc_many.diff) [alloc_many_source.rs](alloc_many_source.rs) [alloc_many_verus.rs](alloc_many_verus.rs) | MISMATCH | 207-221 | 353-508 |
| `free` [free.diff](free.diff) [free_source.rs](free_source.rs) [free_verus.rs](free_verus.rs) | MISMATCH | 236-238 | 536-557 |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | 184-188 | 206-219 |
| `capacity` [capacity_verus.rs](capacity_verus.rs) | EXTRA_IN_VERUS |  | 223-231 |
| `free_by_addr` [free_by_addr_verus.rs](free_by_addr_verus.rs) | EXTRA_IN_VERUS |  | 587-609 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `address` [address.diff](address.diff) [address_source.rs](address_source.rs) [address_verus.rs](address_verus.rs) | MISMATCH | ❌ |
| `alloc` [alloc.diff](alloc.diff) [alloc_source.rs](alloc_source.rs) [alloc_verus.rs](alloc_verus.rs) | MISMATCH | ❌ |
| `alloc_many` [alloc_many.diff](alloc_many.diff) [alloc_many_source.rs](alloc_many_source.rs) [alloc_many_verus.rs](alloc_many_verus.rs) | MISMATCH | ❌ |
| `free` [free.diff](free.diff) [free_source.rs](free_source.rs) [free_verus.rs](free_verus.rs) | MISMATCH | ❌ |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | ❌ |
| `capacity` [capacity_verus.rs](capacity_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `free_by_addr` [free_by_addr_verus.rs](free_by_addr_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `Upool` [struct_Upool.diff](struct_Upool.diff) [struct_Upool_source.rs](struct_Upool_source.rs) [struct_Upool_verus.rs](struct_Upool_verus.rs): MISMATCH
- `UpoolInner` [struct_UpoolInner_source.rs](struct_UpoolInner_source.rs): MISSING_IN_VERUS
- `UserFrame` [struct_UserFrame.diff](struct_UserFrame.diff) [struct_UserFrame_source.rs](struct_UserFrame_source.rs) [struct_UserFrame_verus.rs](struct_UserFrame_verus.rs): MISMATCH
