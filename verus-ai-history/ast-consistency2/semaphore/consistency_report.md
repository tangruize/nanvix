# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/sync/semaphore.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/sync/semaphore.rs`

## Summary

- Functions matched: 0/4
- Functions mismatched: 3
- Missing in Verus: 1
- Extra in Verus: 4
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `down` [down.diff](down.diff) | [down_source.rs](down_source.rs) | [down_verus.rs](down_verus.rs) | MISSING_IN_VERUS | 83-101 |  |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISMATCH | 54-59 | 212-222 |
| `try_down` [try_down.diff](try_down.diff) | [try_down_source.rs](try_down_source.rs) | [try_down_verus.rs](try_down_verus.rs) | MISMATCH | 114-130 | 323-341 |
| `up` [up.diff](up.diff) | [up_source.rs](up_source.rs) | [up_verus.rs](up_verus.rs) | MISMATCH | 154-157 | 369-382 |
| `down_available` [down_available_verus.rs](down_available_verus.rs) | EXTRA_IN_VERUS |  | 246-258 |
| `down_or_block` [down_or_block_verus.rs](down_or_block_verus.rs) | EXTRA_IN_VERUS |  | 288-310 |
| `get_value` [get_value_verus.rs](get_value_verus.rs) | EXTRA_IN_VERUS |  | 394-402 |
| `is_available` [is_available_verus.rs](is_available_verus.rs) | EXTRA_IN_VERUS |  | 414-422 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `down` [down.diff](down.diff) | [down_source.rs](down_source.rs) | [down_verus.rs](down_verus.rs) | MISSING_IN_VERUS | ❌ |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISMATCH | ❌ |
| `try_down` [try_down.diff](try_down.diff) | [try_down_source.rs](try_down_source.rs) | [try_down_verus.rs](try_down_verus.rs) | MISMATCH | ❌ |
| `up` [up.diff](up.diff) | [up_source.rs](up_source.rs) | [up_verus.rs](up_verus.rs) | MISMATCH | ❌ |
| `down_available` [down_available_verus.rs](down_available_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `down_or_block` [down_or_block_verus.rs](down_or_block_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `get_value` [get_value_verus.rs](get_value_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `is_available` [is_available_verus.rs](is_available_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `Semaphore` [struct_Semaphore.diff](struct_Semaphore.diff) | [struct_Semaphore_source.rs](struct_Semaphore_source.rs) | [struct_Semaphore_verus.rs](struct_Semaphore_verus.rs): MISMATCH
