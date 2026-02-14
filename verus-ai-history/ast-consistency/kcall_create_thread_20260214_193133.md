# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/kcall/create_thread.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/kcall/create_thread.rs`

## Summary

- Functions matched: 0/1
- Functions mismatched: 0
- Missing in Verus: 1
- Extra in Verus: 5
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `create_thread` | MISSING_IN_VERUS | 56-132 |  |
| `assert_user_stack_size` | EXTRA_IN_VERUS |  | 578-582 |
| `copy_from_user` | EXTRA_IN_VERUS |  | 463-481 |
| `create_thread_model` | EXTRA_IN_VERUS |  | 620-847 |
| `is_user_addr` | EXTRA_IN_VERUS |  | 430-438 |
| `is_user_region` | EXTRA_IN_VERUS |  | 411-420 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `create_thread` | MISSING_IN_VERUS | ❌ |
| `assert_user_stack_size` | EXTRA_IN_VERUS | ❌ |
| `copy_from_user` | EXTRA_IN_VERUS | ❌ |
| `create_thread_model` | EXTRA_IN_VERUS | ❌ |
| `is_user_addr` | EXTRA_IN_VERUS | ❌ |
| `is_user_region` | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `ThreadCreateArgsModel`: EXTRA_IN_VERUS
