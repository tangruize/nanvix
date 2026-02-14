# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/kcall/unlock_mutex.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/kcall/unlock_mutex.rs`

## Summary

- Functions matched: 0/1
- Functions mismatched: 0
- Missing in Verus: 1
- Extra in Verus: 3
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `unlock_mutex` | MISSING_IN_VERUS | 46-60 |  |
| `drop_guard_model` | EXTRA_IN_VERUS |  | 316-325 |
| `take_mutex_guard_model` | EXTRA_IN_VERUS |  | 264-296 |
| `unlock_mutex_model` | EXTRA_IN_VERUS |  | 354-420 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `unlock_mutex` | MISSING_IN_VERUS | ❌ |
| `drop_guard_model` | EXTRA_IN_VERUS | ❌ |
| `take_mutex_guard_model` | EXTRA_IN_VERUS | ❌ |
| `unlock_mutex_model` | EXTRA_IN_VERUS | ❌ |
