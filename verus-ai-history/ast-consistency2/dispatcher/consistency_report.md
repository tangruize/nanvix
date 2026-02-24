# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/kcall/dispatcher.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/kcall/dispatcher.rs`

## Summary

- Functions matched: 0/2
- Functions mismatched: 2
- Missing in Verus: 0
- Extra in Verus: 36
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `do_kcall` [do_kcall.diff](do_kcall.diff) | [do_kcall_source.rs](do_kcall_source.rs) | [do_kcall_verus.rs](do_kcall_verus.rs) | MISMATCH | 54-146 | 1102-1123 |
| `handle_sleep_error` [handle_sleep_error.diff](handle_sleep_error.diff) | [handle_sleep_error_source.rs](handle_sleep_error_source.rs) | [handle_sleep_error_verus.rs](handle_sleep_error_verus.rs) | MISMATCH | 148-167 | 496-517 |
| `classify_kcall_number` [classify_kcall_number_verus.rs](classify_kcall_number_verus.rs) | EXTRA_IN_VERUS |  | 404-427 |
| `convert_fallible` [convert_fallible_verus.rs](convert_fallible_verus.rs) | EXTRA_IN_VERUS |  | 886-899 |
| `convert_sleepable` [convert_sleepable_verus.rs](convert_sleepable_verus.rs) | EXTRA_IN_VERUS |  | 837-869 |
| `diverge_after_exit` [diverge_after_exit_verus.rs](diverge_after_exit_verus.rs) | EXTRA_IN_VERUS |  | 570-575 |
| `do_kcall_abi` [do_kcall_abi_verus.rs](do_kcall_abi_verus.rs) | EXTRA_IN_VERUS |  | 1226-1243 |
| `do_kcall_context` [do_kcall_context_verus.rs](do_kcall_context_verus.rs) | EXTRA_IN_VERUS |  | 1043-1073 |
| `do_kcall_dispatch` [do_kcall_dispatch_verus.rs](do_kcall_dispatch_verus.rs) | EXTRA_IN_VERUS |  | 924-1000 |
| `do_kcall_encoded` [do_kcall_encoded_verus.rs](do_kcall_encoded_verus.rs) | EXTRA_IN_VERUS |  | 1173-1201 |
| `encode_result` [encode_result_verus.rs](encode_result_verus.rs) | EXTRA_IN_VERUS |  | 1147-1152 |
| `error` [error_verus.rs](error_verus.rs) | EXTRA_IN_VERUS |  | 297-305 |
| `event_resume` [event_resume_verus.rs](event_resume_verus.rs) | EXTRA_IN_VERUS |  | 658-660 |
| `generic` [generic_verus.rs](generic_verus.rs) | EXTRA_IN_VERUS |  | 350-357 |
| `handle_sleep_error_killed` [handle_sleep_error_killed_verus.rs](handle_sleep_error_killed_verus.rs) | EXTRA_IN_VERUS |  | 539-545 |
| `interrupted_killed` [interrupted_killed_verus.rs](interrupted_killed_verus.rs) | EXTRA_IN_VERUS |  | 364-370 |
| `interrupted_timed_out` [interrupted_timed_out_verus.rs](interrupted_timed_out_verus.rs) | EXTRA_IN_VERUS |  | 377-383 |
| `ipc_recv` [ipc_recv_verus.rs](ipc_recv_verus.rs) | EXTRA_IN_VERUS |  | 648-650 |
| `is_locally_handled` [is_locally_handled_verus.rs](is_locally_handled_verus.rs) | EXTRA_IN_VERUS |  | 443-449 |
| `is_sleepable` [is_sleepable_verus.rs](is_sleepable_verus.rs) | EXTRA_IN_VERUS |  | 464-470 |
| `new` [new_verus.rs](new_verus.rs) | EXTRA_IN_VERUS |  | 323-333 |
| `ok` [ok_verus.rs](ok_verus.rs) | EXTRA_IN_VERUS |  | 259-267 |
| `pm_exit` [pm_exit_verus.rs](pm_exit_verus.rs) | EXTRA_IN_VERUS |  | 618-620 |
| `pm_exit_interrupted` [pm_exit_interrupted_verus.rs](pm_exit_interrupted_verus.rs) | EXTRA_IN_VERUS |  | 557-559 |
| `pm_exit_thread` [pm_exit_thread_verus.rs](pm_exit_thread_verus.rs) | EXTRA_IN_VERUS |  | 628-630 |
| `pm_get_pid` [pm_get_pid_verus.rs](pm_get_pid_verus.rs) | EXTRA_IN_VERUS |  | 597-599 |
| `pm_get_tid` [pm_get_tid_verus.rs](pm_get_tid_verus.rs) | EXTRA_IN_VERUS |  | 607-609 |
| `pm_giveup` [pm_giveup_verus.rs](pm_giveup_verus.rs) | EXTRA_IN_VERUS |  | 710-712 |
| `pm_join_thread` [pm_join_thread_verus.rs](pm_join_thread_verus.rs) | EXTRA_IN_VERUS |  | 638-640 |
| `pm_lock_mutex` [pm_lock_mutex_verus.rs](pm_lock_mutex_verus.rs) | EXTRA_IN_VERUS |  | 668-670 |
| `pm_signal_cond` [pm_signal_cond_verus.rs](pm_signal_cond_verus.rs) | EXTRA_IN_VERUS |  | 700-702 |
| `pm_sleep` [pm_sleep_verus.rs](pm_sleep_verus.rs) | EXTRA_IN_VERUS |  | 720-722 |
| `pm_unlock_mutex` [pm_unlock_mutex_verus.rs](pm_unlock_mutex_verus.rs) | EXTRA_IN_VERUS |  | 678-680 |
| `pm_wait_cond` [pm_wait_cond_verus.rs](pm_wait_cond_verus.rs) | EXTRA_IN_VERUS |  | 688-690 |
| `remote_dispatch_verified` [remote_dispatch_verified_verus.rs](remote_dispatch_verified_verus.rs) | EXTRA_IN_VERUS |  | 779-818 |
| `scoreboard_dispatch_call` [scoreboard_dispatch_call_verus.rs](scoreboard_dispatch_call_verus.rs) | EXTRA_IN_VERUS |  | 746-748 |
| `scoreboard_get_mut` [scoreboard_get_mut_verus.rs](scoreboard_get_mut_verus.rs) | EXTRA_IN_VERUS |  | 734-736 |
| `success` [success_verus.rs](success_verus.rs) | EXTRA_IN_VERUS |  | 278-286 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `do_kcall` [do_kcall.diff](do_kcall.diff) | [do_kcall_source.rs](do_kcall_source.rs) | [do_kcall_verus.rs](do_kcall_verus.rs) | MISMATCH | ❌ |
| `handle_sleep_error` [handle_sleep_error.diff](handle_sleep_error.diff) | [handle_sleep_error_source.rs](handle_sleep_error_source.rs) | [handle_sleep_error_verus.rs](handle_sleep_error_verus.rs) | MISMATCH | ❌ |
| `classify_kcall_number` [classify_kcall_number_verus.rs](classify_kcall_number_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `convert_fallible` [convert_fallible_verus.rs](convert_fallible_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `convert_sleepable` [convert_sleepable_verus.rs](convert_sleepable_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `diverge_after_exit` [diverge_after_exit_verus.rs](diverge_after_exit_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `do_kcall_abi` [do_kcall_abi_verus.rs](do_kcall_abi_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `do_kcall_context` [do_kcall_context_verus.rs](do_kcall_context_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `do_kcall_dispatch` [do_kcall_dispatch_verus.rs](do_kcall_dispatch_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `do_kcall_encoded` [do_kcall_encoded_verus.rs](do_kcall_encoded_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `encode_result` [encode_result_verus.rs](encode_result_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `error` [error_verus.rs](error_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `event_resume` [event_resume_verus.rs](event_resume_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `generic` [generic_verus.rs](generic_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `handle_sleep_error_killed` [handle_sleep_error_killed_verus.rs](handle_sleep_error_killed_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `interrupted_killed` [interrupted_killed_verus.rs](interrupted_killed_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `interrupted_timed_out` [interrupted_timed_out_verus.rs](interrupted_timed_out_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `ipc_recv` [ipc_recv_verus.rs](ipc_recv_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `is_locally_handled` [is_locally_handled_verus.rs](is_locally_handled_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `is_sleepable` [is_sleepable_verus.rs](is_sleepable_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `new` [new_verus.rs](new_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `ok` [ok_verus.rs](ok_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `pm_exit` [pm_exit_verus.rs](pm_exit_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `pm_exit_interrupted` [pm_exit_interrupted_verus.rs](pm_exit_interrupted_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `pm_exit_thread` [pm_exit_thread_verus.rs](pm_exit_thread_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `pm_get_pid` [pm_get_pid_verus.rs](pm_get_pid_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `pm_get_tid` [pm_get_tid_verus.rs](pm_get_tid_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `pm_giveup` [pm_giveup_verus.rs](pm_giveup_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `pm_join_thread` [pm_join_thread_verus.rs](pm_join_thread_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `pm_lock_mutex` [pm_lock_mutex_verus.rs](pm_lock_mutex_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `pm_signal_cond` [pm_signal_cond_verus.rs](pm_signal_cond_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `pm_sleep` [pm_sleep_verus.rs](pm_sleep_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `pm_unlock_mutex` [pm_unlock_mutex_verus.rs](pm_unlock_mutex_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `pm_wait_cond` [pm_wait_cond_verus.rs](pm_wait_cond_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `remote_dispatch_verified` [remote_dispatch_verified_verus.rs](remote_dispatch_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `scoreboard_dispatch_call` [scoreboard_dispatch_call_verus.rs](scoreboard_dispatch_call_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `scoreboard_get_mut` [scoreboard_get_mut_verus.rs](scoreboard_get_mut_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `success` [success_verus.rs](success_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `DispatchArgs` [struct_DispatchArgs_verus.rs](struct_DispatchArgs_verus.rs): EXTRA_IN_VERUS
- `DispatchResult` [struct_DispatchResult_verus.rs](struct_DispatchResult_verus.rs): EXTRA_IN_VERUS
- `FallibleOutcome` [struct_FallibleOutcome_verus.rs](struct_FallibleOutcome_verus.rs): EXTRA_IN_VERUS
- `ScoreboardDispatchOutcome` [struct_ScoreboardDispatchOutcome_verus.rs](struct_ScoreboardDispatchOutcome_verus.rs): EXTRA_IN_VERUS
- `SleepError` [struct_SleepError_verus.rs](struct_SleepError_verus.rs): EXTRA_IN_VERUS
- `SleepableOutcome` [struct_SleepableOutcome_verus.rs](struct_SleepableOutcome_verus.rs): EXTRA_IN_VERUS
