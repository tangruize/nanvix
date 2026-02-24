# Exec Diff: kpool

**Source:** `/home/ubuntu/nanvix/src/kernel/src/mm/phys/kpool.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/mm/phys/kpool.rs`

| Function | Status | Files |
|----------|--------|-------|
| `alloc` | MISMATCH | alloc_source.rs, alloc_verus.rs, alloc.diff |
| `alloc_many` | MISMATCH | alloc_many_source.rs, alloc_many_verus.rs, alloc_many.diff |
| `alloc_range` | MISMATCH | alloc_range_source.rs, alloc_range_verus.rs, alloc_range.diff |
| `base` | MISMATCH | base_source.rs, base_verus.rs, base.diff |
| `clear` | MISSING_IN_VERUS | clear_source.rs (MISSING in verus) |
| `deref` | MISSING_IN_VERUS | deref_source.rs (MISSING in verus) |
| `deref_mut` | MISSING_IN_VERUS | deref_mut_source.rs (MISSING in verus) |
| `drop` | MISSING_IN_VERUS | drop_source.rs (MISSING in verus) |
| `free` | MISMATCH | free_source.rs, free_verus.rs, free.diff |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `address` | EXTRA_IN_VERUS | address_verus.rs (EXTRA) |
| `alloc_contiguous` | EXTRA_IN_VERUS | alloc_contiguous_verus.rs (EXTRA) |
| `alloc_noncontiguous` | EXTRA_IN_VERUS | alloc_noncontiguous_verus.rs (EXTRA) |
| `capacity` | EXTRA_IN_VERUS | capacity_verus.rs (EXTRA) |
| `free_contiguous` | EXTRA_IN_VERUS | free_contiguous_verus.rs (EXTRA) |
| `free_range` | EXTRA_IN_VERUS | free_range_verus.rs (EXTRA) |
| `get_pool_id` | EXTRA_IN_VERUS | get_pool_id_verus.rs (EXTRA) |
| `new_internal` | EXTRA_IN_VERUS | new_internal_verus.rs (EXTRA) |
| `pool_id` | EXTRA_IN_VERUS | pool_id_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `KernelFrame` | MISMATCH | struct_KernelFrame_source.rs, struct_KernelFrame_verus.rs, struct_KernelFrame.diff |
| `Kpool` | MISMATCH | struct_Kpool_source.rs, struct_Kpool_verus.rs, struct_Kpool.diff |
| `KpoolInner` | MISSING_IN_VERUS | struct_KpoolInner_source.rs (MISSING in verus) |
