# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/sync/fence.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/sync/fence.rs`

## Summary

- Functions matched: 0/3
- Functions mismatched: 3
- Missing in Verus: 0
- Extra in Verus: 3
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | 43-48 | 152-163 |
| `signal` [signal.diff](signal.diff) [signal_source.rs](signal_source.rs) [signal_verus.rs](signal_verus.rs) | MISMATCH | 66-68 | 201-214 |
| `wait` [wait.diff](wait.diff) [wait_source.rs](wait_source.rs) [wait_verus.rs](wait_verus.rs) | MISMATCH | 55-59 | 179-188 |
| `get_count` [get_count_verus.rs](get_count_verus.rs) | EXTRA_IN_VERUS |  | 243-248 |
| `get_total` [get_total_verus.rs](get_total_verus.rs) | EXTRA_IN_VERUS |  | 259-264 |
| `is_satisfied` [is_satisfied_verus.rs](is_satisfied_verus.rs) | EXTRA_IN_VERUS |  | 225-232 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | ❌ |
| `signal` [signal.diff](signal.diff) [signal_source.rs](signal_source.rs) [signal_verus.rs](signal_verus.rs) | MISMATCH | ❌ |
| `wait` [wait.diff](wait.diff) [wait_source.rs](wait_source.rs) [wait_verus.rs](wait_verus.rs) | MISMATCH | ❌ |
| `get_count` [get_count_verus.rs](get_count_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `get_total` [get_total_verus.rs](get_total_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `is_satisfied` [is_satisfied_verus.rs](is_satisfied_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `Fence` [struct_Fence.diff](struct_Fence.diff) [struct_Fence_source.rs](struct_Fence_source.rs) [struct_Fence_verus.rs](struct_Fence_verus.rs): MISMATCH
