# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/sync/condvar.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/sync/condvar.rs`

## Summary

- Functions matched: 0/9
- Functions mismatched: 1
- Missing in Verus: 8
- Extra in Verus: 12
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `drop` [drop_source.rs](drop_source.rs) | MISSING_IN_VERUS | 346-350 |  |
| `fmt` [fmt_source.rs](fmt_source.rs) | MISSING_IN_VERUS | 340-342 |  |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISMATCH | 70-76 | 223-230 |
| `notify_all` [notify_all.diff](notify_all.diff) | [notify_all_source.rs](notify_all_source.rs) | [notify_all_verus.rs](notify_all_verus.rs) | MISSING_IN_VERUS | 239-261 |  |
| `notify_first` [notify_first.diff](notify_first.diff) | [notify_first_source.rs](notify_first_source.rs) | [notify_first_verus.rs](notify_first_verus.rs) | MISSING_IN_VERUS | 115-125 |  |
| `notify_process` [notify_process.diff](notify_process.diff) | [notify_process_source.rs](notify_process_source.rs) | [notify_process_verus.rs](notify_process_verus.rs) | MISSING_IN_VERUS | 148-170 |  |
| `notify_thread` [notify_thread.diff](notify_thread.diff) | [notify_thread_source.rs](notify_thread_source.rs) | [notify_thread_verus.rs](notify_thread_verus.rs) | MISSING_IN_VERUS | 193-216 |  |
| `reference_count` [reference_count_source.rs](reference_count_source.rs) | MISSING_IN_VERUS | 93-95 |  |
| `wait` [wait.diff](wait.diff) | [wait_source.rs](wait_source.rs) | [wait_verus.rs](wait_verus.rs) | MISSING_IN_VERUS | 284-326 |  |
| `clear` [clear_verus.rs](clear_verus.rs) | EXTRA_IN_VERUS |  | 673-689 |
| `dequeue_first` [dequeue_first_verus.rs](dequeue_first_verus.rs) | EXTRA_IN_VERUS |  | 360-382 |
| `enqueue` [enqueue_verus.rs](enqueue_verus.rs) | EXTRA_IN_VERUS |  | 243-299 |
| `get_len` [get_len_verus.rs](get_len_verus.rs) | EXTRA_IN_VERUS |  | 710-717 |
| `is_empty` [is_empty_verus.rs](is_empty_verus.rs) | EXTRA_IN_VERUS |  | 696-703 |
| `remove_at` [remove_at_verus.rs](remove_at_verus.rs) | EXTRA_IN_VERUS |  | 400-437 |
| `remove_by_pid` [remove_by_pid_verus.rs](remove_by_pid_verus.rs) | EXTRA_IN_VERUS |  | 494-511 |
| `remove_by_tid` [remove_by_tid_verus.rs](remove_by_tid_verus.rs) | EXTRA_IN_VERUS |  | 530-547 |
| `remove_entry` [remove_entry_verus.rs](remove_entry_verus.rs) | EXTRA_IN_VERUS |  | 463-475 |
| `try_enqueue` [try_enqueue_verus.rs](try_enqueue_verus.rs) | EXTRA_IN_VERUS |  | 320-348 |
| `try_remove_by_pid` [try_remove_by_pid_verus.rs](try_remove_by_pid_verus.rs) | EXTRA_IN_VERUS |  | 574-604 |
| `try_remove_by_tid` [try_remove_by_tid_verus.rs](try_remove_by_tid_verus.rs) | EXTRA_IN_VERUS |  | 625-655 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `drop` [drop_source.rs](drop_source.rs) | MISSING_IN_VERUS | ❌ |
| `fmt` [fmt_source.rs](fmt_source.rs) | MISSING_IN_VERUS | ❌ |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISMATCH | ❌ |
| `notify_all` [notify_all.diff](notify_all.diff) | [notify_all_source.rs](notify_all_source.rs) | [notify_all_verus.rs](notify_all_verus.rs) | MISSING_IN_VERUS | ❌ |
| `notify_first` [notify_first.diff](notify_first.diff) | [notify_first_source.rs](notify_first_source.rs) | [notify_first_verus.rs](notify_first_verus.rs) | MISSING_IN_VERUS | ❌ |
| `notify_process` [notify_process.diff](notify_process.diff) | [notify_process_source.rs](notify_process_source.rs) | [notify_process_verus.rs](notify_process_verus.rs) | MISSING_IN_VERUS | ❌ |
| `notify_thread` [notify_thread.diff](notify_thread.diff) | [notify_thread_source.rs](notify_thread_source.rs) | [notify_thread_verus.rs](notify_thread_verus.rs) | MISSING_IN_VERUS | ❌ |
| `reference_count` [reference_count_source.rs](reference_count_source.rs) | MISSING_IN_VERUS | ❌ |
| `wait` [wait.diff](wait.diff) | [wait_source.rs](wait_source.rs) | [wait_verus.rs](wait_verus.rs) | MISSING_IN_VERUS | ❌ |
| `clear` [clear_verus.rs](clear_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `dequeue_first` [dequeue_first_verus.rs](dequeue_first_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `enqueue` [enqueue_verus.rs](enqueue_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `get_len` [get_len_verus.rs](get_len_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `is_empty` [is_empty_verus.rs](is_empty_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `remove_at` [remove_at_verus.rs](remove_at_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `remove_by_pid` [remove_by_pid_verus.rs](remove_by_pid_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `remove_by_tid` [remove_by_tid_verus.rs](remove_by_tid_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `remove_entry` [remove_entry_verus.rs](remove_entry_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `try_enqueue` [try_enqueue_verus.rs](try_enqueue_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `try_remove_by_pid` [try_remove_by_pid_verus.rs](try_remove_by_pid_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `try_remove_by_tid` [try_remove_by_tid_verus.rs](try_remove_by_tid_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `Condvar` [struct_Condvar.diff](struct_Condvar.diff) | [struct_Condvar_source.rs](struct_Condvar_source.rs) | [struct_Condvar_verus.rs](struct_Condvar_verus.rs): MISMATCH
- `CondvarInner` [struct_CondvarInner_source.rs](struct_CondvarInner_source.rs): MISSING_IN_VERUS
