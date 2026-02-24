# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/sync/spinlock.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/sync/spinlock.rs`

## Summary

- Functions matched: 0/3
- Functions mismatched: 2
- Missing in Verus: 1
- Extra in Verus: 3
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `drop` [drop_source.rs](drop_source.rs) | MISSING_IN_VERUS | 75-77 |  |
| `lock` [lock.diff](lock.diff) [lock_source.rs](lock_source.rs) [lock_verus.rs](lock_verus.rs) | MISMATCH | 59-71 | 214-230 |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | 45-47 | 138-148 |
| `is_locked` [is_locked_verus.rs](is_locked_verus.rs) | EXTRA_IN_VERUS |  | 274-280 |
| `try_lock` [try_lock_verus.rs](try_lock_verus.rs) | EXTRA_IN_VERUS |  | 171-197 |
| `unlock` [unlock_verus.rs](unlock_verus.rs) | EXTRA_IN_VERUS |  | 246-264 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `drop` [drop_source.rs](drop_source.rs) | MISSING_IN_VERUS | ❌ |
| `lock` [lock.diff](lock.diff) [lock_source.rs](lock_source.rs) [lock_verus.rs](lock_verus.rs) | MISMATCH | ❌ |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | ❌ |
| `is_locked` [is_locked_verus.rs](is_locked_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `try_lock` [try_lock_verus.rs](try_lock_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `unlock` [unlock_verus.rs](unlock_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `Spinlock` [struct_Spinlock.diff](struct_Spinlock.diff) [struct_Spinlock_source.rs](struct_Spinlock_source.rs) [struct_Spinlock_verus.rs](struct_Spinlock_verus.rs): MISMATCH
- `SpinlockGuard` [struct_SpinlockGuard_source.rs](struct_SpinlockGuard_source.rs): MISSING_IN_VERUS
