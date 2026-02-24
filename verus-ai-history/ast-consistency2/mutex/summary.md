# Exec Diff: mutex

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/sync/mutex.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/sync/mutex.rs`

| Function | Status | Files |
|----------|--------|-------|
| `drop` | MISSING_IN_VERUS | drop_source.rs (MISSING in verus) |
| `fmt` | MISSING_IN_VERUS | fmt_source.rs (MISSING in verus) |
| `lock` | MISMATCH | lock_source.rs, lock_verus.rs, lock.diff |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `reference_count` | MISMATCH | reference_count_source.rs, reference_count_verus.rs, reference_count.diff |
| `try_lock` | MISMATCH | try_lock_source.rs, try_lock_verus.rs, try_lock.diff |
| `unlock_unchecked` | MISMATCH | unlock_unchecked_source.rs, unlock_unchecked_verus.rs, unlock_unchecked.diff |
| `is_locked` | EXTRA_IN_VERUS | is_locked_verus.rs (EXTRA) |
| `unlock` | EXTRA_IN_VERUS | unlock_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `Mutex` | MISMATCH | struct_Mutex_source.rs, struct_Mutex_verus.rs, struct_Mutex.diff |
| `MutexGuard` | MISSING_IN_VERUS | struct_MutexGuard_source.rs (MISSING in verus) |
| `MutexInner` | MISSING_IN_VERUS | struct_MutexInner_source.rs (MISSING in verus) |
