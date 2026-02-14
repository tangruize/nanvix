# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/process/state/interrupted.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/process/state/interrupted.rs`

## Summary

- Functions matched: 0/8
- Functions mismatched: 8
- Missing in Verus: 0
- Extra in Verus: 1
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `find_thread` | MISMATCH | 110-135 | 510-520 |
| `find_thread_mut` | MISMATCH | 151-179 | 535-551 |
| `from_sleeping` | MISMATCH | 60-72 | 210-240 |
| `interrupt` | MISMATCH | 182-184 | 572-578 |
| `new` | MISMATCH | 47-58 | 167-193 |
| `resume` | MISMATCH | 82-94 | 317-436 |
| `state` | MISMATCH | 74-76 | 249-256 |
| `state_mut` | MISMATCH | 78-80 | 268-283 |
| `resume_with_valid_clock` | EXTRA_IN_VERUS |  | 458-480 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `find_thread` | MISMATCH | ❌ |
| `find_thread_mut` | MISMATCH | ❌ |
| `from_sleeping` | MISMATCH | ❌ |
| `interrupt` | MISMATCH | ❌ |
| `new` | MISMATCH | ❌ |
| `resume` | MISMATCH | ❌ |
| `state` | MISMATCH | ❌ |
| `state_mut` | MISMATCH | ❌ |
| `resume_with_valid_clock` | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `InterruptedProcess`: MISMATCH
- `RunnableProcess`: EXTRA_IN_VERUS
