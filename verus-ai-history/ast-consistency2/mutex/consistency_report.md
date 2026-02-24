# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/sync/mutex.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/sync/mutex.rs`

## Summary

- Functions matched: 0/7
- Functions mismatched: 3
- Missing in Verus: 4
- Extra in Verus: 2
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `drop` [drop_source.rs](drop_source.rs) | MISSING_IN_VERUS | 201-206 |  |
| `fmt` [fmt_source.rs](fmt_source.rs) | MISSING_IN_VERUS | 190-197 |  |
| `lock` [lock.diff](lock.diff) [lock_source.rs](lock_source.rs) [lock_verus.rs](lock_verus.rs) | MISMATCH | 174-186 | 270-286 |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | 98-103 | 197-207 |
| `reference_count` [reference_count.diff](reference_count.diff) [reference_count_source.rs](reference_count_source.rs) [reference_count_verus.rs](reference_count_verus.rs) | MISSING_IN_VERUS | 120-122 |  |
| `try_lock` [try_lock.diff](try_lock.diff) [try_lock_source.rs](try_lock_source.rs) [try_lock_verus.rs](try_lock_verus.rs) | MISMATCH | 133-146 | 229-255 |
| `unlock_unchecked` [unlock_unchecked.diff](unlock_unchecked.diff) [unlock_unchecked_source.rs](unlock_unchecked_source.rs) [unlock_unchecked_verus.rs](unlock_unchecked_verus.rs) | MISSING_IN_VERUS | 78-81 |  |
| `is_locked` [is_locked_verus.rs](is_locked_verus.rs) | EXTRA_IN_VERUS |  | 329-335 |
| `unlock` [unlock_verus.rs](unlock_verus.rs) | EXTRA_IN_VERUS |  | 304-322 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `drop` [drop_source.rs](drop_source.rs) | MISSING_IN_VERUS | ❌ |
| `fmt` [fmt_source.rs](fmt_source.rs) | MISSING_IN_VERUS | ❌ |
| `lock` [lock.diff](lock.diff) [lock_source.rs](lock_source.rs) [lock_verus.rs](lock_verus.rs) | MISMATCH | ❌ |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | ❌ |
| `reference_count` [reference_count.diff](reference_count.diff) [reference_count_source.rs](reference_count_source.rs) [reference_count_verus.rs](reference_count_verus.rs) | MISSING_IN_VERUS | ❌ |
| `try_lock` [try_lock.diff](try_lock.diff) [try_lock_source.rs](try_lock_source.rs) [try_lock_verus.rs](try_lock_verus.rs) | MISMATCH | ❌ |
| `unlock_unchecked` [unlock_unchecked.diff](unlock_unchecked.diff) [unlock_unchecked_source.rs](unlock_unchecked_source.rs) [unlock_unchecked_verus.rs](unlock_unchecked_verus.rs) | MISSING_IN_VERUS | ❌ |
| `is_locked` [is_locked_verus.rs](is_locked_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `unlock` [unlock_verus.rs](unlock_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `Mutex` [struct_Mutex.diff](struct_Mutex.diff) [struct_Mutex_source.rs](struct_Mutex_source.rs) [struct_Mutex_verus.rs](struct_Mutex_verus.rs): MISMATCH
- `MutexGuard` [struct_MutexGuard_source.rs](struct_MutexGuard_source.rs): MISSING_IN_VERUS
- `MutexInner` [struct_MutexInner_source.rs](struct_MutexInner_source.rs): MISSING_IN_VERUS
