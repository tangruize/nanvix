# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/process/manager/mod.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/process/manager/mod.rs`

## Summary

- Functions matched: 0/53
- Functions mismatched: 0
- Missing in Verus: 53
- Extra in Verus: 0
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `add_event` [add_event_source.rs](add_event_source.rs) | MISSING_IN_VERUS | 1924-1931 |  |
| `attach_pmio` [attach_pmio_source.rs](attach_pmio_source.rs) | MISSING_IN_VERUS | 1853-1858 |  |
| `capctl` [capctl_source.rs](capctl_source.rs) | MISSING_IN_VERUS | 1698-1705 |  |
| `check_alarm` [check_alarm_source.rs](check_alarm_source.rs) | MISSING_IN_VERUS | 682-705 |  |
| `create_process` [create_process_source.rs](create_process_source.rs) | MISSING_IN_VERUS | 1578-1586 |  |
| `create_thread` [create_thread_source.rs](create_thread_source.rs) | MISSING_IN_VERUS | 1611-1622 |  |
| `detach_pmio` [detach_pmio_source.rs](detach_pmio_source.rs) | MISSING_IN_VERUS | 1860-1868 |  |
| `exit` [exit_source.rs](exit_source.rs) | MISSING_IN_VERUS | 895-946 |  |
| `exit_thread` [exit_thread_source.rs](exit_thread_source.rs) | MISSING_IN_VERUS | 974-1034 |  |
| `find_process` [find_process_source.rs](find_process_source.rs) | MISSING_IN_VERUS | 1389-1405 |  |
| `find_process_by_tid` [find_process_by_tid_source.rs](find_process_by_tid_source.rs) | MISSING_IN_VERUS | 1439-1467 |  |
| `find_process_mut` [find_process_mut_source.rs](find_process_mut_source.rs) | MISSING_IN_VERUS | 1407-1423 |  |
| `find_thread_mut` [find_thread_mut_source.rs](find_thread_mut_source.rs) | MISSING_IN_VERUS | 1483-1522 |  |
| `forge_user_context` [forge_user_context_source.rs](forge_user_context_source.rs) | MISSING_IN_VERUS | 203-244 |  |
| `get_cond` [get_cond_source.rs](get_cond_source.rs) | MISSING_IN_VERUS | 1275-1277 |  |
| `get_mutex` [get_mutex_source.rs](get_mutex_source.rs) | MISSING_IN_VERUS | 1256-1258 |  |
| `get_pid` [get_pid_source.rs](get_pid_source.rs) | MISSING_IN_VERUS | 1542-1545 |  |
| `get_running` [get_running_source.rs](get_running_source.rs) | MISSING_IN_VERUS | 1379-1382 |  |
| `get_running_mut` [get_running_mut_source.rs](get_running_mut_source.rs) | MISSING_IN_VERUS | 1384-1387 |  |
| `get_thread_data_area` [get_thread_data_area_source.rs](get_thread_data_area_source.rs) | MISSING_IN_VERUS | 1678-1684 |  |
| `get_tid` [get_tid_source.rs](get_tid_source.rs) | MISSING_IN_VERUS | 1557-1559 |  |
| `handle_fpu_exception` [handle_fpu_exception_source.rs](handle_fpu_exception_source.rs) | MISSING_IN_VERUS | 1957-1959 |  |
| `harvest_zombies` [harvest_zombies_source.rs](harvest_zombies_source.rs) | MISSING_IN_VERUS | 1737-1782 |  |
| `has_capability` [has_capability_source.rs](has_capability_source.rs) | MISSING_IN_VERUS | 1686-1696 |  |
| `interrupt_reason` [interrupt_reason_source.rs](interrupt_reason_source.rs) | MISSING_IN_VERUS | 1187-1189 |  |
| `mctrl` [mctrl_source.rs](mctrl_source.rs) | MISSING_IN_VERUS | 1809-1820 |  |
| `mmap` [mmap_source.rs](mmap_source.rs) | MISSING_IN_VERUS | 1784-1795 |  |
| `mmio_alloc` [mmio_alloc_source.rs](mmio_alloc_source.rs) | MISSING_IN_VERUS | 1822-1838 |  |
| `mmio_free` [mmio_free_source.rs](mmio_free_source.rs) | MISSING_IN_VERUS | 1840-1851 |  |
| `munmap` [munmap_source.rs](munmap_source.rs) | MISSING_IN_VERUS | 1797-1807 |  |
| `new` [new_source.rs](new_source.rs) | MISSING_IN_VERUS | 146-174 |  |
| `number_buffered_messages` [number_buffered_messages_source.rs](number_buffered_messages_source.rs) | MISSING_IN_VERUS | 1953-1955 |  |
| `post_message` [post_message_source.rs](post_message_source.rs) | MISSING_IN_VERUS | 1909-1922 |  |
| `put_cond` [put_cond_source.rs](put_cond_source.rs) | MISSING_IN_VERUS | 1292-1294 |  |
| `put_mutex_guard` [put_mutex_guard_source.rs](put_mutex_guard_source.rs) | MISSING_IN_VERUS | 1306-1310 |  |
| `read_pmio` [read_pmio_source.rs](read_pmio_source.rs) | MISSING_IN_VERUS | 1870-1879 |  |
| `remove_event` [remove_event_source.rs](remove_event_source.rs) | MISSING_IN_VERUS | 1933-1940 |  |
| `schedule` [schedule_source.rs](schedule_source.rs) | MISSING_IN_VERUS | 640-678 |  |
| `set_thread_data_area` [set_thread_data_area_source.rs](set_thread_data_area_source.rs) | MISSING_IN_VERUS | 1646-1654 |  |
| `sleep` [sleep_source.rs](sleep_source.rs) | MISSING_IN_VERUS | 725-771 |  |
| `take_earliest_ready` [take_earliest_ready_source.rs](take_earliest_ready_source.rs) | MISSING_IN_VERUS | 1352-1372 |  |
| `take_mutex_guard` [take_mutex_guard_source.rs](take_mutex_guard_source.rs) | MISSING_IN_VERUS | 1328-1350 |  |
| `take_running` [take_running_source.rs](take_running_source.rs) | MISSING_IN_VERUS | 1374-1377 |  |
| `terminate` [terminate_source.rs](terminate_source.rs) | MISSING_IN_VERUS | 1707-1709 |  |
| `try_add_thread` [try_add_thread_source.rs](try_add_thread_source.rs) | MISSING_IN_VERUS | 325-372 |  |
| `try_borrow` [try_borrow_source.rs](try_borrow_source.rs) | MISSING_IN_VERUS | 1961-1970 |  |
| `try_borrow_mut` [try_borrow_mut_source.rs](try_borrow_mut_source.rs) | MISSING_IN_VERUS | 1972-1981 |  |
| `try_join_thread` [try_join_thread_source.rs](try_join_thread_source.rs) | MISSING_IN_VERUS | 1211-1239 |  |
| `try_wakeup` [try_wakeup_source.rs](try_wakeup_source.rs) | MISSING_IN_VERUS | 819-875 |  |
| `vmcopy_from_user` [vmcopy_from_user_source.rs](vmcopy_from_user_source.rs) | MISSING_IN_VERUS | 1711-1722 |  |
| `vmcopy_to_user` [vmcopy_to_user_source.rs](vmcopy_to_user_source.rs) | MISSING_IN_VERUS | 1724-1735 |  |
| `wakeup` [wakeup_source.rs](wakeup_source.rs) | MISSING_IN_VERUS | 786-817 |  |
| `write_pmio` [write_pmio_source.rs](write_pmio_source.rs) | MISSING_IN_VERUS | 1881-1893 |  |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `add_event` [add_event_source.rs](add_event_source.rs) | MISSING_IN_VERUS | ❌ |
| `attach_pmio` [attach_pmio_source.rs](attach_pmio_source.rs) | MISSING_IN_VERUS | ❌ |
| `capctl` [capctl_source.rs](capctl_source.rs) | MISSING_IN_VERUS | ❌ |
| `check_alarm` [check_alarm_source.rs](check_alarm_source.rs) | MISSING_IN_VERUS | ❌ |
| `create_process` [create_process_source.rs](create_process_source.rs) | MISSING_IN_VERUS | ❌ |
| `create_thread` [create_thread_source.rs](create_thread_source.rs) | MISSING_IN_VERUS | ❌ |
| `detach_pmio` [detach_pmio_source.rs](detach_pmio_source.rs) | MISSING_IN_VERUS | ❌ |
| `exit` [exit_source.rs](exit_source.rs) | MISSING_IN_VERUS | ❌ |
| `exit_thread` [exit_thread_source.rs](exit_thread_source.rs) | MISSING_IN_VERUS | ❌ |
| `find_process` [find_process_source.rs](find_process_source.rs) | MISSING_IN_VERUS | ❌ |
| `find_process_by_tid` [find_process_by_tid_source.rs](find_process_by_tid_source.rs) | MISSING_IN_VERUS | ❌ |
| `find_process_mut` [find_process_mut_source.rs](find_process_mut_source.rs) | MISSING_IN_VERUS | ❌ |
| `find_thread_mut` [find_thread_mut_source.rs](find_thread_mut_source.rs) | MISSING_IN_VERUS | ❌ |
| `forge_user_context` [forge_user_context_source.rs](forge_user_context_source.rs) | MISSING_IN_VERUS | ❌ |
| `get_cond` [get_cond_source.rs](get_cond_source.rs) | MISSING_IN_VERUS | ❌ |
| `get_mutex` [get_mutex_source.rs](get_mutex_source.rs) | MISSING_IN_VERUS | ❌ |
| `get_pid` [get_pid_source.rs](get_pid_source.rs) | MISSING_IN_VERUS | ❌ |
| `get_running` [get_running_source.rs](get_running_source.rs) | MISSING_IN_VERUS | ❌ |
| `get_running_mut` [get_running_mut_source.rs](get_running_mut_source.rs) | MISSING_IN_VERUS | ❌ |
| `get_thread_data_area` [get_thread_data_area_source.rs](get_thread_data_area_source.rs) | MISSING_IN_VERUS | ❌ |
| `get_tid` [get_tid_source.rs](get_tid_source.rs) | MISSING_IN_VERUS | ❌ |
| `handle_fpu_exception` [handle_fpu_exception_source.rs](handle_fpu_exception_source.rs) | MISSING_IN_VERUS | ❌ |
| `harvest_zombies` [harvest_zombies_source.rs](harvest_zombies_source.rs) | MISSING_IN_VERUS | ❌ |
| `has_capability` [has_capability_source.rs](has_capability_source.rs) | MISSING_IN_VERUS | ❌ |
| `interrupt_reason` [interrupt_reason_source.rs](interrupt_reason_source.rs) | MISSING_IN_VERUS | ❌ |
| `mctrl` [mctrl_source.rs](mctrl_source.rs) | MISSING_IN_VERUS | ❌ |
| `mmap` [mmap_source.rs](mmap_source.rs) | MISSING_IN_VERUS | ❌ |
| `mmio_alloc` [mmio_alloc_source.rs](mmio_alloc_source.rs) | MISSING_IN_VERUS | ❌ |
| `mmio_free` [mmio_free_source.rs](mmio_free_source.rs) | MISSING_IN_VERUS | ❌ |
| `munmap` [munmap_source.rs](munmap_source.rs) | MISSING_IN_VERUS | ❌ |
| `new` [new_source.rs](new_source.rs) | MISSING_IN_VERUS | ❌ |
| `number_buffered_messages` [number_buffered_messages_source.rs](number_buffered_messages_source.rs) | MISSING_IN_VERUS | ❌ |
| `post_message` [post_message_source.rs](post_message_source.rs) | MISSING_IN_VERUS | ❌ |
| `put_cond` [put_cond_source.rs](put_cond_source.rs) | MISSING_IN_VERUS | ❌ |
| `put_mutex_guard` [put_mutex_guard_source.rs](put_mutex_guard_source.rs) | MISSING_IN_VERUS | ❌ |
| `read_pmio` [read_pmio_source.rs](read_pmio_source.rs) | MISSING_IN_VERUS | ❌ |
| `remove_event` [remove_event_source.rs](remove_event_source.rs) | MISSING_IN_VERUS | ❌ |
| `schedule` [schedule_source.rs](schedule_source.rs) | MISSING_IN_VERUS | ❌ |
| `set_thread_data_area` [set_thread_data_area_source.rs](set_thread_data_area_source.rs) | MISSING_IN_VERUS | ❌ |
| `sleep` [sleep_source.rs](sleep_source.rs) | MISSING_IN_VERUS | ❌ |
| `take_earliest_ready` [take_earliest_ready_source.rs](take_earliest_ready_source.rs) | MISSING_IN_VERUS | ❌ |
| `take_mutex_guard` [take_mutex_guard_source.rs](take_mutex_guard_source.rs) | MISSING_IN_VERUS | ❌ |
| `take_running` [take_running_source.rs](take_running_source.rs) | MISSING_IN_VERUS | ❌ |
| `terminate` [terminate_source.rs](terminate_source.rs) | MISSING_IN_VERUS | ❌ |
| `try_add_thread` [try_add_thread_source.rs](try_add_thread_source.rs) | MISSING_IN_VERUS | ❌ |
| `try_borrow` [try_borrow_source.rs](try_borrow_source.rs) | MISSING_IN_VERUS | ❌ |
| `try_borrow_mut` [try_borrow_mut_source.rs](try_borrow_mut_source.rs) | MISSING_IN_VERUS | ❌ |
| `try_join_thread` [try_join_thread_source.rs](try_join_thread_source.rs) | MISSING_IN_VERUS | ❌ |
| `try_wakeup` [try_wakeup_source.rs](try_wakeup_source.rs) | MISSING_IN_VERUS | ❌ |
| `vmcopy_from_user` [vmcopy_from_user_source.rs](vmcopy_from_user_source.rs) | MISSING_IN_VERUS | ❌ |
| `vmcopy_to_user` [vmcopy_to_user_source.rs](vmcopy_to_user_source.rs) | MISSING_IN_VERUS | ❌ |
| `wakeup` [wakeup_source.rs](wakeup_source.rs) | MISSING_IN_VERUS | ❌ |
| `write_pmio` [write_pmio_source.rs](write_pmio_source.rs) | MISSING_IN_VERUS | ❌ |

## Inconsistent Structs

- `ProcessManager` [struct_ProcessManager_source.rs](struct_ProcessManager_source.rs): MISSING_IN_VERUS
- `ProcessManagerInner` [struct_ProcessManagerInner_source.rs](struct_ProcessManagerInner_source.rs): MISSING_IN_VERUS
