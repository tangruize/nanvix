# Exec Consistency Fix: handler

## Summary
- Mismatches fixed: 0
- Missing functions added: 1
- Documented equivalences: 23 (22 extra functions + 8 extra structs justified)

## Context

The original `kcall_handler` [kcall_handler.diff](kcall_handler.diff) | [kcall_handler_source.rs](kcall_handler_source.rs) | [kcall_handler_verus.rs](kcall_handler_verus.rs) is a monolithic 154-line function that uses OS types
(`Hal`, `VirtMemoryManager`, `ProcessManager`, `ExitStatus`), `unsafe` blocks,
`panic!`, `unreachable!`, and `cfg_if!` macros. These constructs cannot be compiled
by Verus. The Verus version uses a **shadow model** approach (documented in the
module header) that decomposes the original into smaller verified functions with
abstracted types. This is the standard verification methodology used across all
verified modules in the Nanvix project.

## Changes

| Function | Action | Justification |
|----------|--------|---------------|
| `kcall_handler` [kcall_handler.diff](kcall_handler.diff) | [kcall_handler_source.rs](kcall_handler_source.rs) | [kcall_handler_verus.rs](kcall_handler_verus.rs) | ADDED as `external_body` | Original parameters (`&mut Hal`, `&mut VirtMemoryManager`, `&mut ProcessManager`) and return type (`ExitStatus`) cannot be compiled by Verus. Added as external_body entry point with documentation of equivalence to `kcall_handler_loop` [kcall_handler_loop_verus.rs](kcall_handler_loop_verus.rs). |
| `signal_handled` [signal_handled_verus.rs](signal_handled_verus.rs) | KEPT (documented) | External body modeling `scoreboard.handled(ret)` (T1 boundary). |
| `dispatch_to_subsystem` [dispatch_to_subsystem_verus.rs](dispatch_to_subsystem_verus.rs) | KEPT (documented) | External body modeling subsystem dispatch (T2 boundary). |
| `poll_messages_raw` [poll_messages_raw_verus.rs](poll_messages_raw_verus.rs) | KEPT (documented) | External body modeling IKC message polling (T4 boundary). |
| `harvest_zombies` [harvest_zombies_verus.rs](harvest_zombies_verus.rs) | KEPT (documented) | External body modeling `pm.harvest_zombies(mm)` (T2 boundary). |
| `notify_termination` [notify_termination_verus.rs](notify_termination_verus.rs) | KEPT (documented) | External body modeling `EventManager::notify_process_termination` (T2 boundary). |
| `yield_cpu` [yield_cpu_verus.rs](yield_cpu_verus.rs) | KEPT (documented) | External body modeling `ProcessManager::giveup()` (T3 boundary). |
| `event_init` [event_init_verus.rs](event_init_verus.rs) | KEPT (documented) | External body modeling `event::init(hal)` (T5 boundary). |
| `drain_remaining_zombies` [drain_remaining_zombies_verus.rs](drain_remaining_zombies_verus.rs) | KEPT (documented) | External body modeling post-loop `while let` zombie drain (T2 boundary). |
| `poll_scoreboard_full` [poll_scoreboard_full_verus.rs](poll_scoreboard_full_verus.rs) | KEPT (documented) | External body modeling `ScoreBoard::get_mut()` + `handle()` (T1 boundary). |
| `classify_and_check_invalid` [classify_and_check_invalid_verus.rs](classify_and_check_invalid_verus.rs) | KEPT (documented) | Verified function modeling the `match KcallNumber::from(...)` dispatch classification. |
| `new_work_state` [new_work_state_verus.rs](new_work_state_verus.rs) | KEPT (documented) | Verified helper creating fresh iteration state (all flags false). |
| `should_yield` [should_yield_verus.rs](should_yield_verus.rs) | KEPT (documented) | Verified helper implementing yield decision (`!kcall_handled && !message_received && !harvested_process`). |
| `is_initd_terminated` [is_initd_terminated_verus.rs](is_initd_terminated_verus.rs) | KEPT (documented) | Verified helper checking INITD termination condition (`pid == 1`). |
| `make_invalid_syscall_error` [make_invalid_syscall_error_verus.rs](make_invalid_syscall_error_verus.rs) | KEPT (documented) | Verified helper constructing InvalidSysCall error (code 88). |
| `handle_kcall_phase` [handle_kcall_phase_verus.rs](handle_kcall_phase_verus.rs) | KEPT (documented) | Verified function modeling kcall dispatch phase of one iteration. |
| `handle_harvest_phase` [handle_harvest_phase_verus.rs](handle_harvest_phase_verus.rs) | KEPT (documented) | Verified function modeling zombie harvest phase of one iteration. |
| `poll_messages_gated` [poll_messages_gated_verus.rs](poll_messages_gated_verus.rs) | KEPT (documented) | Verified function modeling `cfg_if!(feature = "stdio")` message gate. |
| `run_iteration` [run_iteration_verus.rs](run_iteration_verus.rs) | KEPT (documented) | Verified function composing one iteration (phases 1-3) without yield. |
| `run_full_iteration` [run_full_iteration_verus.rs](run_full_iteration_verus.rs) | KEPT (documented) | Verified function composing full iteration including yield decision. |
| `kcall_handler_init` [kcall_handler_init_verus.rs](kcall_handler_init_verus.rs) | KEPT (documented) | Verified function modeling initialization + loop invariant base case. |
| `kcall_handler_lifecycle_step` [kcall_handler_lifecycle_step_verus.rs](kcall_handler_lifecycle_step_verus.rs) | KEPT (documented) | Verified function modeling one lifecycle step with invariant induction. |
| `kcall_handler_loop` [kcall_handler_loop_verus.rs](kcall_handler_loop_verus.rs) | KEPT (documented) | Verified function modeling complete handler lifecycle (init → loop → drain). |

### Extra Structs (all justified)

| Struct | Justification |
|--------|---------------|
| `HandlerKcallResult` [struct_HandlerKcallResult_verus.rs](struct_HandlerKcallResult_verus.rs) | Models `KcallResult` for dispatch output (original type unavailable). |
| `HandlerKcallPhaseResult` [struct_HandlerKcallPhaseResult_verus.rs](struct_HandlerKcallPhaseResult_verus.rs) | Bundles kcall phase output (handled flag + invalid flag). |
| `HandlerWorkState` [struct_HandlerWorkState_verus.rs](struct_HandlerWorkState_verus.rs) | Models per-iteration work flags (`kcall_handled`, `message_received`, `harvested_process`). |
| `ZombieHarvestResult` [struct_ZombieHarvestResult_verus.rs](struct_ZombieHarvestResult_verus.rs) | Models `pm.harvest_zombies(mm)` output with exec-to-spec bridge fields. |
| `ScoreBoardPollResult` [struct_ScoreBoardPollResult_verus.rs](struct_ScoreBoardPollResult_verus.rs) | Models `ScoreBoard::get_mut()` + `handle()` output. |
| `IterationResult` [struct_IterationResult_verus.rs](struct_IterationResult_verus.rs) | Bundles full iteration output for lifecycle composition. |
| `LifecycleStepResult` [struct_LifecycleStepResult_verus.rs](struct_LifecycleStepResult_verus.rs) | Bundles lifecycle step output (terminated, exit_status, termination_pid). |
| `LoopResult` [struct_LoopResult_verus.rs](struct_LoopResult_verus.rs) | Bundles final loop output for top-level consumption. |

## API Mapping (Original → Shadow Model)

| Original API | Shadow Model | Type |
|-------------|-------------|------|
| `kcall_handler()` | `kcall_handler()` (external_body) + `kcall_handler_loop()` (verified) | Entry point |
| `event::init(hal)` | `event_init()` | External body (T5) |
| `ScoreBoard::get_mut()` + `handle()` | `poll_scoreboard_full()` | External body (T1) |
| `scoreboard.handled(ret)` | `signal_handled()` | External body (T1) |
| `match KcallNumber::from(...)` | `classify_and_check_invalid()` + `dispatch_to_subsystem()` | Verified + External |
| `pm.harvest_zombies(mm)` | `harvest_zombies()` | External body (T2) |
| `EventManager::notify_process_termination()` | `notify_termination()` | External body (T2) |
| `ProcessManager::giveup()` | `yield_cpu()` | External body (T3) |
| IKC message polling (`cfg_if!`) | `poll_messages_gated()` + `poll_messages_raw()` | Verified + External |
| Post-loop `while let` drain | `drain_remaining_zombies()` | External body (T2) |

## Verification: PASS
- 49 verified, 0 errors
- No `assume`, `admit`, or unjustified `external_body` added
- All `external_body` functions correspond to OS/HAL dependency boundaries (T1-T5)
