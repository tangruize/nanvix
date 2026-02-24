# Exec Diff: clock

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/clock.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/clock.rs`

| Function | Status | Files |
|----------|--------|-------|
| `get` | MISMATCH | get_source.rs, get_verus.rs, get.diff |
| `increment` | MISMATCH | increment_source.rs, increment_verus.rs, increment.diff |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `now` | MISMATCH | now_source.rs, now_verus.rs, now.diff |
| `ticks` | MISMATCH | ticks_source.rs, ticks_verus.rs, ticks.diff |
| `timer_handler` | MISSING_IN_VERUS | timer_handler_source.rs (MISSING in verus) |
| `compute_nanoseconds` | EXTRA_IN_VERUS | compute_nanoseconds_verus.rs (EXTRA) |
| `compute_seconds` | EXTRA_IN_VERUS | compute_seconds_verus.rs (EXTRA) |
| `is_max` | EXTRA_IN_VERUS | is_max_verus.rs (EXTRA) |
| `is_zero` | EXTRA_IN_VERUS | is_zero_verus.rs (EXTRA) |
| `now_fallback_model` | EXTRA_IN_VERUS | now_fallback_model_verus.rs (EXTRA) |
| `now_pit_model` | EXTRA_IN_VERUS | now_pit_model_verus.rs (EXTRA) |
| `standalone_now` | EXTRA_IN_VERUS | standalone_now_verus.rs (EXTRA) |
| `standalone_ticks` | EXTRA_IN_VERUS | standalone_ticks_verus.rs (EXTRA) |
| `timer_handler_model` | EXTRA_IN_VERUS | timer_handler_model_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `TimerTicks` | MISMATCH | struct_TimerTicks_source.rs, struct_TimerTicks_verus.rs, struct_TimerTicks.diff |
