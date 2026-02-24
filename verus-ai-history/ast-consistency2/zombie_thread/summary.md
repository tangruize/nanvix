# Exec Diff: zombie

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/thread/zombie.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/thread/zombie.rs`

| Function | Status | Files |
|----------|--------|-------|
| `from_state` | MISMATCH | from_state_source.rs, from_state_verus.rs, from_state.diff |
| `harvest` | MISMATCH | harvest_source.rs, harvest_verus.rs, harvest.diff |
| `id` | MISMATCH | id_source.rs, id_verus.rs, id.diff |
| `status` | MISMATCH | status_source.rs, status_verus.rs, status.diff |
| `thread_state` | MISMATCH | thread_state_source.rs, thread_state_verus.rs, thread_state.diff |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `ZombieThread` | MISMATCH | struct_ZombieThread_source.rs, struct_ZombieThread_verus.rs, struct_ZombieThread.diff |
