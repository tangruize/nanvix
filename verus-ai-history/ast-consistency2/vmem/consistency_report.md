# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/mm/virt/vmem.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/mm/virt/vmem.rs`

## Summary

- Functions matched: 0/22
- Functions mismatched: 19
- Missing in Verus: 3
- Extra in Verus: 0
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `clone` [clone.diff](clone.diff) [clone_source.rs](clone_source.rs) [clone_verus.rs](clone_verus.rs) | MISMATCH | 188-217 | 392-420 |
| `copy_from_user_unaligned` [copy_from_user_unaligned.diff](copy_from_user_unaligned.diff) [copy_from_user_unaligned_source.rs](copy_from_user_unaligned_source.rs) [copy_from_user_unaligned_verus.rs](copy_from_user_unaligned_verus.rs) | MISMATCH | 598-677 | 1090-1126 |
| `copy_to_user_unaligned` [copy_to_user_unaligned.diff](copy_to_user_unaligned.diff) [copy_to_user_unaligned_source.rs](copy_to_user_unaligned_source.rs) [copy_to_user_unaligned_verus.rs](copy_to_user_unaligned_verus.rs) | MISMATCH | 865-875 | 1169-1191 |
| `copy_to_user_unaligned_unchecked` [copy_to_user_unaligned_unchecked.diff](copy_to_user_unaligned_unchecked.diff) [copy_to_user_unaligned_unchecked_source.rs](copy_to_user_unaligned_unchecked_source.rs) [copy_to_user_unaligned_unchecked_verus.rs](copy_to_user_unaligned_unchecked_verus.rs) | MISMATCH | 712-839 | 1229-1248 |
| `drop` [drop_source.rs](drop_source.rs) | MISSING_IN_VERUS | 1092-1111 |  |
| `find_user_frame` [find_user_frame.diff](find_user_frame.diff) [find_user_frame_source.rs](find_user_frame_source.rs) [find_user_frame_verus.rs](find_user_frame_verus.rs) | MISMATCH | 553-571 | 915-950 |
| `is_kernel_addr` [is_kernel_addr.diff](is_kernel_addr.diff) [is_kernel_addr_source.rs](is_kernel_addr_source.rs) [is_kernel_addr_verus.rs](is_kernel_addr_verus.rs) | MISMATCH | 413-415 | 546-551 |
| `is_kernel_region` [is_kernel_region.diff](is_kernel_region.diff) [is_kernel_region_source.rs](is_kernel_region_source.rs) [is_kernel_region_verus.rs](is_kernel_region_verus.rs) | MISMATCH | 431-442 | 596-613 |
| `is_physical_region` [is_physical_region.diff](is_physical_region.diff) [is_physical_region_source.rs](is_physical_region_source.rs) [is_physical_region_verus.rs](is_physical_region_verus.rs) | MISMATCH | 458-469 | 626-643 |
| `is_user_addr` [is_user_addr.diff](is_user_addr.diff) [is_user_addr_source.rs](is_user_addr_source.rs) [is_user_addr_verus.rs](is_user_addr_verus.rs) | MISMATCH | 381-383 | 529-534 |
| `is_user_region` [is_user_region.diff](is_user_region.diff) [is_user_region_source.rs](is_user_region_source.rs) [is_user_region_verus.rs](is_user_region_verus.rs) | MISMATCH | 399-410 | 565-582 |
| `kctrl` [kctrl.diff](kctrl.diff) [kctrl_source.rs](kctrl_source.rs) [kctrl_verus.rs](kctrl_verus.rs) | MISMATCH | 1040-1088 | 1038-1048 |
| `load` [load.diff](load.diff) [load_source.rs](load_source.rs) [load_verus.rs](load_verus.rs) | MISMATCH | 219-223 | 438-443 |
| `lookup_kernel_page_table` [lookup_kernel_page_table_source.rs](lookup_kernel_page_table_source.rs) | MISSING_IN_VERUS | 505-537 |  |
| `lookup_page_table` [lookup_page_table_source.rs](lookup_page_table_source.rs) | MISSING_IN_VERUS | 472-503 |  |
| `map` [map.diff](map.diff) [map_source.rs](map_source.rs) [map_verus.rs](map_verus.rs) | MISMATCH | 314-378 | 691-778 |
| `map_kpage` [map_kpage.diff](map_kpage.diff) [map_kpage_source.rs](map_kpage_source.rs) [map_kpage_verus.rs](map_kpage_verus.rs) | MISMATCH | 243-311 | 501-516 |
| `memset` [memset.diff](memset.diff) [memset_source.rs](memset_source.rs) [memset_verus.rs](memset_verus.rs) | MISMATCH | 890-903 | 1263-1308 |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | 151-185 | 316-331 |
| `pgdir` [pgdir.diff](pgdir.diff) [pgdir_source.rs](pgdir_source.rs) [pgdir_verus.rs](pgdir_verus.rs) | MISMATCH | 226-228 | 461-466 |
| `uctrl` [uctrl.diff](uctrl.diff) [uctrl_source.rs](uctrl_source.rs) [uctrl_verus.rs](uctrl_verus.rs) | MISMATCH | 994-1037 | 964-1014 |
| `unmap` [unmap.diff](unmap.diff) [unmap_source.rs](unmap_source.rs) [unmap_verus.rs](unmap_verus.rs) | MISMATCH | 919-991 | 808-897 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `clone` [clone.diff](clone.diff) [clone_source.rs](clone_source.rs) [clone_verus.rs](clone_verus.rs) | MISMATCH | ❌ |
| `copy_from_user_unaligned` [copy_from_user_unaligned.diff](copy_from_user_unaligned.diff) [copy_from_user_unaligned_source.rs](copy_from_user_unaligned_source.rs) [copy_from_user_unaligned_verus.rs](copy_from_user_unaligned_verus.rs) | MISMATCH | ❌ |
| `copy_to_user_unaligned` [copy_to_user_unaligned.diff](copy_to_user_unaligned.diff) [copy_to_user_unaligned_source.rs](copy_to_user_unaligned_source.rs) [copy_to_user_unaligned_verus.rs](copy_to_user_unaligned_verus.rs) | MISMATCH | ❌ |
| `copy_to_user_unaligned_unchecked` [copy_to_user_unaligned_unchecked.diff](copy_to_user_unaligned_unchecked.diff) [copy_to_user_unaligned_unchecked_source.rs](copy_to_user_unaligned_unchecked_source.rs) [copy_to_user_unaligned_unchecked_verus.rs](copy_to_user_unaligned_unchecked_verus.rs) | MISMATCH | ❌ |
| `drop` [drop_source.rs](drop_source.rs) | MISSING_IN_VERUS | ❌ |
| `find_user_frame` [find_user_frame.diff](find_user_frame.diff) [find_user_frame_source.rs](find_user_frame_source.rs) [find_user_frame_verus.rs](find_user_frame_verus.rs) | MISMATCH | ❌ |
| `is_kernel_addr` [is_kernel_addr.diff](is_kernel_addr.diff) [is_kernel_addr_source.rs](is_kernel_addr_source.rs) [is_kernel_addr_verus.rs](is_kernel_addr_verus.rs) | MISMATCH | ❌ |
| `is_kernel_region` [is_kernel_region.diff](is_kernel_region.diff) [is_kernel_region_source.rs](is_kernel_region_source.rs) [is_kernel_region_verus.rs](is_kernel_region_verus.rs) | MISMATCH | ❌ |
| `is_physical_region` [is_physical_region.diff](is_physical_region.diff) [is_physical_region_source.rs](is_physical_region_source.rs) [is_physical_region_verus.rs](is_physical_region_verus.rs) | MISMATCH | ❌ |
| `is_user_addr` [is_user_addr.diff](is_user_addr.diff) [is_user_addr_source.rs](is_user_addr_source.rs) [is_user_addr_verus.rs](is_user_addr_verus.rs) | MISMATCH | ❌ |
| `is_user_region` [is_user_region.diff](is_user_region.diff) [is_user_region_source.rs](is_user_region_source.rs) [is_user_region_verus.rs](is_user_region_verus.rs) | MISMATCH | ❌ |
| `kctrl` [kctrl.diff](kctrl.diff) [kctrl_source.rs](kctrl_source.rs) [kctrl_verus.rs](kctrl_verus.rs) | MISMATCH | ❌ |
| `load` [load.diff](load.diff) [load_source.rs](load_source.rs) [load_verus.rs](load_verus.rs) | MISMATCH | ❌ |
| `lookup_kernel_page_table` [lookup_kernel_page_table_source.rs](lookup_kernel_page_table_source.rs) | MISSING_IN_VERUS | ❌ |
| `lookup_page_table` [lookup_page_table_source.rs](lookup_page_table_source.rs) | MISSING_IN_VERUS | ❌ |
| `map` [map.diff](map.diff) [map_source.rs](map_source.rs) [map_verus.rs](map_verus.rs) | MISMATCH | ❌ |
| `map_kpage` [map_kpage.diff](map_kpage.diff) [map_kpage_source.rs](map_kpage_source.rs) [map_kpage_verus.rs](map_kpage_verus.rs) | MISMATCH | ❌ |
| `memset` [memset.diff](memset.diff) [memset_source.rs](memset_source.rs) [memset_verus.rs](memset_verus.rs) | MISMATCH | ❌ |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | ❌ |
| `pgdir` [pgdir.diff](pgdir.diff) [pgdir_source.rs](pgdir_source.rs) [pgdir_verus.rs](pgdir_verus.rs) | MISMATCH | ❌ |
| `uctrl` [uctrl.diff](uctrl.diff) [uctrl_source.rs](uctrl_source.rs) [uctrl_verus.rs](uctrl_verus.rs) | MISMATCH | ❌ |
| `unmap` [unmap.diff](unmap.diff) [unmap_source.rs](unmap_source.rs) [unmap_verus.rs](unmap_verus.rs) | MISMATCH | ❌ |

## Inconsistent Structs

- `PageMapping` [struct_PageMapping_verus.rs](struct_PageMapping_verus.rs): EXTRA_IN_VERUS
- `Vmem` [struct_Vmem.diff](struct_Vmem.diff) [struct_Vmem_source.rs](struct_Vmem_source.rs) [struct_Vmem_verus.rs](struct_Vmem_verus.rs): MISMATCH
