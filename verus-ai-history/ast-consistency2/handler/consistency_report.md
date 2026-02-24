# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/kcall/handler.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/kcall/handler.rs`

## Summary

- Functions matched: 0/1
- Functions mismatched: 0
- Missing in Verus: 1
- Extra in Verus: 22
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `kcall_handler` [kcall_handler.diff](kcall_handler.diff) | [kcall_handler_source.rs](kcall_handler_source.rs) | [kcall_handler_verus.rs](kcall_handler_verus.rs) | MISSING_IN_VERUS | 47-200 |  |
| `classify_and_check_invalid` [classify_and_check_invalid_verus.rs](classify_and_check_invalid_verus.rs) | EXTRA_IN_VERUS |  | 455-470 |
| `dispatch_to_subsystem` [dispatch_to_subsystem_verus.rs](dispatch_to_subsystem_verus.rs) | EXTRA_IN_VERUS |  | 280-286 |
| `drain_remaining_zombies` [drain_remaining_zombies_verus.rs](drain_remaining_zombies_verus.rs) | EXTRA_IN_VERUS |  | 709-713 |
| `event_init` [event_init_verus.rs](event_init_verus.rs) | EXTRA_IN_VERUS |  | 390-396 |
| `handle_harvest_phase` [handle_harvest_phase_verus.rs](handle_harvest_phase_verus.rs) | EXTRA_IN_VERUS |  | 581-588 |
| `handle_kcall_phase` [handle_kcall_phase_verus.rs](handle_kcall_phase_verus.rs) | EXTRA_IN_VERUS |  | 539-565 |
| `harvest_zombies` [harvest_zombies_verus.rs](harvest_zombies_verus.rs) | EXTRA_IN_VERUS |  | 332-339 |
| `is_initd_terminated` [is_initd_terminated_verus.rs](is_initd_terminated_verus.rs) | EXTRA_IN_VERUS |  | 509-514 |
| `kcall_handler_init` [kcall_handler_init_verus.rs](kcall_handler_init_verus.rs) | EXTRA_IN_VERUS |  | 839-847 |
| `kcall_handler_lifecycle_step` [kcall_handler_lifecycle_step_verus.rs](kcall_handler_lifecycle_step_verus.rs) | EXTRA_IN_VERUS |  | 869-917 |
| `kcall_handler_loop` [kcall_handler_loop_verus.rs](kcall_handler_loop_verus.rs) | EXTRA_IN_VERUS |  | 974-1020 |
| `make_invalid_syscall_error` [make_invalid_syscall_error_verus.rs](make_invalid_syscall_error_verus.rs) | EXTRA_IN_VERUS |  | 521-530 |
| `new_work_state` [new_work_state_verus.rs](new_work_state_verus.rs) | EXTRA_IN_VERUS |  | 477-488 |
| `notify_termination` [notify_termination_verus.rs](notify_termination_verus.rs) | EXTRA_IN_VERUS |  | 352-355 |
| `poll_messages_gated` [poll_messages_gated_verus.rs](poll_messages_gated_verus.rs) | EXTRA_IN_VERUS |  | 437-446 |
| `poll_messages_raw` [poll_messages_raw_verus.rs](poll_messages_raw_verus.rs) | EXTRA_IN_VERUS |  | 309-312 |
| `poll_scoreboard_full` [poll_scoreboard_full_verus.rs](poll_scoreboard_full_verus.rs) | EXTRA_IN_VERUS |  | 787-799 |
| `run_full_iteration` [run_full_iteration_verus.rs](run_full_iteration_verus.rs) | EXTRA_IN_VERUS |  | 735-770 |
| `run_iteration` [run_iteration_verus.rs](run_iteration_verus.rs) | EXTRA_IN_VERUS |  | 606-671 |
| `should_yield` [should_yield_verus.rs](should_yield_verus.rs) | EXTRA_IN_VERUS |  | 496-501 |
| `signal_handled` [signal_handled_verus.rs](signal_handled_verus.rs) | EXTRA_IN_VERUS |  | 256-263 |
| `yield_cpu` [yield_cpu_verus.rs](yield_cpu_verus.rs) | EXTRA_IN_VERUS |  | 366-369 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `kcall_handler` [kcall_handler.diff](kcall_handler.diff) | [kcall_handler_source.rs](kcall_handler_source.rs) | [kcall_handler_verus.rs](kcall_handler_verus.rs) | MISSING_IN_VERUS | ❌ |
| `classify_and_check_invalid` [classify_and_check_invalid_verus.rs](classify_and_check_invalid_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `dispatch_to_subsystem` [dispatch_to_subsystem_verus.rs](dispatch_to_subsystem_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `drain_remaining_zombies` [drain_remaining_zombies_verus.rs](drain_remaining_zombies_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `event_init` [event_init_verus.rs](event_init_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `handle_harvest_phase` [handle_harvest_phase_verus.rs](handle_harvest_phase_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `handle_kcall_phase` [handle_kcall_phase_verus.rs](handle_kcall_phase_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `harvest_zombies` [harvest_zombies_verus.rs](harvest_zombies_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `is_initd_terminated` [is_initd_terminated_verus.rs](is_initd_terminated_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `kcall_handler_init` [kcall_handler_init_verus.rs](kcall_handler_init_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `kcall_handler_lifecycle_step` [kcall_handler_lifecycle_step_verus.rs](kcall_handler_lifecycle_step_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `kcall_handler_loop` [kcall_handler_loop_verus.rs](kcall_handler_loop_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `make_invalid_syscall_error` [make_invalid_syscall_error_verus.rs](make_invalid_syscall_error_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `new_work_state` [new_work_state_verus.rs](new_work_state_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `notify_termination` [notify_termination_verus.rs](notify_termination_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `poll_messages_gated` [poll_messages_gated_verus.rs](poll_messages_gated_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `poll_messages_raw` [poll_messages_raw_verus.rs](poll_messages_raw_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `poll_scoreboard_full` [poll_scoreboard_full_verus.rs](poll_scoreboard_full_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `run_full_iteration` [run_full_iteration_verus.rs](run_full_iteration_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `run_iteration` [run_iteration_verus.rs](run_iteration_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `should_yield` [should_yield_verus.rs](should_yield_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `signal_handled` [signal_handled_verus.rs](signal_handled_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `yield_cpu` [yield_cpu_verus.rs](yield_cpu_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `HandlerKcallPhaseResult` [struct_HandlerKcallPhaseResult_verus.rs](struct_HandlerKcallPhaseResult_verus.rs): EXTRA_IN_VERUS
- `HandlerKcallResult` [struct_HandlerKcallResult_verus.rs](struct_HandlerKcallResult_verus.rs): EXTRA_IN_VERUS
- `HandlerWorkState` [struct_HandlerWorkState_verus.rs](struct_HandlerWorkState_verus.rs): EXTRA_IN_VERUS
- `IterationResult` [struct_IterationResult_verus.rs](struct_IterationResult_verus.rs): EXTRA_IN_VERUS
- `LifecycleStepResult` [struct_LifecycleStepResult_verus.rs](struct_LifecycleStepResult_verus.rs): EXTRA_IN_VERUS
- `LoopResult` [struct_LoopResult_verus.rs](struct_LoopResult_verus.rs): EXTRA_IN_VERUS
- `ScoreBoardPollResult` [struct_ScoreBoardPollResult_verus.rs](struct_ScoreBoardPollResult_verus.rs): EXTRA_IN_VERUS
- `ZombieHarvestResult` [struct_ZombieHarvestResult_verus.rs](struct_ZombieHarvestResult_verus.rs): EXTRA_IN_VERUS
