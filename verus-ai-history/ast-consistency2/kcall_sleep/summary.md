# Exec Diff: sleep

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/kcall/sleep.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/kcall/sleep.rs`

| Function | Status | Files |
|----------|--------|-------|
| `sleep` | MISMATCH | sleep_source.rs, sleep_verus.rs, sleep.diff |
| `checked_add_duration` | EXTRA_IN_VERUS | checked_add_duration_verus.rs (EXTRA) |
| `clock_now` | EXTRA_IN_VERUS | clock_now_verus.rs (EXTRA) |
| `duration_new` | EXTRA_IN_VERUS | duration_new_verus.rs (EXTRA) |
| `new` | EXTRA_IN_VERUS | new_verus.rs (EXTRA) |
| `process_manager_sleep` | EXTRA_IN_VERUS | process_manager_sleep_verus.rs (EXTRA) |
| `sleep_end_to_end` | EXTRA_IN_VERUS | sleep_end_to_end_verus.rs (EXTRA) |
| `sleep_model` | EXTRA_IN_VERUS | sleep_model_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `DurationModel` | EXTRA_IN_VERUS | struct_DurationModel_verus.rs (EXTRA) |
| `SystemTimeModel` | EXTRA_IN_VERUS | struct_SystemTimeModel_verus.rs (EXTRA) |
