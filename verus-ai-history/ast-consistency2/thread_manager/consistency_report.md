# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/thread/mod.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/thread/mod.rs`

## Summary

- Functions matched: 0/5
- Functions mismatched: 0
- Missing in Verus: 5
- Extra in Verus: 0
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `create_thread` [create_thread.diff](create_thread.diff) | [create_thread_source.rs](create_thread_source.rs) | [create_thread_verus.rs](create_thread_verus.rs) | MISSING_IN_VERUS | 194-213 |  |
| `init` [init.diff](init.diff) | [init_source.rs](init_source.rs) | [init_verus.rs](init_verus.rs) | MISSING_IN_VERUS | 229-233 |  |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISSING_IN_VERUS | 158-176 |  |
| `thread_state` [thread_state.diff](thread_state.diff) | [thread_state_source.rs](thread_state_source.rs) | [thread_state_verus.rs](thread_state_verus.rs) | MISSING_IN_VERUS | 80-88 |  |
| `thread_state_mut` [thread_state_mut_source.rs](thread_state_mut_source.rs) | MISSING_IN_VERUS | 123-131 |  |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `create_thread` [create_thread.diff](create_thread.diff) | [create_thread_source.rs](create_thread_source.rs) | [create_thread_verus.rs](create_thread_verus.rs) | MISSING_IN_VERUS | ❌ |
| `init` [init.diff](init.diff) | [init_source.rs](init_source.rs) | [init_verus.rs](init_verus.rs) | MISSING_IN_VERUS | ❌ |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISSING_IN_VERUS | ❌ |
| `thread_state` [thread_state.diff](thread_state.diff) | [thread_state_source.rs](thread_state_source.rs) | [thread_state_verus.rs](thread_state_verus.rs) | MISSING_IN_VERUS | ❌ |
| `thread_state_mut` [thread_state_mut_source.rs](thread_state_mut_source.rs) | MISSING_IN_VERUS | ❌ |

## Inconsistent Structs

- `ThreadManager` [struct_ThreadManager.diff](struct_ThreadManager.diff) | [struct_ThreadManager_source.rs](struct_ThreadManager_source.rs) | [struct_ThreadManager_verus.rs](struct_ThreadManager_verus.rs): MISSING_IN_VERUS
