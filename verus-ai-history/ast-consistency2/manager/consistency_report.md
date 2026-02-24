# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/mm/phys/manager.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/mm/phys/manager.rs`

## Summary

- Functions matched: 0/6
- Functions mismatched: 1
- Missing in Verus: 5
- Extra in Verus: 9
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `alloc_kernel_frame` [alloc_kernel_frame.diff](alloc_kernel_frame.diff) [alloc_kernel_frame_source.rs](alloc_kernel_frame_source.rs) [alloc_kernel_frame_verus.rs](alloc_kernel_frame_verus.rs) | MISSING_IN_VERUS | 56-58 |  |
| `alloc_many_kernel_frames` [alloc_many_kernel_frames.diff](alloc_many_kernel_frames.diff) [alloc_many_kernel_frames_source.rs](alloc_many_kernel_frames_source.rs) [alloc_many_kernel_frames_verus.rs](alloc_many_kernel_frames_verus.rs) | MISSING_IN_VERUS | 75-81 |  |
| `alloc_many_user_frames` [alloc_many_user_frames.diff](alloc_many_user_frames.diff) [alloc_many_user_frames_source.rs](alloc_many_user_frames_source.rs) [alloc_many_user_frames_verus.rs](alloc_many_user_frames_verus.rs) | MISSING_IN_VERUS | 39-41 |  |
| `alloc_user_frame` [alloc_user_frame.diff](alloc_user_frame.diff) [alloc_user_frame_source.rs](alloc_user_frame_source.rs) [alloc_user_frame_verus.rs](alloc_user_frame_verus.rs) | MISSING_IN_VERUS | 35-37 |  |
| `free_user_frame` [free_user_frame.diff](free_user_frame.diff) [free_user_frame_source.rs](free_user_frame_source.rs) [free_user_frame_verus.rs](free_user_frame_verus.rs) | MISSING_IN_VERUS | 96-98 |  |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | 31-33 | 177-189 |
| `alloc_kpage` [alloc_kpage_verus.rs](alloc_kpage_verus.rs) | EXTRA_IN_VERUS |  | 426-447 |
| `alloc_kpages` [alloc_kpages_verus.rs](alloc_kpages_verus.rs) | EXTRA_IN_VERUS |  | 478-489 |
| `alloc_upage` [alloc_upage_verus.rs](alloc_upage_verus.rs) | EXTRA_IN_VERUS |  | 261-309 |
| `alloc_upages` [alloc_upages_verus.rs](alloc_upages_verus.rs) | EXTRA_IN_VERUS |  | 534-565 |
| `ctrl_upage` [ctrl_upage_verus.rs](ctrl_upage_verus.rs) | EXTRA_IN_VERUS |  | 391-409 |
| `kpool_capacity` [kpool_capacity_verus.rs](kpool_capacity_verus.rs) | EXTRA_IN_VERUS |  | 570-577 |
| `new_vmem` [new_vmem_verus.rs](new_vmem_verus.rs) | EXTRA_IN_VERUS |  | 214-226 |
| `unmap_upage` [unmap_upage_verus.rs](unmap_upage_verus.rs) | EXTRA_IN_VERUS |  | 334-368 |
| `upool_capacity` [upool_capacity_verus.rs](upool_capacity_verus.rs) | EXTRA_IN_VERUS |  | 581-588 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `alloc_kernel_frame` [alloc_kernel_frame.diff](alloc_kernel_frame.diff) [alloc_kernel_frame_source.rs](alloc_kernel_frame_source.rs) [alloc_kernel_frame_verus.rs](alloc_kernel_frame_verus.rs) | MISSING_IN_VERUS | ❌ |
| `alloc_many_kernel_frames` [alloc_many_kernel_frames.diff](alloc_many_kernel_frames.diff) [alloc_many_kernel_frames_source.rs](alloc_many_kernel_frames_source.rs) [alloc_many_kernel_frames_verus.rs](alloc_many_kernel_frames_verus.rs) | MISSING_IN_VERUS | ❌ |
| `alloc_many_user_frames` [alloc_many_user_frames.diff](alloc_many_user_frames.diff) [alloc_many_user_frames_source.rs](alloc_many_user_frames_source.rs) [alloc_many_user_frames_verus.rs](alloc_many_user_frames_verus.rs) | MISSING_IN_VERUS | ❌ |
| `alloc_user_frame` [alloc_user_frame.diff](alloc_user_frame.diff) [alloc_user_frame_source.rs](alloc_user_frame_source.rs) [alloc_user_frame_verus.rs](alloc_user_frame_verus.rs) | MISSING_IN_VERUS | ❌ |
| `free_user_frame` [free_user_frame.diff](free_user_frame.diff) [free_user_frame_source.rs](free_user_frame_source.rs) [free_user_frame_verus.rs](free_user_frame_verus.rs) | MISSING_IN_VERUS | ❌ |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | ❌ |
| `alloc_kpage` [alloc_kpage_verus.rs](alloc_kpage_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `alloc_kpages` [alloc_kpages_verus.rs](alloc_kpages_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `alloc_upage` [alloc_upage_verus.rs](alloc_upage_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `alloc_upages` [alloc_upages_verus.rs](alloc_upages_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `ctrl_upage` [ctrl_upage_verus.rs](ctrl_upage_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `kpool_capacity` [kpool_capacity_verus.rs](kpool_capacity_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `new_vmem` [new_vmem_verus.rs](new_vmem_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `unmap_upage` [unmap_upage_verus.rs](unmap_upage_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `upool_capacity` [upool_capacity_verus.rs](upool_capacity_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `PhysMemoryManager` [struct_PhysMemoryManager_source.rs](struct_PhysMemoryManager_source.rs): MISSING_IN_VERUS
- `VirtMemoryManager` [struct_VirtMemoryManager_verus.rs](struct_VirtMemoryManager_verus.rs): EXTRA_IN_VERUS
