# Exec Diff: mod

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/thread/mod.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/thread/mod.rs`

| Function | Status | Files |
|----------|--------|-------|
| `create_thread` | MISMATCH | create_thread_source.rs, create_thread_verus.rs, create_thread.diff |
| `init` | MISMATCH | init_source.rs, init_verus.rs, init.diff |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `thread_state` | MISMATCH | thread_state_source.rs, thread_state_verus.rs, thread_state.diff |
| `thread_state_mut` | MISSING_IN_VERUS | thread_state_mut_source.rs (MISSING in verus) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `ReadyThread` | EXTRA_IN_VERUS | struct_ReadyThread_verus.rs (EXTRA) |
| `ThreadManager` | MISMATCH | struct_ThreadManager_source.rs, struct_ThreadManager_verus.rs, struct_ThreadManager.diff |
