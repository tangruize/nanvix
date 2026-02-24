# Exec Diff: interrupted

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/thread/interrupted.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/thread/interrupted.rs`

| Function | Status | Files |
|----------|--------|-------|
| `from_state` | MISMATCH | from_state_source.rs, from_state_verus.rs, from_state.diff |
| `id` | MISMATCH | id_source.rs, id_verus.rs, id.diff |
| `join_cond` | MISMATCH | join_cond_source.rs, join_cond_verus.rs, join_cond.diff |
| `resume` | MISMATCH | resume_source.rs, resume_verus.rs, resume.diff |
| `thread_state` | MISMATCH | thread_state_source.rs, thread_state_verus.rs, thread_state.diff |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `Condvar` | EXTRA_IN_VERUS | struct_Condvar_verus.rs (EXTRA) |
| `InterruptedThread` | MISMATCH | struct_InterruptedThread_source.rs, struct_InterruptedThread_verus.rs, struct_InterruptedThread.diff |
| `ReadyThread` | EXTRA_IN_VERUS | struct_ReadyThread_verus.rs (EXTRA) |
