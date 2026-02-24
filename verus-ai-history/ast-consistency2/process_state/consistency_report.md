# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/process/state/mod.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/process/state/mod.rs`

## Summary

- Functions matched: 0/28
- Functions mismatched: 0
- Missing in Verus: 28
- Extra in Verus: 0
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `add_event` [add_event_source.rs](add_event_source.rs) | MISSING_IN_VERUS | 213-215 |  |
| `add_mmio` [add_mmio_source.rs](add_mmio_source.rs) | MISSING_IN_VERUS | 229-231 |  |
| `add_pmio` [add_pmio.diff](add_pmio.diff) [add_pmio_source.rs](add_pmio_source.rs) [add_pmio_verus.rs](add_pmio_verus.rs) | MISSING_IN_VERUS | 237-239 |  |
| `clear_capability` [clear_capability.diff](clear_capability.diff) [clear_capability_source.rs](clear_capability_source.rs) [clear_capability_verus.rs](clear_capability_verus.rs) | MISSING_IN_VERUS | 179-181 |  |
| `copy_from_user_unaligned` [copy_from_user_unaligned_source.rs](copy_from_user_unaligned_source.rs) | MISSING_IN_VERUS | 195-202 |  |
| `copy_to_user_unaligned` [copy_to_user_unaligned_source.rs](copy_to_user_unaligned_source.rs) | MISSING_IN_VERUS | 204-211 |  |
| `fmt` [fmt_source.rs](fmt_source.rs) | MISSING_IN_VERUS | 412-414 |  |
| `get_cond` [get_cond.diff](get_cond.diff) [get_cond_source.rs](get_cond_source.rs) [get_cond_verus.rs](get_cond_verus.rs) | MISSING_IN_VERUS | 354-367 |  |
| `get_mutex` [get_mutex.diff](get_mutex.diff) [get_mutex_source.rs](get_mutex_source.rs) [get_mutex_verus.rs](get_mutex_verus.rs) | MISSING_IN_VERUS | 295-308 |  |
| `get_pmio` [get_pmio_source.rs](get_pmio_source.rs) | MISSING_IN_VERUS | 253-263 |  |
| `get_pmio_mut` [get_pmio_mut_source.rs](get_pmio_mut_source.rs) | MISSING_IN_VERUS | 398-408 |  |
| `has_capability` [has_capability.diff](has_capability.diff) [has_capability_source.rs](has_capability_source.rs) [has_capability_verus.rs](has_capability_verus.rs) | MISSING_IN_VERUS | 183-185 |  |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISSING_IN_VERUS | 157-169 |  |
| `pid` [pid.diff](pid.diff) [pid_source.rs](pid_source.rs) [pid_verus.rs](pid_verus.rs) | MISSING_IN_VERUS | 171-173 |  |
| `post_message` [post_message_source.rs](post_message_source.rs) | MISSING_IN_VERUS | 221-223 |  |
| `put_cond` [put_cond.diff](put_cond.diff) [put_cond_source.rs](put_cond_source.rs) [put_cond_verus.rs](put_cond_verus.rs) | MISSING_IN_VERUS | 382-396 |  |
| `put_mutex` [put_mutex.diff](put_mutex.diff) [put_mutex_source.rs](put_mutex_source.rs) [put_mutex_verus.rs](put_mutex_verus.rs) | MISSING_IN_VERUS | 323-337 |  |
| `read_pmio` [read_pmio_source.rs](read_pmio_source.rs) | MISSING_IN_VERUS | 265-268 |  |
| `receive_message` [receive_message_source.rs](receive_message_source.rs) | MISSING_IN_VERUS | 225-227 |  |
| `remove_event` [remove_event_source.rs](remove_event_source.rs) | MISSING_IN_VERUS | 217-219 |  |
| `remove_mmio` [remove_mmio_source.rs](remove_mmio_source.rs) | MISSING_IN_VERUS | 233-235 |  |
| `remove_pmio` [remove_pmio.diff](remove_pmio.diff) [remove_pmio_source.rs](remove_pmio_source.rs) [remove_pmio_verus.rs](remove_pmio_verus.rs) | MISSING_IN_VERUS | 241-251 |  |
| `set_capability` [set_capability.diff](set_capability.diff) [set_capability_source.rs](set_capability_source.rs) [set_capability_verus.rs](set_capability_verus.rs) | MISSING_IN_VERUS | 175-177 |  |
| `state` [state_source.rs](state_source.rs) | MISSING_IN_VERUS | 115-123 |  |
| `state_mut` [state_mut_source.rs](state_mut_source.rs) | MISSING_IN_VERUS | 95-103 |  |
| `vmem` [vmem_source.rs](vmem_source.rs) | MISSING_IN_VERUS | 187-189 |  |
| `vmem_mut` [vmem_mut_source.rs](vmem_mut_source.rs) | MISSING_IN_VERUS | 191-193 |  |
| `write_pmio` [write_pmio_source.rs](write_pmio_source.rs) | MISSING_IN_VERUS | 270-278 |  |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `add_event` [add_event_source.rs](add_event_source.rs) | MISSING_IN_VERUS | ❌ |
| `add_mmio` [add_mmio_source.rs](add_mmio_source.rs) | MISSING_IN_VERUS | ❌ |
| `add_pmio` [add_pmio.diff](add_pmio.diff) [add_pmio_source.rs](add_pmio_source.rs) [add_pmio_verus.rs](add_pmio_verus.rs) | MISSING_IN_VERUS | ❌ |
| `clear_capability` [clear_capability.diff](clear_capability.diff) [clear_capability_source.rs](clear_capability_source.rs) [clear_capability_verus.rs](clear_capability_verus.rs) | MISSING_IN_VERUS | ❌ |
| `copy_from_user_unaligned` [copy_from_user_unaligned_source.rs](copy_from_user_unaligned_source.rs) | MISSING_IN_VERUS | ❌ |
| `copy_to_user_unaligned` [copy_to_user_unaligned_source.rs](copy_to_user_unaligned_source.rs) | MISSING_IN_VERUS | ❌ |
| `fmt` [fmt_source.rs](fmt_source.rs) | MISSING_IN_VERUS | ❌ |
| `get_cond` [get_cond.diff](get_cond.diff) [get_cond_source.rs](get_cond_source.rs) [get_cond_verus.rs](get_cond_verus.rs) | MISSING_IN_VERUS | ❌ |
| `get_mutex` [get_mutex.diff](get_mutex.diff) [get_mutex_source.rs](get_mutex_source.rs) [get_mutex_verus.rs](get_mutex_verus.rs) | MISSING_IN_VERUS | ❌ |
| `get_pmio` [get_pmio_source.rs](get_pmio_source.rs) | MISSING_IN_VERUS | ❌ |
| `get_pmio_mut` [get_pmio_mut_source.rs](get_pmio_mut_source.rs) | MISSING_IN_VERUS | ❌ |
| `has_capability` [has_capability.diff](has_capability.diff) [has_capability_source.rs](has_capability_source.rs) [has_capability_verus.rs](has_capability_verus.rs) | MISSING_IN_VERUS | ❌ |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISSING_IN_VERUS | ❌ |
| `pid` [pid.diff](pid.diff) [pid_source.rs](pid_source.rs) [pid_verus.rs](pid_verus.rs) | MISSING_IN_VERUS | ❌ |
| `post_message` [post_message_source.rs](post_message_source.rs) | MISSING_IN_VERUS | ❌ |
| `put_cond` [put_cond.diff](put_cond.diff) [put_cond_source.rs](put_cond_source.rs) [put_cond_verus.rs](put_cond_verus.rs) | MISSING_IN_VERUS | ❌ |
| `put_mutex` [put_mutex.diff](put_mutex.diff) [put_mutex_source.rs](put_mutex_source.rs) [put_mutex_verus.rs](put_mutex_verus.rs) | MISSING_IN_VERUS | ❌ |
| `read_pmio` [read_pmio_source.rs](read_pmio_source.rs) | MISSING_IN_VERUS | ❌ |
| `receive_message` [receive_message_source.rs](receive_message_source.rs) | MISSING_IN_VERUS | ❌ |
| `remove_event` [remove_event_source.rs](remove_event_source.rs) | MISSING_IN_VERUS | ❌ |
| `remove_mmio` [remove_mmio_source.rs](remove_mmio_source.rs) | MISSING_IN_VERUS | ❌ |
| `remove_pmio` [remove_pmio.diff](remove_pmio.diff) [remove_pmio_source.rs](remove_pmio_source.rs) [remove_pmio_verus.rs](remove_pmio_verus.rs) | MISSING_IN_VERUS | ❌ |
| `set_capability` [set_capability.diff](set_capability.diff) [set_capability_source.rs](set_capability_source.rs) [set_capability_verus.rs](set_capability_verus.rs) | MISSING_IN_VERUS | ❌ |
| `state` [state_source.rs](state_source.rs) | MISSING_IN_VERUS | ❌ |
| `state_mut` [state_mut_source.rs](state_mut_source.rs) | MISSING_IN_VERUS | ❌ |
| `vmem` [vmem_source.rs](vmem_source.rs) | MISSING_IN_VERUS | ❌ |
| `vmem_mut` [vmem_mut_source.rs](vmem_mut_source.rs) | MISSING_IN_VERUS | ❌ |
| `write_pmio` [write_pmio_source.rs](write_pmio_source.rs) | MISSING_IN_VERUS | ❌ |

## Inconsistent Structs

- `ProcessState` [struct_ProcessState.diff](struct_ProcessState.diff) [struct_ProcessState_source.rs](struct_ProcessState_source.rs) [struct_ProcessState_verus.rs](struct_ProcessState_verus.rs): MISSING_IN_VERUS
