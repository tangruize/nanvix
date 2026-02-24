# Exec Diff: ustack

**Source:** `/home/ubuntu/nanvix/src/kernel/src/mm/ustack.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/mm/ustack.rs`

| Function | Status | Files |
|----------|--------|-------|
| `base` | MISMATCH | base_source.rs, base_verus.rs, base.diff |
| `fmt` | MISMATCH | fmt_source.rs, fmt_verus.rs, fmt.diff |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `size` | MISMATCH | size_source.rs, size_verus.rs, size.diff |
| `top` | MISMATCH | top_source.rs, top_verus.rs, top.diff |
| `base_raw` | EXTRA_IN_VERUS | base_raw_verus.rs (EXTRA) |
| `contains` | EXTRA_IN_VERUS | contains_verus.rs (EXTRA) |
| `from_raw` | EXTRA_IN_VERUS | from_raw_verus.rs (EXTRA) |
| `from_raw_unchecked` | EXTRA_IN_VERUS | from_raw_unchecked_verus.rs (EXTRA) |
| `has_room` | EXTRA_IN_VERUS | has_room_verus.rs (EXTRA) |
| `initial_sp` | EXTRA_IN_VERUS | initial_sp_verus.rs (EXTRA) |
| `into_raw` | EXTRA_IN_VERUS | into_raw_verus.rs (EXTRA) |
| `page_index` | EXTRA_IN_VERUS | page_index_verus.rs (EXTRA) |
| `top_raw` | EXTRA_IN_VERUS | top_raw_verus.rs (EXTRA) |
| `try_new` | EXTRA_IN_VERUS | try_new_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `PageAlignedAddr` | EXTRA_IN_VERUS | struct_PageAlignedAddr_verus.rs (EXTRA) |
| `UserStack` | MISMATCH | struct_UserStack_source.rs, struct_UserStack_verus.rs, struct_UserStack.diff |
