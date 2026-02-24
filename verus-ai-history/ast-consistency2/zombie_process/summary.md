# Exec Diff: zombie

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/process/state/zombie.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/process/state/zombie.rs`

| Function | Status | Files |
|----------|--------|-------|
| `bury` | MISMATCH | bury_source.rs, bury_verus.rs, bury.diff |
| `find_thread` | MISMATCH | find_thread_source.rs, find_thread_verus.rs, find_thread.diff |
| `find_thread_mut` | MISMATCH | find_thread_mut_source.rs, find_thread_mut_verus.rs, find_thread_mut.diff |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `state` | MISMATCH | state_source.rs, state_verus.rs, state.diff |
| `state_mut` | MISMATCH | state_mut_source.rs, state_mut_verus.rs, state_mut.diff |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `ZombieProcess` | MISMATCH | struct_ZombieProcess_source.rs, struct_ZombieProcess_verus.rs, struct_ZombieProcess.diff |
