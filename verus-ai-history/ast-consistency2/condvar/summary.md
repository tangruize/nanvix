# Exec Diff: condvar

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/sync/condvar.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/sync/condvar.rs`

| Function | Status | Files |
|----------|--------|-------|
| `drop` | MISSING_IN_VERUS | drop_source.rs (MISSING in verus) |
| `fmt` | MISSING_IN_VERUS | fmt_source.rs (MISSING in verus) |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `notify_all` | MISMATCH | notify_all_source.rs, notify_all_verus.rs, notify_all.diff |
| `notify_first` | MISMATCH | notify_first_source.rs, notify_first_verus.rs, notify_first.diff |
| `notify_process` | MISMATCH | notify_process_source.rs, notify_process_verus.rs, notify_process.diff |
| `notify_thread` | MISMATCH | notify_thread_source.rs, notify_thread_verus.rs, notify_thread.diff |
| `reference_count` | MISSING_IN_VERUS | reference_count_source.rs (MISSING in verus) |
| `wait` | MISMATCH | wait_source.rs, wait_verus.rs, wait.diff |
| `clear` | EXTRA_IN_VERUS | clear_verus.rs (EXTRA) |
| `dequeue_first` | EXTRA_IN_VERUS | dequeue_first_verus.rs (EXTRA) |
| `drop_check` | EXTRA_IN_VERUS | drop_check_verus.rs (EXTRA) |
| `enqueue` | EXTRA_IN_VERUS | enqueue_verus.rs (EXTRA) |
| `get_len` | EXTRA_IN_VERUS | get_len_verus.rs (EXTRA) |
| `is_empty` | EXTRA_IN_VERUS | is_empty_verus.rs (EXTRA) |
| `remove_at` | EXTRA_IN_VERUS | remove_at_verus.rs (EXTRA) |
| `remove_by_pid` | EXTRA_IN_VERUS | remove_by_pid_verus.rs (EXTRA) |
| `remove_by_tid` | EXTRA_IN_VERUS | remove_by_tid_verus.rs (EXTRA) |
| `remove_entry` | EXTRA_IN_VERUS | remove_entry_verus.rs (EXTRA) |
| `try_enqueue` | EXTRA_IN_VERUS | try_enqueue_verus.rs (EXTRA) |
| `try_remove_by_pid` | EXTRA_IN_VERUS | try_remove_by_pid_verus.rs (EXTRA) |
| `try_remove_by_tid` | EXTRA_IN_VERUS | try_remove_by_tid_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `Condvar` | MISMATCH | struct_Condvar_source.rs, struct_Condvar_verus.rs, struct_Condvar.diff |
| `CondvarInner` | MISSING_IN_VERUS | struct_CondvarInner_source.rs (MISSING in verus) |
