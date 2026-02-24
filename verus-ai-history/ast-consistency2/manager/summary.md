# Exec Diff: manager

**Source:** `/home/ubuntu/nanvix/src/kernel/src/mm/phys/manager.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/mm/phys/manager.rs`

| Function | Status | Files |
|----------|--------|-------|
| `alloc_kernel_frame` | MISMATCH | alloc_kernel_frame_source.rs, alloc_kernel_frame_verus.rs, alloc_kernel_frame.diff |
| `alloc_many_kernel_frames` | MISMATCH | alloc_many_kernel_frames_source.rs, alloc_many_kernel_frames_verus.rs, alloc_many_kernel_frames.diff |
| `alloc_many_user_frames` | MISMATCH | alloc_many_user_frames_source.rs, alloc_many_user_frames_verus.rs, alloc_many_user_frames.diff |
| `alloc_user_frame` | MISMATCH | alloc_user_frame_source.rs, alloc_user_frame_verus.rs, alloc_user_frame.diff |
| `free_user_frame` | MISMATCH | free_user_frame_source.rs, free_user_frame_verus.rs, free_user_frame.diff |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `alloc_kpage` | EXTRA_IN_VERUS | alloc_kpage_verus.rs (EXTRA) |
| `alloc_kpages` | EXTRA_IN_VERUS | alloc_kpages_verus.rs (EXTRA) |
| `alloc_upage` | EXTRA_IN_VERUS | alloc_upage_verus.rs (EXTRA) |
| `alloc_upages` | EXTRA_IN_VERUS | alloc_upages_verus.rs (EXTRA) |
| `ctrl_upage` | EXTRA_IN_VERUS | ctrl_upage_verus.rs (EXTRA) |
| `kpool_capacity` | EXTRA_IN_VERUS | kpool_capacity_verus.rs (EXTRA) |
| `new_vmem` | EXTRA_IN_VERUS | new_vmem_verus.rs (EXTRA) |
| `unmap_upage` | EXTRA_IN_VERUS | unmap_upage_verus.rs (EXTRA) |
| `upool_capacity` | EXTRA_IN_VERUS | upool_capacity_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `PhysMemoryManager` | MISSING_IN_VERUS | struct_PhysMemoryManager_source.rs (MISSING in verus) |
| `VirtMemoryManager` | EXTRA_IN_VERUS | struct_VirtMemoryManager_verus.rs (EXTRA) |
