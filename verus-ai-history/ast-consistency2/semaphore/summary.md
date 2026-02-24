# Exec Diff: semaphore

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/sync/semaphore.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/sync/semaphore.rs`

| Function | Status | Files |
|----------|--------|-------|
| `down` | MISMATCH | down_source.rs, down_verus.rs, down.diff |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `try_down` | MISMATCH | try_down_source.rs, try_down_verus.rs, try_down.diff |
| `up` | MISMATCH | up_source.rs, up_verus.rs, up.diff |
| `down_available` | EXTRA_IN_VERUS | down_available_verus.rs (EXTRA) |
| `down_or_block` | EXTRA_IN_VERUS | down_or_block_verus.rs (EXTRA) |
| `get_value` | EXTRA_IN_VERUS | get_value_verus.rs (EXTRA) |
| `is_available` | EXTRA_IN_VERUS | is_available_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `Semaphore` | MISMATCH | struct_Semaphore_source.rs, struct_Semaphore_verus.rs, struct_Semaphore.diff |
