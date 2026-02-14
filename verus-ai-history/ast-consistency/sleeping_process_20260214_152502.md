# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/process/state/sleeping.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/process/state/sleeping.rs`

## Summary

- Functions matched: 0/9
- Functions mismatched: 9
- Missing in Verus: 0
- Extra in Verus: 0
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `add_thread` | MISMATCH | 178-187 | 561-602 |
| `find_thread` | MISMATCH | 203-219 | 624-633 |
| `find_thread_mut` | MISMATCH | 235-251 | 650-666 |
| `new` | MISMATCH | 51-61 | 140-188 |
| `state` | MISMATCH | 63-65 | 201-206 |
| `state_mut` | MISMATCH | 67-69 | 220-228 |
| `terminate` | MISMATCH | 71-83 | 239-265 |
| `wakeup` | MISMATCH | 85-105 | 287-431 |
| `wakeup_alarm` | MISMATCH | 107-176 | 457-546 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `add_thread` | MISMATCH | ❌ |
| `find_thread` | MISMATCH | ❌ |
| `find_thread_mut` | MISMATCH | ❌ |
| `new` | MISMATCH | ❌ |
| `state` | MISMATCH | ❌ |
| `state_mut` | MISMATCH | ❌ |
| `terminate` | MISMATCH | ❌ |
| `wakeup` | MISMATCH | ❌ |
| `wakeup_alarm` | MISMATCH | ❌ |

## Inconsistent Structs

- `InterruptedProcess`: EXTRA_IN_VERUS
- `RunnableProcess`: EXTRA_IN_VERUS
- `SleepingProcess`: MISMATCH
