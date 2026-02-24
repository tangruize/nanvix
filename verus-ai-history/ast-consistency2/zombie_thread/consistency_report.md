# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/thread/zombie.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/thread/zombie.rs`

## Summary

- Functions matched: 1/6
- Functions mismatched: 5
- Missing in Verus: 0
- Extra in Verus: 0
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `from_state` [from_state.diff](from_state.diff) [from_state_source.rs](from_state_source.rs) [from_state_verus.rs](from_state_verus.rs) | MISMATCH | 58-60 | 93-104 |
| `harvest` [harvest.diff](harvest.diff) [harvest_source.rs](harvest_source.rs) [harvest_verus.rs](harvest_verus.rs) | MISMATCH | 110-112 | 153-165 |
| `id` [id.diff](id.diff) [id_source.rs](id_source.rs) [id_verus.rs](id_verus.rs) | MISMATCH | 71-73 | 111-118 |
| `status` [status.diff](status.diff) [status_source.rs](status_source.rs) [status_verus.rs](status_verus.rs) | MISMATCH | 123-125 | 172-179 |
| `thread_state` [thread_state.diff](thread_state.diff) [thread_state_source.rs](thread_state_source.rs) [thread_state_verus.rs](thread_state_verus.rs) | MISMATCH | 84-86 | 126-134 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `from_state` [from_state.diff](from_state.diff) [from_state_source.rs](from_state_source.rs) [from_state_verus.rs](from_state_verus.rs) | MISMATCH | ❌ |
| `harvest` [harvest.diff](harvest.diff) [harvest_source.rs](harvest_source.rs) [harvest_verus.rs](harvest_verus.rs) | MISMATCH | ❌ |
| `id` [id.diff](id.diff) [id_source.rs](id_source.rs) [id_verus.rs](id_verus.rs) | MISMATCH | ❌ |
| `status` [status.diff](status.diff) [status_source.rs](status_source.rs) [status_verus.rs](status_verus.rs) | MISMATCH | ❌ |
| `thread_state` [thread_state.diff](thread_state.diff) [thread_state_source.rs](thread_state_source.rs) [thread_state_verus.rs](thread_state_verus.rs) | MISMATCH | ❌ |
| `thread_state_mut` | MATCH | ✅ |

## Inconsistent Structs

- `ZombieThread` [struct_ZombieThread.diff](struct_ZombieThread.diff) [struct_ZombieThread_source.rs](struct_ZombieThread_source.rs) [struct_ZombieThread_verus.rs](struct_ZombieThread_verus.rs): MISMATCH
