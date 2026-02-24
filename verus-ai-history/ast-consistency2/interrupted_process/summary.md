# Exec Diff: interrupted

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/process/state/interrupted.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/process/state/interrupted.rs`

| Function | Status | Files |
|----------|--------|-------|
| `find_thread` | MISMATCH | find_thread_source.rs, find_thread_verus.rs, find_thread.diff |
| `find_thread_mut` | MISMATCH | find_thread_mut_source.rs, find_thread_mut_verus.rs, find_thread_mut.diff |
| `from_sleeping` | MISMATCH | from_sleeping_source.rs, from_sleeping_verus.rs, from_sleeping.diff |
| `interrupt` | MISMATCH | interrupt_source.rs, interrupt_verus.rs, interrupt.diff |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `resume` | MISMATCH | resume_source.rs, resume_verus.rs, resume.diff |
| `state` | MISMATCH | state_source.rs, state_verus.rs, state.diff |
| `state_mut` | MISMATCH | state_mut_source.rs, state_mut_verus.rs, state_mut.diff |
| `resume_with_valid_clock` | EXTRA_IN_VERUS | resume_with_valid_clock_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `InterruptedProcess` | MISMATCH | struct_InterruptedProcess_source.rs, struct_InterruptedProcess_verus.rs, struct_InterruptedProcess.diff |
| `RunnableProcess` | EXTRA_IN_VERUS | struct_RunnableProcess_verus.rs (EXTRA) |
