# Exec Diff: fence

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/sync/fence.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/sync/fence.rs`

| Function | Status | Files |
|----------|--------|-------|
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `signal` | MISMATCH | signal_source.rs, signal_verus.rs, signal.diff |
| `wait` | MISMATCH | wait_source.rs, wait_verus.rs, wait.diff |
| `get_count` | EXTRA_IN_VERUS | get_count_verus.rs (EXTRA) |
| `get_total` | EXTRA_IN_VERUS | get_total_verus.rs (EXTRA) |
| `is_satisfied` | EXTRA_IN_VERUS | is_satisfied_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `Fence` | MISMATCH | struct_Fence_source.rs, struct_Fence_verus.rs, struct_Fence.diff |
