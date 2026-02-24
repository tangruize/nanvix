# Exec Diff: handler

**Source:** `/home/ubuntu/nanvix/src/kernel/src/kcall/handler.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/kcall/handler.rs`

| Function | Status | Files |
|----------|--------|-------|
| `kcall_handler` | MISMATCH | kcall_handler_source.rs, kcall_handler_verus.rs, kcall_handler.diff |
| `classify_and_check_invalid` | EXTRA_IN_VERUS | classify_and_check_invalid_verus.rs (EXTRA) |
| `dispatch_to_subsystem` | EXTRA_IN_VERUS | dispatch_to_subsystem_verus.rs (EXTRA) |
| `drain_remaining_zombies` | EXTRA_IN_VERUS | drain_remaining_zombies_verus.rs (EXTRA) |
| `event_init` | EXTRA_IN_VERUS | event_init_verus.rs (EXTRA) |
| `handle_harvest_phase` | EXTRA_IN_VERUS | handle_harvest_phase_verus.rs (EXTRA) |
| `handle_kcall_phase` | EXTRA_IN_VERUS | handle_kcall_phase_verus.rs (EXTRA) |
| `harvest_zombies` | EXTRA_IN_VERUS | harvest_zombies_verus.rs (EXTRA) |
| `is_initd_terminated` | EXTRA_IN_VERUS | is_initd_terminated_verus.rs (EXTRA) |
| `kcall_handler_init` | EXTRA_IN_VERUS | kcall_handler_init_verus.rs (EXTRA) |
| `kcall_handler_lifecycle_step` | EXTRA_IN_VERUS | kcall_handler_lifecycle_step_verus.rs (EXTRA) |
| `kcall_handler_loop` | EXTRA_IN_VERUS | kcall_handler_loop_verus.rs (EXTRA) |
| `make_invalid_syscall_error` | EXTRA_IN_VERUS | make_invalid_syscall_error_verus.rs (EXTRA) |
| `new_work_state` | EXTRA_IN_VERUS | new_work_state_verus.rs (EXTRA) |
| `notify_termination` | EXTRA_IN_VERUS | notify_termination_verus.rs (EXTRA) |
| `poll_messages_gated` | EXTRA_IN_VERUS | poll_messages_gated_verus.rs (EXTRA) |
| `poll_messages_raw` | EXTRA_IN_VERUS | poll_messages_raw_verus.rs (EXTRA) |
| `poll_scoreboard_full` | EXTRA_IN_VERUS | poll_scoreboard_full_verus.rs (EXTRA) |
| `run_full_iteration` | EXTRA_IN_VERUS | run_full_iteration_verus.rs (EXTRA) |
| `run_iteration` | EXTRA_IN_VERUS | run_iteration_verus.rs (EXTRA) |
| `should_yield` | EXTRA_IN_VERUS | should_yield_verus.rs (EXTRA) |
| `signal_handled` | EXTRA_IN_VERUS | signal_handled_verus.rs (EXTRA) |
| `yield_cpu` | EXTRA_IN_VERUS | yield_cpu_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `HandlerKcallPhaseResult` | EXTRA_IN_VERUS | struct_HandlerKcallPhaseResult_verus.rs (EXTRA) |
| `HandlerKcallResult` | EXTRA_IN_VERUS | struct_HandlerKcallResult_verus.rs (EXTRA) |
| `HandlerWorkState` | EXTRA_IN_VERUS | struct_HandlerWorkState_verus.rs (EXTRA) |
| `IterationResult` | EXTRA_IN_VERUS | struct_IterationResult_verus.rs (EXTRA) |
| `LifecycleStepResult` | EXTRA_IN_VERUS | struct_LifecycleStepResult_verus.rs (EXTRA) |
| `LoopResult` | EXTRA_IN_VERUS | struct_LoopResult_verus.rs (EXTRA) |
| `ScoreBoardPollResult` | EXTRA_IN_VERUS | struct_ScoreBoardPollResult_verus.rs (EXTRA) |
| `ZombieHarvestResult` | EXTRA_IN_VERUS | struct_ZombieHarvestResult_verus.rs (EXTRA) |
