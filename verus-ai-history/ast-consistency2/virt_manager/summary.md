# Exec Diff: manager

**Source:** `src/kernel/src/mm/virt/manager.rs`
**Verus:** `verus/split/kernel/mm/virt/mod.rs`

| Function | Status | Files |
|----------|--------|-------|
| `alloc_kpage` | MISSING_IN_VERUS | alloc_kpage_source.rs (MISSING in verus) |
| `alloc_kpages` | MISSING_IN_VERUS | alloc_kpages_source.rs (MISSING in verus) |
| `alloc_upage` | MISSING_IN_VERUS | alloc_upage_source.rs (MISSING in verus) |
| `alloc_upages` | MISSING_IN_VERUS | alloc_upages_source.rs (MISSING in verus) |
| `ctrl_upage` | MISSING_IN_VERUS | ctrl_upage_source.rs (MISSING in verus) |
| `get` | MISSING_IN_VERUS | get_source.rs (MISSING in verus) |
| `get_mut` | MISSING_IN_VERUS | get_mut_source.rs (MISSING in verus) |
| `init` | MISMATCH | init_source.rs, init_verus.rs, init.diff |
| `load_elf` | MISSING_IN_VERUS | load_elf_source.rs (MISSING in verus) |
| `new` | MISSING_IN_VERUS | new_source.rs (MISSING in verus) |
| `new_vmem` | MISSING_IN_VERUS | new_vmem_source.rs (MISSING in verus) |
| `unmap_upage` | MISSING_IN_VERUS | unmap_upage_source.rs (MISSING in verus) |
| `check_pgtab_monotonicity` | EXTRA_IN_VERUS | check_pgtab_monotonicity_verus.rs (EXTRA) |
| `compute_loop_end` | EXTRA_IN_VERUS | compute_loop_end_verus.rs (EXTRA) |
| `compute_pgtab_base` | EXTRA_IN_VERUS | compute_pgtab_base_verus.rs (EXTRA) |
| `compute_region_page_count` | EXTRA_IN_VERUS | compute_region_page_count_verus.rs (EXTRA) |
| `deref_len` | EXTRA_IN_VERUS | deref_len_verus.rs (EXTRA) |
| `deref_mut_len` | EXTRA_IN_VERUS | deref_mut_len_verus.rs (EXTRA) |
| `get_mmio_paddr` | EXTRA_IN_VERUS | get_mmio_paddr_verus.rs (EXTRA) |
| `get_nth_page_addr` | EXTRA_IN_VERUS | get_nth_page_addr_verus.rs (EXTRA) |
| `get_page_paddr` | EXTRA_IN_VERUS | get_page_paddr_verus.rs (EXTRA) |
| `init_checked` | EXTRA_IN_VERUS | init_checked_verus.rs (EXTRA) |
| `init_full` | EXTRA_IN_VERUS | init_full_verus.rs (EXTRA) |
| `is_last_kernel_page` | EXTRA_IN_VERUS | is_last_kernel_page_verus.rs (EXTRA) |
| `merge_regions` | EXTRA_IN_VERUS | merge_regions_verus.rs (EXTRA) |
| `page_table_map_page` | EXTRA_IN_VERUS | page_table_map_page_verus.rs (EXTRA) |
| `pgtab_decision` | EXTRA_IN_VERUS | pgtab_decision_verus.rs (EXTRA) |
| `sort_regions_by_start` | EXTRA_IN_VERUS | sort_regions_by_start_verus.rs (EXTRA) |
| `validate_regions` | EXTRA_IN_VERUS | validate_regions_verus.rs (EXTRA) |
| `virt_align_down` | EXTRA_IN_VERUS | virt_align_down_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `MemRegion` | EXTRA_IN_VERUS | struct_MemRegion_verus.rs (EXTRA) |
| `VirtMemoryManager` | MISSING_IN_VERUS | struct_VirtMemoryManager_source.rs (MISSING in verus) |
