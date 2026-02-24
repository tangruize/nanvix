# Exec Diff: lock_mutex

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/kcall/lock_mutex.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/kcall/lock_mutex.rs`

| Function | Status | Files |
|----------|--------|-------|
| `lock_mutex` | MISMATCH | lock_mutex_source.rs, lock_mutex_verus.rs, lock_mutex.diff |
| `lock_mutex_model` | EXTRA_IN_VERUS | lock_mutex_model_verus.rs (EXTRA) |
| `parse_timeout` | EXTRA_IN_VERUS | parse_timeout_verus.rs (EXTRA) |
| `put_mutex_guard_model` | EXTRA_IN_VERUS | put_mutex_guard_model_verus.rs (EXTRA) |
| `system_time_new` | EXTRA_IN_VERUS | system_time_new_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `SystemTimeModel` | EXTRA_IN_VERUS | struct_SystemTimeModel_verus.rs (EXTRA) |
