# Exec Diff: mod

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/process/state/mod.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/process/state/mod.rs`

| Function | Status | Files |
|----------|--------|-------|
| `add_event` | MISSING_IN_VERUS | add_event_source.rs (MISSING in verus) |
| `add_mmio` | MISSING_IN_VERUS | add_mmio_source.rs (MISSING in verus) |
| `add_pmio` | MISMATCH | add_pmio_source.rs, add_pmio_verus.rs, add_pmio.diff |
| `clear_capability` | MISMATCH | clear_capability_source.rs, clear_capability_verus.rs, clear_capability.diff |
| `copy_from_user_unaligned` | MISSING_IN_VERUS | copy_from_user_unaligned_source.rs (MISSING in verus) |
| `copy_to_user_unaligned` | MISSING_IN_VERUS | copy_to_user_unaligned_source.rs (MISSING in verus) |
| `fmt` | MISSING_IN_VERUS | fmt_source.rs (MISSING in verus) |
| `get_cond` | MISMATCH | get_cond_source.rs, get_cond_verus.rs, get_cond.diff |
| `get_mutex` | MISMATCH | get_mutex_source.rs, get_mutex_verus.rs, get_mutex.diff |
| `get_pmio` | MISSING_IN_VERUS | get_pmio_source.rs (MISSING in verus) |
| `get_pmio_mut` | MISSING_IN_VERUS | get_pmio_mut_source.rs (MISSING in verus) |
| `has_capability` | MISMATCH | has_capability_source.rs, has_capability_verus.rs, has_capability.diff |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `pid` | MISMATCH | pid_source.rs, pid_verus.rs, pid.diff |
| `post_message` | MISSING_IN_VERUS | post_message_source.rs (MISSING in verus) |
| `put_cond` | MISMATCH | put_cond_source.rs, put_cond_verus.rs, put_cond.diff |
| `put_mutex` | MISMATCH | put_mutex_source.rs, put_mutex_verus.rs, put_mutex.diff |
| `read_pmio` | MISSING_IN_VERUS | read_pmio_source.rs (MISSING in verus) |
| `receive_message` | MISSING_IN_VERUS | receive_message_source.rs (MISSING in verus) |
| `remove_event` | MISSING_IN_VERUS | remove_event_source.rs (MISSING in verus) |
| `remove_mmio` | MISSING_IN_VERUS | remove_mmio_source.rs (MISSING in verus) |
| `remove_pmio` | MISMATCH | remove_pmio_source.rs, remove_pmio_verus.rs, remove_pmio.diff |
| `set_capability` | MISMATCH | set_capability_source.rs, set_capability_verus.rs, set_capability.diff |
| `state` | MISSING_IN_VERUS | state_source.rs (MISSING in verus) |
| `state_mut` | MISSING_IN_VERUS | state_mut_source.rs (MISSING in verus) |
| `vmem` | MISSING_IN_VERUS | vmem_source.rs (MISSING in verus) |
| `vmem_mut` | MISSING_IN_VERUS | vmem_mut_source.rs (MISSING in verus) |
| `write_pmio` | MISSING_IN_VERUS | write_pmio_source.rs (MISSING in verus) |
| `COND_MAX_EXEC` | EXTRA_IN_VERUS | COND_MAX_EXEC_verus.rs (EXTRA) |
| `MUTEX_MAX_EXEC` | EXTRA_IN_VERUS | MUTEX_MAX_EXEC_verus.rs (EXTRA) |
| `add_event_stub` | EXTRA_IN_VERUS | add_event_stub_verus.rs (EXTRA) |
| `add_mmio_stub` | EXTRA_IN_VERUS | add_mmio_stub_verus.rs (EXTRA) |
| `copy_from_user_unaligned_stub` | EXTRA_IN_VERUS | copy_from_user_unaligned_stub_verus.rs (EXTRA) |
| `copy_to_user_unaligned_stub` | EXTRA_IN_VERUS | copy_to_user_unaligned_stub_verus.rs (EXTRA) |
| `debug_fmt_stub` | EXTRA_IN_VERUS | debug_fmt_stub_verus.rs (EXTRA) |
| `get_pmio_mut_stub` | EXTRA_IN_VERUS | get_pmio_mut_stub_verus.rs (EXTRA) |
| `get_pmio_stub` | EXTRA_IN_VERUS | get_pmio_stub_verus.rs (EXTRA) |
| `post_message_stub` | EXTRA_IN_VERUS | post_message_stub_verus.rs (EXTRA) |
| `read_pmio_stub` | EXTRA_IN_VERUS | read_pmio_stub_verus.rs (EXTRA) |
| `receive_message_stub` | EXTRA_IN_VERUS | receive_message_stub_verus.rs (EXTRA) |
| `remove_event_stub` | EXTRA_IN_VERUS | remove_event_stub_verus.rs (EXTRA) |
| `remove_mmio_stub` | EXTRA_IN_VERUS | remove_mmio_stub_verus.rs (EXTRA) |
| `state_mut_stub` | EXTRA_IN_VERUS | state_mut_stub_verus.rs (EXTRA) |
| `state_stub` | EXTRA_IN_VERUS | state_stub_verus.rs (EXTRA) |
| `vmem_mut_stub` | EXTRA_IN_VERUS | vmem_mut_stub_verus.rs (EXTRA) |
| `vmem_stub` | EXTRA_IN_VERUS | vmem_stub_verus.rs (EXTRA) |
| `write_pmio_stub` | EXTRA_IN_VERUS | write_pmio_stub_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `ProcessRef` | EXTRA_IN_VERUS | struct_ProcessRef_verus.rs (EXTRA) |
| `ProcessRefMut` | EXTRA_IN_VERUS | struct_ProcessRefMut_verus.rs (EXTRA) |
| `ProcessState` | MISMATCH | struct_ProcessState_source.rs, struct_ProcessState_verus.rs, struct_ProcessState.diff |
