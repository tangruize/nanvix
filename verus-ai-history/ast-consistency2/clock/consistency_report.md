# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/clock.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/clock.rs`

## Summary

- Functions matched: 0/6
- Functions mismatched: 5
- Missing in Verus: 1
- Extra in Verus: 9
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `get` [get.diff](get.diff) [get_source.rs](get_source.rs) [get_verus.rs](get_verus.rs) | MISMATCH | 64-66 | 215-226 |
| `increment` [increment.diff](increment.diff) [increment_source.rs](increment_source.rs) [increment_verus.rs](increment_verus.rs) | MISMATCH | 69-83 | 246-308 |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | 56-61 | 186-196 |
| `now` [now.diff](now.diff) [now_source.rs](now_source.rs) [now_verus.rs](now_verus.rs) | MISMATCH | 148-168 | 466-491 |
| `ticks` [ticks.diff](ticks.diff) [ticks_source.rs](ticks_source.rs) [ticks_verus.rs](ticks_verus.rs) | MISMATCH | 138-141 | 320-330 |
| `timer_handler` [timer_handler_source.rs](timer_handler_source.rs) | MISSING_IN_VERUS | 105-128 |  |
| `compute_nanoseconds` [compute_nanoseconds_verus.rs](compute_nanoseconds_verus.rs) | EXTRA_IN_VERUS |  | 399-411 |
| `compute_seconds` [compute_seconds_verus.rs](compute_seconds_verus.rs) | EXTRA_IN_VERUS |  | 429-440 |
| `is_max` [is_max_verus.rs](is_max_verus.rs) | EXTRA_IN_VERUS |  | 341-351 |
| `is_zero` [is_zero_verus.rs](is_zero_verus.rs) | EXTRA_IN_VERUS |  | 362-377 |
| `now_fallback_model` [now_fallback_model_verus.rs](now_fallback_model_verus.rs) | EXTRA_IN_VERUS |  | 611-625 |
| `now_pit_model` [now_pit_model_verus.rs](now_pit_model_verus.rs) | EXTRA_IN_VERUS |  | 644-659 |
| `standalone_now` [standalone_now_verus.rs](standalone_now_verus.rs) | EXTRA_IN_VERUS |  | 579-592 |
| `standalone_ticks` [standalone_ticks_verus.rs](standalone_ticks_verus.rs) | EXTRA_IN_VERUS |  | 542-557 |
| `timer_handler_model` [timer_handler_model_verus.rs](timer_handler_model_verus.rs) | EXTRA_IN_VERUS |  | 516-526 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `get` [get.diff](get.diff) [get_source.rs](get_source.rs) [get_verus.rs](get_verus.rs) | MISMATCH | ❌ |
| `increment` [increment.diff](increment.diff) [increment_source.rs](increment_source.rs) [increment_verus.rs](increment_verus.rs) | MISMATCH | ❌ |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | ❌ |
| `now` [now.diff](now.diff) [now_source.rs](now_source.rs) [now_verus.rs](now_verus.rs) | MISMATCH | ❌ |
| `ticks` [ticks.diff](ticks.diff) [ticks_source.rs](ticks_source.rs) [ticks_verus.rs](ticks_verus.rs) | MISMATCH | ❌ |
| `timer_handler` [timer_handler_source.rs](timer_handler_source.rs) | MISSING_IN_VERUS | ❌ |
| `compute_nanoseconds` [compute_nanoseconds_verus.rs](compute_nanoseconds_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `compute_seconds` [compute_seconds_verus.rs](compute_seconds_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `is_max` [is_max_verus.rs](is_max_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `is_zero` [is_zero_verus.rs](is_zero_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `now_fallback_model` [now_fallback_model_verus.rs](now_fallback_model_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `now_pit_model` [now_pit_model_verus.rs](now_pit_model_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `standalone_now` [standalone_now_verus.rs](standalone_now_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `standalone_ticks` [standalone_ticks_verus.rs](standalone_ticks_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `timer_handler_model` [timer_handler_model_verus.rs](timer_handler_model_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `TimerTicks` [struct_TimerTicks.diff](struct_TimerTicks.diff) [struct_TimerTicks_source.rs](struct_TimerTicks_source.rs) [struct_TimerTicks_verus.rs](struct_TimerTicks_verus.rs): MISMATCH
