# Exec Diff: upool

**Source:** `/home/ubuntu/nanvix/src/kernel/src/mm/phys/upool.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/mm/phys/upool.rs`

| Function | Status | Files |
|----------|--------|-------|
| `address` | MISMATCH | address_source.rs, address_verus.rs, address.diff |
| `alloc` | MISMATCH | alloc_source.rs, alloc_verus.rs, alloc.diff |
| `alloc_many` | MISMATCH | alloc_many_source.rs, alloc_many_verus.rs, alloc_many.diff |
| `free` | MISMATCH | free_source.rs, free_verus.rs, free.diff |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `capacity` | EXTRA_IN_VERUS | capacity_verus.rs (EXTRA) |
| `free_by_addr` | EXTRA_IN_VERUS | free_by_addr_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `Upool` | MISMATCH | struct_Upool_source.rs, struct_Upool_verus.rs, struct_Upool.diff |
| `UpoolInner` | MISSING_IN_VERUS | struct_UpoolInner_source.rs (MISSING in verus) |
| `UserFrame` | MISMATCH | struct_UserFrame_source.rs, struct_UserFrame_verus.rs, struct_UserFrame.diff |
