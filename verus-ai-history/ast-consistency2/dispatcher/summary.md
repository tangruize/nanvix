# Exec Diff: dispatcher

**Source:** `/home/ubuntu/nanvix/src/kernel/src/kcall/dispatcher.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/kcall/dispatcher.rs`

| Function | Status | Files |
|----------|--------|-------|
| `do_kcall` | MISMATCH | do_kcall_source.rs, do_kcall_verus.rs, do_kcall.diff |
| `handle_sleep_error` | MISMATCH | handle_sleep_error_source.rs, handle_sleep_error_verus.rs, handle_sleep_error.diff |
| `classify_kcall_number` | EXTRA_IN_VERUS | classify_kcall_number_verus.rs (EXTRA) |
| `convert_fallible` | EXTRA_IN_VERUS | convert_fallible_verus.rs (EXTRA) |
| `convert_sleepable` | EXTRA_IN_VERUS | convert_sleepable_verus.rs (EXTRA) |
| `diverge_after_exit` | EXTRA_IN_VERUS | diverge_after_exit_verus.rs (EXTRA) |
| `do_kcall_abi` | EXTRA_IN_VERUS | do_kcall_abi_verus.rs (EXTRA) |
| `do_kcall_context` | EXTRA_IN_VERUS | do_kcall_context_verus.rs (EXTRA) |
| `do_kcall_dispatch` | EXTRA_IN_VERUS | do_kcall_dispatch_verus.rs (EXTRA) |
| `do_kcall_encoded` | EXTRA_IN_VERUS | do_kcall_encoded_verus.rs (EXTRA) |
| `encode_result` | EXTRA_IN_VERUS | encode_result_verus.rs (EXTRA) |
| `error` | EXTRA_IN_VERUS | error_verus.rs (EXTRA) |
| `event_resume` | EXTRA_IN_VERUS | event_resume_verus.rs (EXTRA) |
| `generic` | EXTRA_IN_VERUS | generic_verus.rs (EXTRA) |
| `handle_sleep_error_killed` | EXTRA_IN_VERUS | handle_sleep_error_killed_verus.rs (EXTRA) |
| `interrupted_killed` | EXTRA_IN_VERUS | interrupted_killed_verus.rs (EXTRA) |
| `interrupted_timed_out` | EXTRA_IN_VERUS | interrupted_timed_out_verus.rs (EXTRA) |
| `ipc_recv` | EXTRA_IN_VERUS | ipc_recv_verus.rs (EXTRA) |
| `is_locally_handled` | EXTRA_IN_VERUS | is_locally_handled_verus.rs (EXTRA) |
| `is_sleepable` | EXTRA_IN_VERUS | is_sleepable_verus.rs (EXTRA) |
| `new` | EXTRA_IN_VERUS | new_verus.rs (EXTRA) |
| `ok` | EXTRA_IN_VERUS | ok_verus.rs (EXTRA) |
| `pm_exit` | EXTRA_IN_VERUS | pm_exit_verus.rs (EXTRA) |
| `pm_exit_interrupted` | EXTRA_IN_VERUS | pm_exit_interrupted_verus.rs (EXTRA) |
| `pm_exit_thread` | EXTRA_IN_VERUS | pm_exit_thread_verus.rs (EXTRA) |
| `pm_get_pid` | EXTRA_IN_VERUS | pm_get_pid_verus.rs (EXTRA) |
| `pm_get_tid` | EXTRA_IN_VERUS | pm_get_tid_verus.rs (EXTRA) |
| `pm_giveup` | EXTRA_IN_VERUS | pm_giveup_verus.rs (EXTRA) |
| `pm_join_thread` | EXTRA_IN_VERUS | pm_join_thread_verus.rs (EXTRA) |
| `pm_lock_mutex` | EXTRA_IN_VERUS | pm_lock_mutex_verus.rs (EXTRA) |
| `pm_signal_cond` | EXTRA_IN_VERUS | pm_signal_cond_verus.rs (EXTRA) |
| `pm_sleep` | EXTRA_IN_VERUS | pm_sleep_verus.rs (EXTRA) |
| `pm_unlock_mutex` | EXTRA_IN_VERUS | pm_unlock_mutex_verus.rs (EXTRA) |
| `pm_wait_cond` | EXTRA_IN_VERUS | pm_wait_cond_verus.rs (EXTRA) |
| `remote_dispatch_verified` | EXTRA_IN_VERUS | remote_dispatch_verified_verus.rs (EXTRA) |
| `scoreboard_dispatch_call` | EXTRA_IN_VERUS | scoreboard_dispatch_call_verus.rs (EXTRA) |
| `scoreboard_get_mut` | EXTRA_IN_VERUS | scoreboard_get_mut_verus.rs (EXTRA) |
| `success` | EXTRA_IN_VERUS | success_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `DispatchArgs` | EXTRA_IN_VERUS | struct_DispatchArgs_verus.rs (EXTRA) |
| `DispatchResult` | EXTRA_IN_VERUS | struct_DispatchResult_verus.rs (EXTRA) |
| `FallibleOutcome` | EXTRA_IN_VERUS | struct_FallibleOutcome_verus.rs (EXTRA) |
| `ScoreboardDispatchOutcome` | EXTRA_IN_VERUS | struct_ScoreboardDispatchOutcome_verus.rs (EXTRA) |
| `SleepError` | EXTRA_IN_VERUS | struct_SleepError_verus.rs (EXTRA) |
| `SleepableOutcome` | EXTRA_IN_VERUS | struct_SleepableOutcome_verus.rs (EXTRA) |
