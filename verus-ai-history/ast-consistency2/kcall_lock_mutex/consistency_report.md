# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/kcall/lock_mutex.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/kcall/lock_mutex.rs`

## Summary

- Functions matched: 0/1
- Functions mismatched: 0
- Missing in Verus: 1
- Extra in Verus: 4
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `lock_mutex` [lock_mutex.diff](lock_mutex.diff) | [lock_mutex_source.rs](lock_mutex_source.rs) | [lock_mutex_verus.rs](lock_mutex_verus.rs) | MISSING_IN_VERUS | 63-95 |  |
| `lock_mutex_model` [lock_mutex_model_verus.rs](lock_mutex_model_verus.rs) | EXTRA_IN_VERUS |  | 563-710 |
| `parse_timeout` [parse_timeout_verus.rs](parse_timeout_verus.rs) | EXTRA_IN_VERUS |  | 498-534 |
| `put_mutex_guard_model` [put_mutex_guard_model_verus.rs](put_mutex_guard_model_verus.rs) | EXTRA_IN_VERUS |  | 420-712 |
| `system_time_new` [system_time_new_verus.rs](system_time_new_verus.rs) | EXTRA_IN_VERUS |  | 456-468 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `lock_mutex` [lock_mutex.diff](lock_mutex.diff) | [lock_mutex_source.rs](lock_mutex_source.rs) | [lock_mutex_verus.rs](lock_mutex_verus.rs) | MISSING_IN_VERUS | ❌ |
| `lock_mutex_model` [lock_mutex_model_verus.rs](lock_mutex_model_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `parse_timeout` [parse_timeout_verus.rs](parse_timeout_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `put_mutex_guard_model` [put_mutex_guard_model_verus.rs](put_mutex_guard_model_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `system_time_new` [system_time_new_verus.rs](system_time_new_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `SystemTimeModel` [struct_SystemTimeModel_verus.rs](struct_SystemTimeModel_verus.rs): EXTRA_IN_VERUS
