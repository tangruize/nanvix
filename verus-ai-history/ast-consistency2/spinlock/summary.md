# Exec Diff: spinlock

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/sync/spinlock.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/sync/spinlock.rs`

| Function | Status | Files |
|----------|--------|-------|
| `drop` | MISSING_IN_VERUS | drop_source.rs (MISSING in verus) |
| `lock` | MISMATCH | lock_source.rs, lock_verus.rs, lock.diff |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `is_locked` | EXTRA_IN_VERUS | is_locked_verus.rs (EXTRA) |
| `try_lock` | EXTRA_IN_VERUS | try_lock_verus.rs (EXTRA) |
| `unlock` | EXTRA_IN_VERUS | unlock_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `Spinlock` | MISMATCH | struct_Spinlock_source.rs, struct_Spinlock_verus.rs, struct_Spinlock.diff |
| `SpinlockGuard` | MISSING_IN_VERUS | struct_SpinlockGuard_source.rs (MISSING in verus) |
