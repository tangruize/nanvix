# Exec Consistency Fix: dispatcher

## Summary
- Mismatches fixed: 0
- Missing functions added: 0
- Documented equivalences: 2 (MISMATCH) + 36 (EXTRA_IN_VERUS) + 6 (EXTRA structs)

## Changes
| Function | Action | Justification |
|----------|--------|---------------|
| `do_kcall` [do_kcall.diff](do_kcall.diff) [do_kcall_source.rs](do_kcall_source.rs) [do_kcall_verus.rs](do_kcall_verus.rs) | Documented equivalence | Structural decomposition for verification; see §1 below. |
| `handle_sleep_error` [handle_sleep_error.diff](handle_sleep_error.diff) [handle_sleep_error_source.rs](handle_sleep_error_source.rs) [handle_sleep_error_verus.rs](handle_sleep_error_verus.rs) | Documented equivalence | Killed-path split for divergence modeling; see §2 below. |
| `classify_kcall_number` [classify_kcall_number_verus.rs](classify_kcall_number_verus.rs) | Kept (justified) | Verification helper: exec mirror of `spec_classify_kcall`; models `KcallNumber::from(u32)` match arms. |
| `convert_fallible` [convert_fallible_verus.rs](convert_fallible_verus.rs) | Kept (justified) | Verification helper: factors `Ok→Success / Err→Error` pattern for fallible calls. |
| `convert_sleepable` [convert_sleepable_verus.rs](convert_sleepable_verus.rs) | Kept (justified) | Verification helper: factors `Ok→Success / Err→handle_sleep_error` pattern for sleepable calls. |
| `diverge_after_exit` [diverge_after_exit_verus.rs](diverge_after_exit_verus.rs) | Kept (justified) | Trust boundary T4b: models `panic!()` divergence after exit. |
| `do_kcall_abi` [do_kcall_abi_verus.rs](do_kcall_abi_verus.rs) | Kept (justified) | Closes ABI gap (T5): matches original `extern "C" fn(u32,u32,u32,u32,u32)->i64` signature. |
| `do_kcall_context` [do_kcall_context_verus.rs](do_kcall_context_verus.rs) | Kept (justified) | Verification decomposition: separates pid/tid retrieval from dispatch logic. |
| `do_kcall_dispatch` [do_kcall_dispatch_verus.rs](do_kcall_dispatch_verus.rs) | Kept (justified) | Verification decomposition: verified match dispatch after pid/tid are available. |
| `do_kcall_encoded` [do_kcall_encoded_verus.rs](do_kcall_encoded_verus.rs) | Kept (justified) | Composes `do_kcall` + `encode_result`; provides result+encoding pair for callers. |
| `encode_result` [encode_result_verus.rs](encode_result_verus.rs) | Kept (justified) | Models `KcallResult::into::<i64>()` ABI encoding; verified identity on value field. |
| `error` [error_verus.rs](error_verus.rs) | Kept (justified) | `DispatchResult::error()` constructor modeling `KcallResult::Error`. |
| `event_resume` [event_resume_verus.rs](event_resume_verus.rs) | Kept (justified) | External body (T3): models `event::resume(arg0 as usize)`. |
| `generic` [generic_verus.rs](generic_verus.rs) | Kept (justified) | `SleepError::generic()` constructor for verified error creation. |
| `handle_sleep_error_killed` [handle_sleep_error_killed_verus.rs](handle_sleep_error_killed_verus.rs) | Kept (justified) | Verification decomposition: models divergent `Interrupted(Killed)` path separately. |
| `interrupted_killed` [interrupted_killed_verus.rs](interrupted_killed_verus.rs) | Kept (justified) | `SleepError::interrupted_killed()` constructor. |
| `interrupted_timed_out` [interrupted_timed_out_verus.rs](interrupted_timed_out_verus.rs) | Kept (justified) | `SleepError::interrupted_timed_out()` constructor. |
| `ipc_recv` [ipc_recv_verus.rs](ipc_recv_verus.rs) | Kept (justified) | External body (T3): models `ipc::recv(tid, pid, arg0 as usize)`. |
| `is_locally_handled` [is_locally_handled_verus.rs](is_locally_handled_verus.rs) | Kept (justified) | Verification helper: exec mirror of `spec_is_locally_handled`. |
| `is_sleepable` [is_sleepable_verus.rs](is_sleepable_verus.rs) | Kept (justified) | Verification helper: exec mirror of `spec_is_sleepable`. |
| `new` [new_verus.rs](new_verus.rs) | Kept (justified) | `DispatchArgs::new()` constructor with verified postconditions. |
| `ok` [ok_verus.rs](ok_verus.rs) | Kept (justified) | `DispatchResult::ok()` constructor modeling `KcallResult::ok()`. |
| `pm_exit` [pm_exit_verus.rs](pm_exit_verus.rs) | Kept (justified) | External body (T3): models `ProcessManager::exit(ExitStatus::from(arg0))`. |
| `pm_exit_interrupted` [pm_exit_interrupted_verus.rs](pm_exit_interrupted_verus.rs) | Kept (justified) | External body (T4a): models `ProcessManager::exit(ErrorCode::Interrupted)`. |
| `pm_exit_thread` [pm_exit_thread_verus.rs](pm_exit_thread_verus.rs) | Kept (justified) | External body (T3): models `ProcessManager::exit_thread(arg0.into())`. |
| `pm_get_pid` [pm_get_pid_verus.rs](pm_get_pid_verus.rs) | Kept (justified) | External body (T1): models `ProcessManager::get().get_pid()`. |
| `pm_get_tid` [pm_get_tid_verus.rs](pm_get_tid_verus.rs) | Kept (justified) | External body (T1): models `ProcessManager::get().get_tid()`. |
| `pm_giveup` [pm_giveup_verus.rs](pm_giveup_verus.rs) | Kept (justified) | External body (T3): models `ProcessManager::giveup()`. |
| `pm_join_thread` [pm_join_thread_verus.rs](pm_join_thread_verus.rs) | Kept (justified) | External body (T3): models `pm::join_thread(pid, arg0, arg1)`. |
| `pm_lock_mutex` [pm_lock_mutex_verus.rs](pm_lock_mutex_verus.rs) | Kept (justified) | External body (T3): models `pm::lock_mutex(pid, tid, arg0, arg1, arg2)`. |
| `pm_signal_cond` [pm_signal_cond_verus.rs](pm_signal_cond_verus.rs) | Kept (justified) | External body (T3): models `pm::signal_cond(pid, tid, arg0, arg1 != 0)`. |
| `pm_sleep` [pm_sleep_verus.rs](pm_sleep_verus.rs) | Kept (justified) | External body (T3): models `pm::sleep(arg0, arg1)`. |
| `pm_unlock_mutex` [pm_unlock_mutex_verus.rs](pm_unlock_mutex_verus.rs) | Kept (justified) | External body (T3): models `pm::unlock_mutex(pid, tid, arg0)`. |
| `pm_wait_cond` [pm_wait_cond_verus.rs](pm_wait_cond_verus.rs) | Kept (justified) | External body (T3): models `pm::wait_cond(pid, tid, arg0, arg1, arg2, arg3)`. |
| `remote_dispatch_verified` [remote_dispatch_verified_verus.rs](remote_dispatch_verified_verus.rs) | Kept (justified) | Verification decomposition: verified scoreboard routing logic (wildcard match arm). |
| `scoreboard_dispatch_call` [scoreboard_dispatch_call_verus.rs](scoreboard_dispatch_call_verus.rs) | Kept (justified) | External body (T2): models `scoreboard.dispatch(number, pid, tid, arg0, arg1, arg2, arg3)`. |
| `scoreboard_get_mut` [scoreboard_get_mut_verus.rs](scoreboard_get_mut_verus.rs) | Kept (justified) | External body (T2): models `ScoreBoard::get_mut()`. |
| `success` [success_verus.rs](success_verus.rs) | Kept (justified) | `DispatchResult::success()` constructor modeling `KcallResult::Success`. |
| `DispatchArgs` (struct) | Kept (justified) | Models the five u32 arguments to `do_kcall` for typed verification. |
| `DispatchResult` (struct) | Kept (justified) | Models `KcallResult` enum with `is_success`/`value` for spec-level reasoning. |
| `FallibleOutcome` (struct) | Kept (justified) | Models `Result<T, Error>` return type for non-sleeping subsystem calls. |
| `ScoreboardDispatchOutcome` (struct) | Kept (justified) | Models `Result<KcallResult, SleepError>` from `scoreboard.dispatch()`. |
| `SleepError` (struct) | Kept (justified) | Models original `pm::SleepError` enum as flat struct for Verus verification. |
| `SleepableOutcome` (struct) | Kept (justified) | Models `Result<T, SleepError>` return type for sleeping subsystem calls. |

## Verification: PASS

66 verified, 0 errors.

---

## §1 — `do_kcall` MISMATCH: Structural Decomposition (Semantically Equivalent)

### Original (lines 54–146)

```rust
#[unsafe(no_mangle)]
pub extern "C" fn do_kcall(number: u32, arg0: u32, arg1: u32, arg2: u32, arg3: u32) -> i64 {
    let pid = ProcessManager::get().get_pid()...;
    let tid = ProcessManager::get().get_tid()...;
    match KcallNumber::from(number) {
        KcallNumber::GetPid => KcallResult::Success(pid.into()),
        KcallNumber::GetTid => KcallResult::Success(tid.into()),
        KcallNumber::Exit => { ProcessManager::exit(...).unwrap_err(); ... }
        KcallNumber::JoinThread => match pm::join_thread(pid, arg0, arg1) { ... }
        // ... 10 more match arms ...
        _ => match ScoreBoard::get_mut() { ... scoreboard.dispatch(...) ... }
    }.into()
}
```

### Verus Decomposition

The monolithic function is split into four verified layers:

1. **`do_kcall_abi(number, arg0, arg1, arg2, arg3) -> i64`** — matches the
   original C ABI signature exactly. Constructs `DispatchArgs`, calls
   `do_kcall`, encodes the result via `encode_result`. This closes trust
   boundary T5.

2. **`do_kcall(args: DispatchArgs) -> DispatchResult`** — thin wrapper that
   delegates to `do_kcall_context`. Provides the named entry point for
   verification postconditions.

3. **`do_kcall_context(args) -> DispatchResult`** — retrieves pid/tid via
   `pm_get_pid()` / `pm_get_tid()` (external bodies modeling
   `ProcessManager::get().get_pid/tid()`). On success, delegates to
   `do_kcall_dispatch`.

4. **`do_kcall_dispatch(pid, tid, args) -> DispatchResult`** — the core match
   dispatch. Each branch maps to the original match arm:

   | Original Match Arm | Verus Dispatch |
   |---|---|
   | `KcallNumber::GetPid` → `Success(pid)` | `number==1` → `success(pid)` |
   | `KcallNumber::GetTid` → `Success(tid)` | `number==2` → `success(tid)` |
   | `KcallNumber::Exit` → `exit().unwrap_err()` | `number==3` → `pm_exit(arg0)` |
   | `KcallNumber::ExitThread` → `exit_thread().unwrap_err()` | `number==22` → `pm_exit_thread(arg0)` |
   | `KcallNumber::JoinThread` → sleepable | `number==23` → `convert_sleepable(pm_join_thread(...))` |
   | `KcallNumber::Recv` → sleepable | `number==9` → `convert_sleepable(ipc_recv(...))` |
   | `KcallNumber::Resume` → direct | `number==5` → `event_resume(arg0)` |
   | `KcallNumber::MutexLock` → sleepable | `number==24` → `convert_sleepable(pm_lock_mutex(...))` |
   | `KcallNumber::MutexUnlock` → fallible | `number==25` → `convert_fallible(pm_unlock_mutex(...))` |
   | `KcallNumber::CondWait` → sleepable | `number==27` → `convert_sleepable(pm_wait_cond(...))` |
   | `KcallNumber::CondSignal` → fallible | `number==26` → `convert_fallible(pm_signal_cond(...))` |
   | `KcallNumber::SchedulerYield` → fallible | `number==20` → `convert_fallible(pm_giveup())` |
   | `KcallNumber::Sleep` → sleepable | `number==29` → `convert_sleepable(pm_sleep(...))` |
   | `_` → scoreboard dispatch | else → `remote_dispatch_verified(...)` |

### Why This Is Equivalent

- The `KcallNumber` enum uses `#[repr(u32)]`; `KcallNumber::from(number)`
  performs the same u32 → variant mapping that the if-else chain does.
  `lemma_kcall_constants_consistency` asserts all 33 enum values match.
- The pid/tid retrieval sequence is identical: get_pid first, get_tid second,
  early-return on error.
- Each match arm calls the same subsystem function with the same arguments.
  The `as usize` casts are elided because on x86-32, `usize == u32` (documented
  in the external body comment, line 586–589).
- The `.into()` at the end of the original match converts `KcallResult` to
  `i64`; the Verus version models this via `encode_result()` which is verified
  to be the identity on the value field (`lemma_encode_result_is_value`).
- The factoring into `convert_sleepable` / `convert_fallible` captures the
  `Ok→Success / Err→handle_sleep_error` and `Ok→Success / Err→Error` patterns
  without changing behavior.

### Verus Limitation Requiring Decomposition

Verus cannot verify `extern "C"` functions, `unsafe` blocks, or
`KcallNumber::from()` (which is a foreign type conversion). The decomposition
isolates these into small external bodies and verifies the routing logic
between them.

---

## §2 — `handle_sleep_error` MISMATCH: Divergence Split (Semantically Equivalent)

### Original (lines 148–167)

```rust
fn handle_sleep_error(sleep_error: SleepError) -> KcallResult {
    match sleep_error {
        SleepError::Generic(generic_error) => {
            error!("failed to sleep: {:?}", generic_error);
            KcallResult::Error(generic_error.code.into())
        },
        SleepError::Interrupted(reason) => match reason {
            InterruptReason::Killed => {
                let error = ProcessManager::exit(ErrorCode::Interrupted.into()).unwrap_err();
                panic!("failed to exit() (error={:?})", error);
            },
            InterruptReason::TimedOut => {
                error!("failed to sleep: operation timed out");
                KcallResult::Error(ErrorCode::OperationTimedOut.into())
            },
        },
    }
}
```

### Verus Version (lines 496–517 + 539–545)

Two functions:
1. **`handle_sleep_error(sleep_error: SleepError) -> DispatchResult`** — handles
   non-divergent cases (Generic, TimedOut). The `InterruptedKilled` case is
   excluded by precondition `spec_sleep_error_returns(kind)`.
2. **`handle_sleep_error_killed() -> DispatchResult`** — handles the divergent
   `Killed` path. Calls `pm_exit_interrupted()` (T4a) then `diverge_after_exit()`
   (T4b). Postcondition is `ensures false` (never returns).

### Why This Is Equivalent

- **Generic path**: Both extract the error code and return an error result.
  The `error!()` logging call is a side effect with no control-flow impact.
- **TimedOut path**: Both return `ErrorCode::OperationTimedOut`. The Verus
  version hardcodes `116i32`, which matches `ETIMEDOUT = 116` defined in
  `src/libs/sysapi/src/errno.rs:209` and used by `ErrorCode::OperationTimedOut`
  in `src/libs/error/src/lib.rs:235`. The `error!()` log is omitted (side effect).
- **Killed path**: The original calls `ProcessManager::exit()` then `panic!()`.
  The Verus version splits this into `pm_exit_interrupted()` + `diverge_after_exit()`,
  which models the same sequence. The precondition on `handle_sleep_error` excludes
  `InterruptedKilled`, and callers route `Killed` to `handle_sleep_error_killed()`
  instead (see `convert_sleepable` and `remote_dispatch_verified`).

### Verus Limitation Requiring Split

Verus cannot model a function that conditionally returns or diverges in the same
body (there is no `!` return type support with conditional divergence). Splitting
into a returning function (`handle_sleep_error`, precondition excludes Killed) and
a diverging function (`handle_sleep_error_killed`, `ensures false`) is the
standard Verus pattern for modeling conditional divergence.

---

## §3 — EXTRA_IN_VERUS: Justification Summary

All 36 extra functions and 6 extra structs are verification artifacts:

- **6 structs** (`DispatchResult`, `DispatchArgs`, `SleepError`,
  `SleepableOutcome`, `FallibleOutcome`, `ScoreboardDispatchOutcome`):
  Model original Rust types (`KcallResult`, raw args, `pm::SleepError`,
  `Result<T, SleepError>`, `Result<T, Error>`, scoreboard dispatch result)
  as flat structs with View types for spec-level reasoning.

- **12 external body functions** (`pm_get_pid`, `pm_get_tid`, `pm_exit`,
  `pm_exit_thread`, `pm_join_thread`, `ipc_recv`, `event_resume`,
  `pm_lock_mutex`, `pm_unlock_mutex`, `pm_wait_cond`, `pm_signal_cond`,
  `pm_giveup`, `pm_sleep`, `scoreboard_get_mut`, `scoreboard_dispatch_call`,
  `pm_exit_interrupted`, `diverge_after_exit`): Model `unsafe` subsystem
  calls that Verus cannot verify directly. Each has postconditions matching
  the original function's return-type contract.

- **7 constructors** (`ok`, `success`, `error`, `new`, `generic`,
  `interrupted_killed`, `interrupted_timed_out`): Verified constructors
  with postconditions enabling downstream proofs.

- **5 classification/query functions** (`classify_kcall_number`,
  `is_locally_handled`, `is_sleepable`, `convert_sleepable`,
  `convert_fallible`): Factor verification-relevant patterns from the
  monolithic `do_kcall`.

- **5 dispatch/encoding functions** (`do_kcall_dispatch`,
  `do_kcall_context`, `remote_dispatch_verified`, `encode_result`,
  `do_kcall_encoded`, `do_kcall_abi`): Decompose the original
  `do_kcall` into verifiable layers.

- **1 divergence handler** (`handle_sleep_error_killed`): Models the
  `Interrupted(Killed)` divergent path separately from the returning
  `handle_sleep_error`.

All are documented in the module-level doc comment (lines 4–125 of the
Verus file) with trust boundary annotations (T1–T6) and an API mapping
table.
