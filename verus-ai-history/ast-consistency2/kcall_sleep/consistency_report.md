# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/kcall/sleep.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/kcall/sleep.rs`

## Summary

- Functions matched: 0/1
- Functions mismatched: 0
- Missing in Verus: 1
- Extra in Verus: 7
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `sleep` [sleep.diff](sleep.diff) [sleep_source.rs](sleep_source.rs) [sleep_verus.rs](sleep_verus.rs) | MISSING_IN_VERUS | 46-67 |  |
| `checked_add_duration` [checked_add_duration_verus.rs](checked_add_duration_verus.rs) | EXTRA_IN_VERUS |  | 272-283 |
| `clock_now` [clock_now_verus.rs](clock_now_verus.rs) | EXTRA_IN_VERUS |  | 256-263 |
| `duration_new` [duration_new_verus.rs](duration_new_verus.rs) | EXTRA_IN_VERUS |  | 340-364 |
| `new` [new_verus.rs](new_verus.rs) | EXTRA_IN_VERUS |  | 142-151 |
| `process_manager_sleep` [process_manager_sleep_verus.rs](process_manager_sleep_verus.rs) | EXTRA_IN_VERUS |  | 310-319 |
| `sleep_end_to_end` [sleep_end_to_end_verus.rs](sleep_end_to_end_verus.rs) | EXTRA_IN_VERUS |  | 520-556 |
| `sleep_model` [sleep_model_verus.rs](sleep_model_verus.rs) | EXTRA_IN_VERUS |  | 437-499 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `sleep` [sleep.diff](sleep.diff) [sleep_source.rs](sleep_source.rs) [sleep_verus.rs](sleep_verus.rs) | MISSING_IN_VERUS | ❌ |
| `checked_add_duration` [checked_add_duration_verus.rs](checked_add_duration_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `clock_now` [clock_now_verus.rs](clock_now_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `duration_new` [duration_new_verus.rs](duration_new_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `new` [new_verus.rs](new_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `process_manager_sleep` [process_manager_sleep_verus.rs](process_manager_sleep_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `sleep_end_to_end` [sleep_end_to_end_verus.rs](sleep_end_to_end_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `sleep_model` [sleep_model_verus.rs](sleep_model_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `DurationModel` [struct_DurationModel_verus.rs](struct_DurationModel_verus.rs): EXTRA_IN_VERUS
- `SystemTimeModel` [struct_SystemTimeModel_verus.rs](struct_SystemTimeModel_verus.rs): EXTRA_IN_VERUS
