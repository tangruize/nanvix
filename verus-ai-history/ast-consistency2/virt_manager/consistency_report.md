# Exec Consistency Report

**Source:** `src/kernel/src/mm/virt/manager.rs`
**Verus:** `verus/split/kernel/mm/virt/mod.rs`

## Summary

- Functions matched: 0/12
- Functions mismatched: 1
- Missing in Verus: 11
- Extra in Verus: 18
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `alloc_kpage` [alloc_kpage_source.rs](alloc_kpage_source.rs) | MISSING_IN_VERUS | 345-355 |  |
| `alloc_kpages` [alloc_kpages_source.rs](alloc_kpages_source.rs) | MISSING_IN_VERUS | 372-388 |  |
| `alloc_upage` [alloc_upage_source.rs](alloc_upage_source.rs) | MISSING_IN_VERUS | 197-238 |  |
| `alloc_upages` [alloc_upages_source.rs](alloc_upages_source.rs) | MISSING_IN_VERUS | 263-306 |  |
| `ctrl_upage` [ctrl_upage_source.rs](ctrl_upage_source.rs) | MISSING_IN_VERUS | 323-330 |  |
| `get` [get_source.rs](get_source.rs) | MISSING_IN_VERUS | 123-130 |  |
| `get_mut` [get_mut_source.rs](get_mut_source.rs) | MISSING_IN_VERUS | 147-154 |  |
| `init` [init.diff](init.diff) | [init_source.rs](init_source.rs) | [init_verus.rs](init_verus.rs) | MISMATCH | 87-105 | 1178-1492 |
| `load_elf` [load_elf_source.rs](load_elf_source.rs) | MISSING_IN_VERUS | 391-397 |  |
| `new` [new_source.rs](new_source.rs) | MISSING_IN_VERUS | 166-182 |  |
| `new_vmem` [new_vmem_source.rs](new_vmem_source.rs) | MISSING_IN_VERUS | 185-195 |  |
| `unmap_upage` [unmap_upage_source.rs](unmap_upage_source.rs) | MISSING_IN_VERUS | 254-261 |  |
| `check_pgtab_monotonicity` [check_pgtab_monotonicity_verus.rs](check_pgtab_monotonicity_verus.rs) | EXTRA_IN_VERUS |  | 449-461 |
| `compute_loop_end` [compute_loop_end_verus.rs](compute_loop_end_verus.rs) | EXTRA_IN_VERUS |  | 614-622 |
| `compute_pgtab_base` [compute_pgtab_base_verus.rs](compute_pgtab_base_verus.rs) | EXTRA_IN_VERUS |  | 356-367 |
| `compute_region_page_count` [compute_region_page_count_verus.rs](compute_region_page_count_verus.rs) | EXTRA_IN_VERUS |  | 473-482 |
| `deref_len` [deref_len_verus.rs](deref_len_verus.rs) | EXTRA_IN_VERUS |  | 282-288 |
| `deref_mut_len` [deref_mut_len_verus.rs](deref_mut_len_verus.rs) | EXTRA_IN_VERUS |  | 297-303 |
| `get_mmio_paddr` [get_mmio_paddr_verus.rs](get_mmio_paddr_verus.rs) | EXTRA_IN_VERUS |  | 586-594 |
| `get_nth_page_addr` [get_nth_page_addr_verus.rs](get_nth_page_addr_verus.rs) | EXTRA_IN_VERUS |  | 495-504 |
| `get_page_paddr` [get_page_paddr_verus.rs](get_page_paddr_verus.rs) | EXTRA_IN_VERUS |  | 540-557 |
| `init_checked` [init_checked_verus.rs](init_checked_verus.rs) | EXTRA_IN_VERUS |  | 835-884 |
| `init_full` [init_full_verus.rs](init_full_verus.rs) | EXTRA_IN_VERUS |  | 1057-1112 |
| `is_last_kernel_page` [is_last_kernel_page_verus.rs](is_last_kernel_page_verus.rs) | EXTRA_IN_VERUS |  | 641-648 |
| `merge_regions` [merge_regions_verus.rs](merge_regions_verus.rs) | EXTRA_IN_VERUS |  | 912-988 |
| `page_table_map_page` [page_table_map_page_verus.rs](page_table_map_page_verus.rs) | EXTRA_IN_VERUS |  | 674-680 |
| `pgtab_decision` [pgtab_decision_verus.rs](pgtab_decision_verus.rs) | EXTRA_IN_VERUS |  | 388-418 |
| `sort_regions_by_start` [sort_regions_by_start_verus.rs](sort_regions_by_start_verus.rs) | EXTRA_IN_VERUS |  | 1006-1032 |
| `validate_regions` [validate_regions_verus.rs](validate_regions_verus.rs) | EXTRA_IN_VERUS |  | 721-809 |
| `virt_align_down` [virt_align_down_verus.rs](virt_align_down_verus.rs) | EXTRA_IN_VERUS |  | 326-339 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `alloc_kpage` [alloc_kpage_source.rs](alloc_kpage_source.rs) | MISSING_IN_VERUS | ❌ |
| `alloc_kpages` [alloc_kpages_source.rs](alloc_kpages_source.rs) | MISSING_IN_VERUS | ❌ |
| `alloc_upage` [alloc_upage_source.rs](alloc_upage_source.rs) | MISSING_IN_VERUS | ❌ |
| `alloc_upages` [alloc_upages_source.rs](alloc_upages_source.rs) | MISSING_IN_VERUS | ❌ |
| `ctrl_upage` [ctrl_upage_source.rs](ctrl_upage_source.rs) | MISSING_IN_VERUS | ❌ |
| `get` [get_source.rs](get_source.rs) | MISSING_IN_VERUS | ❌ |
| `get_mut` [get_mut_source.rs](get_mut_source.rs) | MISSING_IN_VERUS | ❌ |
| `init` [init.diff](init.diff) | [init_source.rs](init_source.rs) | [init_verus.rs](init_verus.rs) | MISMATCH | ❌ |
| `load_elf` [load_elf_source.rs](load_elf_source.rs) | MISSING_IN_VERUS | ❌ |
| `new` [new_source.rs](new_source.rs) | MISSING_IN_VERUS | ❌ |
| `new_vmem` [new_vmem_source.rs](new_vmem_source.rs) | MISSING_IN_VERUS | ❌ |
| `unmap_upage` [unmap_upage_source.rs](unmap_upage_source.rs) | MISSING_IN_VERUS | ❌ |
| `check_pgtab_monotonicity` [check_pgtab_monotonicity_verus.rs](check_pgtab_monotonicity_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `compute_loop_end` [compute_loop_end_verus.rs](compute_loop_end_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `compute_pgtab_base` [compute_pgtab_base_verus.rs](compute_pgtab_base_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `compute_region_page_count` [compute_region_page_count_verus.rs](compute_region_page_count_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `deref_len` [deref_len_verus.rs](deref_len_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `deref_mut_len` [deref_mut_len_verus.rs](deref_mut_len_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `get_mmio_paddr` [get_mmio_paddr_verus.rs](get_mmio_paddr_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `get_nth_page_addr` [get_nth_page_addr_verus.rs](get_nth_page_addr_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `get_page_paddr` [get_page_paddr_verus.rs](get_page_paddr_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `init_checked` [init_checked_verus.rs](init_checked_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `init_full` [init_full_verus.rs](init_full_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `is_last_kernel_page` [is_last_kernel_page_verus.rs](is_last_kernel_page_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `merge_regions` [merge_regions_verus.rs](merge_regions_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `page_table_map_page` [page_table_map_page_verus.rs](page_table_map_page_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `pgtab_decision` [pgtab_decision_verus.rs](pgtab_decision_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `sort_regions_by_start` [sort_regions_by_start_verus.rs](sort_regions_by_start_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `validate_regions` [validate_regions_verus.rs](validate_regions_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `virt_align_down` [virt_align_down_verus.rs](virt_align_down_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `MemRegion` [struct_MemRegion_verus.rs](struct_MemRegion_verus.rs): EXTRA_IN_VERUS
- `VirtMemoryManager` [struct_VirtMemoryManager_source.rs](struct_VirtMemoryManager_source.rs): MISSING_IN_VERUS
