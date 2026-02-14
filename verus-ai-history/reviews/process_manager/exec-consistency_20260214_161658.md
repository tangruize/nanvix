# Review: process_manager Exec Consistency (claude-opus-4.6)

## Grade: A-

## Verification Status
- **152 verified, 0 errors** — module `kernel::pm::process::manager`
- No `assume`, `admit`, or unjustified `external_body` found
- Verification command: `./verus-ai/scripts/verify.sh kernel::pm::process::manager` (note: short name `process_manager` resolves to wrong submodule path post-merge; must use full path)

## Issues Found

### Critical
- None.

### Major
- None.

### Minor

1. **Verify script module resolution mismatch.**
   The consistency report states `./verus-ai/scripts/verify.sh process_manager` as the verification command, but after the `include!()` restructuring, the module name `process_manager` resolves to `kernel::pm::process::manager::process_manager` (which no longer exists as a separate Verus module). The correct invocation is `./verus-ai/scripts/verify.sh kernel::pm::process::manager`. The report's claimed "152 verified" result is valid (verified independently), but the documented command would fail.

2. **`terminate` modeling omits the ready→terminate→resume→ready path detail.**
   The original `terminate()` (mod.rs:1054–1058) for a ready process with surviving threads calls `process.terminate()` which returns `Ok(interrupted_process)`, then `interrupted_process.resume()`, then pushes back to ready. The verified model (`terminate_ready_stays_ready`) correctly models this as a no-op on queue state, but the intermediate `ready→interrupted→ready` transition within the same function is noted only in comments, not as separate verified steps. This is acceptable as the net effect is zero queue change, but documenting the transient state more explicitly would improve confidence.

3. **`SleepError` enum not modeled.**
   The original defines `SleepError` (mod.rs:107–110) with two variants: `Interrupted(InterruptReason)` and `Generic(Error)`. The verified model uses boolean flags and ghost parameters instead. This is a reasonable abstraction for queue-level verification, but the enum itself is not represented in the verified code. Future work on error-path verification would need to add this.

4. **`exit_thread` three-way branch: `exit_thread_to_suspended` modeling.**
   The original `exit_thread` (mod.rs:1008) has a path where the process goes to `suspended` (only sleeping threads remain). The verified `exit_thread_to_suspended` correctly models running→suspended + ready→running, but the original code path goes through `running_process.exit_thread()` returning `Err(Ok(...))` — a nested Result. The model abstracts this as a `branch` parameter. This is sound but worth noting as a T3 trust boundary with three outcomes modeled separately.

### Observations (Non-Issues)

1. **`include!()` restructuring is well-justified.**
   The root cause (AST diff tool comparing only `mod.rs` to `mod.rs`, missing submodule contents) is correctly identified. The `include!()` approach is the right fix: it makes all 53 functions visible in the `mod` scope without changing semantics. The conversion of `//!` inner doc comments to `//` regular comments is required by Rust's `include!()` semantics and is correctly done.

2. **Trust boundary documentation is thorough.**
   The spec file (process_manager.spec.rs) contains 188 lines of trust boundary documentation covering T1–T4 plus error path verification, queue ordering abstraction, and scheduler composition. This is exemplary for a verification model.

3. **Structural correspondence is strong.**
   Every original `ProcessManagerInner` function has either:
   - A direct verified counterpart (e.g., `create_process`, `schedule`, `sleep_running`, `exit_running`, `wakeup_to_ready`, `terminate_ready`, `terminate_suspended`, `harvest_zombie`)
   - A dispatch function decomposing branch logic (e.g., `sleep_dispatch`, `exit_dispatch`, `exit_thread_dispatch`, `wakeup_dispatch`, `create_thread_dispatch`)
   - A verified no-op stub proving wf() preservation (e.g., `forge_user_context`, `take_running`, `get_running`, `find_process`, `get_mutex`, `handle_fpu_exception`)
   - An error-path no-op (e.g., `inner_create_thread_error`, `post_message_not_found`, `wakeup_not_found`)

4. **Outer ProcessManager wrapper coverage is complete.**
   All 34 public methods on `ProcessManager` (mod.rs:1531–1982) have corresponding verified stubs in `ProcessManagerUnsafeState`, including delegation functions, error paths, and the RefCell borrow boundary (T2).

5. **ProcessManagerUnsafeState models the unsafe.rs layer correctly.**
   The `switch()` function correctly models the stale-atomic comparison pattern (next_pid vs. old CURRENT_PID), quantum reset on PID change, and the hard/soft switch distinction. The `ghost_diverged` flag for machine-checked divergence (T10) is a creative solution to verify that exit/exit_thread are terminal operations.

6. **PidSet concrete implementation is well-designed.**
   The `PidSet` type replaces `Ghost<Set<int>>` with a `Vec<u64>` backed by `seq_to_set`, with 8 supporting lemmas proving the correspondence. The `no_dups()` invariant ensures cardinality preservation. The `absorb()` method for `resume_all_interrupted` is a clean abstraction.

## Function Coverage Checklist

| Original Function | Verified Counterpart | Status |
|---|---|---|
| `ProcessManagerInner::new` | `ProcessManagerInner::new` | ✅ Direct |
| `forge_user_context` | `forge_user_context` | ✅ No-op stub |
| `create_thread` | `create_thread_dispatch`, `inner_create_thread` | ✅ Dispatch |
| `try_add_thread` | `inner_try_add_thread` | ✅ Dispatch |
| `set_thread_data_area` | `set_thread_data_area` | ✅ No-op stub |
| `get_thread_data_area` | `get_thread_data_area` | ✅ No-op stub |
| `create_process` | `create_process` | ✅ Full proof |
| `schedule` | `schedule`, `full_schedule` | ✅ Full proof |
| `check_alarm` | `alarm_interrupt`, `check_alarm_wrapper` | ✅ Per-element + wrapper |
| `sleep` | `sleep_running`, `sleep_thread_running`, `sleep_dispatch` | ✅ Full proof |
| `wakeup` | `wakeup_to_ready`, `wakeup_running_noop`, `wakeup_ready_noop`, `wakeup_not_found`, `wakeup_suspended_failed_noop`, `wakeup_dispatch` | ✅ Full coverage |
| `try_wakeup` | (covered by wakeup dispatch) | ✅ Subsumed |
| `exit` | `exit_running`, `exit_thread_running`, `exit_dispatch` | ✅ Full proof |
| `exit_thread` | `exit_thread_running`, `exit_thread_to_suspended`, `exit_thread_to_zombie`, `exit_thread_dispatch` | ✅ Full proof |
| `terminate` | `terminate_ready`, `terminate_ready_stays_ready`, `terminate_suspended` | ✅ Full proof |
| `capctl` | `capctl`, `capctl_error_noop` | ✅ No-op stub |
| `handle_fpu_exception` | `handle_fpu_exception` | ✅ No-op stub |
| `interrupt_reason` | `take_interrupt_reason` | ✅ No-op stub |
| `harvest_zombies` | `harvest_zombie`, `harvest_zombies_wrapper` | ✅ Per-element + wrapper |
| `try_join_thread` | `try_join_thread` | ✅ No-op stub |
| `get_mutex` | `get_mutex` | ✅ No-op stub |
| `get_cond` | `get_cond` | ✅ No-op stub |
| `put_cond` | `put_cond` | ✅ No-op stub |
| `put_mutex_guard` | `put_mutex_guard` | ✅ No-op stub |
| `take_mutex_guard` | `take_mutex_guard` | ✅ No-op stub |
| `take_earliest_ready` | `take_earliest_ready` | ✅ No-op stub |
| `take_running` | `take_running` | ✅ No-op stub |
| `get_running` | `get_running` | ✅ No-op stub |
| `get_running_mut` | `get_running_mut` | ✅ No-op stub |
| `find_process` | `find_process` | ✅ No-op stub |
| `find_process_mut` | `find_process_mut` | ✅ No-op stub |
| `find_process_by_tid` | `find_process_by_tid` | ✅ No-op stub |
| `find_thread_mut` | `find_thread_mut` | ✅ No-op stub |
| `ProcessManager::get_pid` | `outer_get_pid` (in spec) | ✅ Outer stub |
| `ProcessManager::get_tid` | `outer_get_tid` (in spec) | ✅ Outer stub |
| `ProcessManager::create_process` | via inner `create_process` | ✅ Delegation |
| `ProcessManager::create_thread` | via `create_thread_dispatch` | ✅ Delegation |
| `ProcessManager::terminate` | via `terminate_ready`/`terminate_suspended` | ✅ Delegation |
| `ProcessManager::harvest_zombies` | `harvest_zombies_wrapper` | ✅ Delegation |
| `ProcessManager::post_message` | `post_message`, `post_message_not_found` | ✅ Full proof |
| `ProcessManager::number_buffered_messages` | `get_buffered_message_count` | ✅ Direct |
| `ProcessManager::try_borrow` | T2 boundary | ✅ Documented |
| `ProcessManager::try_borrow_mut` | T2 boundary | ✅ Documented |
| All remaining ProcessManager methods | Outer delegation stubs | ✅ Documented in spec |

## Summary

The exec consistency fix correctly addresses the root cause: the AST diff tool's inability to see submodule contents. The `include!()` approach is clean and preserves all existing verification. All 53 original functions are accounted for with verified counterparts, dispatch functions, or justified no-op stubs. The verification model is well-documented with clear trust boundaries (T1–T15) and the shadow-model approach is sound. Key state transitions (create, schedule, sleep, exit, wakeup, terminate, harvest) have full proofs with rich postconditions. The grade is A- rather than A due to the verify command documentation error and the minor modeling gaps noted above.
