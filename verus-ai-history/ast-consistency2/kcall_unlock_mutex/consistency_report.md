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
| `unlock_mutex` [unlock_mutex.diff](unlock_mutex.diff) | [unlock_mutex_source.rs](unlock_mutex_source.rs) | [unlock_mutex_verus.rs](unlock_mutex_verus.rs) | MISSING_IN_VERUS | 46-60 |  |
| `drop_guard_model` [drop_guard_model_verus.rs](drop_guard_model_verus.rs) | EXTRA_IN_VERUS |  | 316-325 |
| `take_mutex_guard_model` [take_mutex_guard_model_verus.rs](take_mutex_guard_model_verus.rs) | EXTRA_IN_VERUS |  | 264-296 |
| `unlock_mutex_model` [unlock_mutex_model_verus.rs](unlock_mutex_model_verus.rs) | EXTRA_IN_VERUS |  | 354-420 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `unlock_mutex` [unlock_mutex.diff](unlock_mutex.diff) | [unlock_mutex_source.rs](unlock_mutex_source.rs) | [unlock_mutex_verus.rs](unlock_mutex_verus.rs) | MISSING_IN_VERUS | ❌ |
| `drop_guard_model` [drop_guard_model_verus.rs](drop_guard_model_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `take_mutex_guard_model` [take_mutex_guard_model_verus.rs](take_mutex_guard_model_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `unlock_mutex_model` [unlock_mutex_model_verus.rs](unlock_mutex_model_verus.rs) | EXTRA_IN_VERUS | ❌ |
