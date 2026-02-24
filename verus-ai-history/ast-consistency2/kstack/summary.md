# Exec Diff: kstack

**Source:** `/home/ubuntu/nanvix/src/kernel/src/mm/kstack.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/mm/kstack.rs`

| Function | Status | Files |
|----------|--------|-------|
| `base` | MISMATCH | base_source.rs, base_verus.rs, base.diff |
| `drop` | MISSING_IN_VERUS | drop_source.rs (MISSING in verus) |
| `fmt` | MISSING_IN_VERUS | fmt_source.rs (MISSING in verus) |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `size` | MISMATCH | size_source.rs, size_verus.rs, size.diff |
| `top` | MISMATCH | top_source.rs, top_verus.rs, top.diff |
| `contains` | EXTRA_IN_VERUS | contains_verus.rs (EXTRA) |
| `has_room` | EXTRA_IN_VERUS | has_room_verus.rs (EXTRA) |
| `initial_sp` | EXTRA_IN_VERUS | initial_sp_verus.rs (EXTRA) |
| `num_pages` | EXTRA_IN_VERUS | num_pages_verus.rs (EXTRA) |
| `page_index` | EXTRA_IN_VERUS | page_index_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `KernelStack` | MISMATCH | struct_KernelStack_source.rs, struct_KernelStack_verus.rs, struct_KernelStack.diff |
