# Exec Diff: vmem

**Source:** `/home/ubuntu/nanvix/src/kernel/src/mm/virt/vmem.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/mm/virt/vmem.rs`

| Function | Status | Files |
|----------|--------|-------|
| `clone` | MISMATCH | clone_source.rs, clone_verus.rs, clone.diff |
| `copy_from_user_unaligned` | MISMATCH | copy_from_user_unaligned_source.rs, copy_from_user_unaligned_verus.rs, copy_from_user_unaligned.diff |
| `copy_to_user_unaligned` | MISMATCH | copy_to_user_unaligned_source.rs, copy_to_user_unaligned_verus.rs, copy_to_user_unaligned.diff |
| `copy_to_user_unaligned_unchecked` | MISMATCH | copy_to_user_unaligned_unchecked_source.rs, copy_to_user_unaligned_unchecked_verus.rs, copy_to_user_unaligned_unchecked.diff |
| `drop` | MISSING_IN_VERUS | drop_source.rs (MISSING in verus) |
| `find_user_frame` | MISMATCH | find_user_frame_source.rs, find_user_frame_verus.rs, find_user_frame.diff |
| `is_kernel_addr` | MISMATCH | is_kernel_addr_source.rs, is_kernel_addr_verus.rs, is_kernel_addr.diff |
| `is_kernel_region` | MISMATCH | is_kernel_region_source.rs, is_kernel_region_verus.rs, is_kernel_region.diff |
| `is_physical_region` | MISMATCH | is_physical_region_source.rs, is_physical_region_verus.rs, is_physical_region.diff |
| `is_user_addr` | MISMATCH | is_user_addr_source.rs, is_user_addr_verus.rs, is_user_addr.diff |
| `is_user_region` | MISMATCH | is_user_region_source.rs, is_user_region_verus.rs, is_user_region.diff |
| `kctrl` | MISMATCH | kctrl_source.rs, kctrl_verus.rs, kctrl.diff |
| `load` | MISMATCH | load_source.rs, load_verus.rs, load.diff |
| `lookup_kernel_page_table` | MISSING_IN_VERUS | lookup_kernel_page_table_source.rs (MISSING in verus) |
| `lookup_page_table` | MISSING_IN_VERUS | lookup_page_table_source.rs (MISSING in verus) |
| `map` | MISMATCH | map_source.rs, map_verus.rs, map.diff |
| `map_kpage` | MISMATCH | map_kpage_source.rs, map_kpage_verus.rs, map_kpage.diff |
| `memset` | MISMATCH | memset_source.rs, memset_verus.rs, memset.diff |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `pgdir` | MISMATCH | pgdir_source.rs, pgdir_verus.rs, pgdir.diff |
| `uctrl` | MISMATCH | uctrl_source.rs, uctrl_verus.rs, uctrl.diff |
| `unmap` | MISMATCH | unmap_source.rs, unmap_verus.rs, unmap.diff |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `PageMapping` | EXTRA_IN_VERUS | struct_PageMapping_verus.rs (EXTRA) |
| `Vmem` | MISMATCH | struct_Vmem_source.rs, struct_Vmem_verus.rs, struct_Vmem.diff |
