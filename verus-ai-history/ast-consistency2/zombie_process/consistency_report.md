# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/process/state/zombie.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/process/state/zombie.rs`

## Summary

- Functions matched: 0/6
- Functions mismatched: 6
- Missing in Verus: 0
- Extra in Verus: 0
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `bury` [bury.diff](bury.diff) [bury_source.rs](bury_source.rs) [bury_verus.rs](bury_verus.rs) | MISMATCH | 61-63 | 232-242 |
| `find_thread` [find_thread.diff](find_thread.diff) [find_thread_source.rs](find_thread_source.rs) [find_thread_verus.rs](find_thread_verus.rs) | MISMATCH | 79-84 | 265-273 |
| `find_thread_mut` [find_thread_mut.diff](find_thread_mut.diff) [find_thread_mut_source.rs](find_thread_mut_source.rs) [find_thread_mut_verus.rs](find_thread_mut_verus.rs) | MISMATCH | 100-105 | 302-312 |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | 41-51 | 144-167 |
| `state` [state.diff](state.diff) [state_source.rs](state_source.rs) [state_verus.rs](state_verus.rs) | MISMATCH | 53-55 | 182-189 |
| `state_mut` [state_mut.diff](state_mut.diff) [state_mut_source.rs](state_mut_source.rs) [state_mut_verus.rs](state_mut_verus.rs) | MISMATCH | 57-59 | 209-218 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `bury` [bury.diff](bury.diff) [bury_source.rs](bury_source.rs) [bury_verus.rs](bury_verus.rs) | MISMATCH | ❌ |
| `find_thread` [find_thread.diff](find_thread.diff) [find_thread_source.rs](find_thread_source.rs) [find_thread_verus.rs](find_thread_verus.rs) | MISMATCH | ❌ |
| `find_thread_mut` [find_thread_mut.diff](find_thread_mut.diff) [find_thread_mut_source.rs](find_thread_mut_source.rs) [find_thread_mut_verus.rs](find_thread_mut_verus.rs) | MISMATCH | ❌ |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | ❌ |
| `state` [state.diff](state.diff) [state_source.rs](state_source.rs) [state_verus.rs](state_verus.rs) | MISMATCH | ❌ |
| `state_mut` [state_mut.diff](state_mut.diff) [state_mut_source.rs](state_mut_source.rs) [state_mut_verus.rs](state_mut_verus.rs) | MISMATCH | ❌ |

## Inconsistent Structs

- `ZombieProcess` [struct_ZombieProcess.diff](struct_ZombieProcess.diff) [struct_ZombieProcess_source.rs](struct_ZombieProcess_source.rs) [struct_ZombieProcess_verus.rs](struct_ZombieProcess_verus.rs): MISMATCH
