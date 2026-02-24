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
| `create_thread` [create_thread.diff](create_thread.diff) [create_thread_source.rs](create_thread_source.rs) [create_thread_verus.rs](create_thread_verus.rs) | MISSING_IN_VERUS | 56-132 |  |
| `assert_user_stack_size` [assert_user_stack_size_verus.rs](assert_user_stack_size_verus.rs) | EXTRA_IN_VERUS |  | 578-582 |
| `copy_from_user` [copy_from_user_verus.rs](copy_from_user_verus.rs) | EXTRA_IN_VERUS |  | 463-481 |
| `create_thread_model` [create_thread_model_verus.rs](create_thread_model_verus.rs) | EXTRA_IN_VERUS |  | 620-847 |
| `is_user_addr` [is_user_addr_verus.rs](is_user_addr_verus.rs) | EXTRA_IN_VERUS |  | 430-438 |
| `is_user_region` [is_user_region_verus.rs](is_user_region_verus.rs) | EXTRA_IN_VERUS |  | 411-420 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `create_thread` [create_thread.diff](create_thread.diff) [create_thread_source.rs](create_thread_source.rs) [create_thread_verus.rs](create_thread_verus.rs) | MISSING_IN_VERUS | ❌ |
| `assert_user_stack_size` [assert_user_stack_size_verus.rs](assert_user_stack_size_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `copy_from_user` [copy_from_user_verus.rs](copy_from_user_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `create_thread_model` [create_thread_model_verus.rs](create_thread_model_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `is_user_addr` [is_user_addr_verus.rs](is_user_addr_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `is_user_region` [is_user_region_verus.rs](is_user_region_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `ThreadCreateArgsModel` [struct_ThreadCreateArgsModel_verus.rs](struct_ThreadCreateArgsModel_verus.rs): EXTRA_IN_VERUS
