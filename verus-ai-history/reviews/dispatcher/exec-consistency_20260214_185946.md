# Review: dispatcher Exec Consistency (claude-opus-4.6)

## Grade: A

## Verification Status

**PASS** — 66 verified, 0 errors. Confirmed by running `./verus-ai/scripts/verify.sh dispatcher`.

## Review Criteria Assessment

### 1. Were all MISMATCH functions properly restored or equivalence documented?

**Yes.** Two MISMATCH functions were identified and both have thorough equivalence
documentation:

- **`do_kcall`** (§1): The monolithic `extern "C"` function is decomposed into
  four verified layers (`do_kcall_abi` → `do_kcall` → `do_kcall_context` →
  `do_kcall_dispatch`). The decomposition is well-justified: Verus cannot verify
  `extern "C"` functions, `unsafe` blocks, or foreign type conversions. Each
  original match arm is mapped 1:1 to a branch in `do_kcall_dispatch` (lines
  957–999), with correct kcall number constants verified against the original
  `KcallNumber` `#[repr(u32)]` enum in `src/libs/sys/src/sys/number.rs:84–117`.
  All 33 values match.

- **`handle_sleep_error`** (§2): Split into `handle_sleep_error` (non-divergent)
  and `handle_sleep_error_killed` (divergent). The split is the standard Verus
  pattern for conditional divergence. The precondition
  `spec_sleep_error_returns(kind)` correctly excludes `InterruptedKilled`. The
  hardcoded `116i32` for `OperationTimedOut` matches `ETIMEDOUT = 116` in
  `src/libs/sysapi/src/errno.rs:209`.

### 2. Were MISSING functions added with proper verification?

**N/A.** The consistency report shows 0 missing functions — no functions from
the original source were absent from the Verus model.

### 3. Are equivalence justifications sound?

**Yes, with one minor observation.**

- **Kcall number constants**: All 33 spec constants (lines 27–123 of spec file)
  match the source enum constants (number.rs:85–117). Cross-referenced and
  confirmed: Debug=0, GetPid=1, ..., Invalid=u32::MAX.

- **Dispatch routing**: Each match arm in `do_kcall_dispatch` maps correctly:
  - GetPid(1)/GetTid(2) → `success(pid/tid)` ✓
  - Exit(3)/ExitThread(22) → `pm_exit`/`pm_exit_thread` (always error) ✓
  - JoinThread(23) → `convert_sleepable(pm_join_thread(...))` ✓
  - Recv(9) → `convert_sleepable(ipc_recv(tid, pid, arg0))` ✓
  - Resume(5) → `event_resume(arg0)` ✓
  - MutexLock(24) → `convert_sleepable(pm_lock_mutex(pid, tid, arg0, arg1, arg2))` ✓
  - MutexUnlock(25) → `convert_fallible(pm_unlock_mutex(pid, tid, arg0))` ✓
  - CondWait(27) → `convert_sleepable(pm_wait_cond(pid, tid, arg0, arg1, arg2, arg3))` ✓
  - CondSignal(26) → `convert_fallible(pm_signal_cond(pid, tid, arg0, arg1 != 0))` ✓
  - SchedulerYield(20) → `convert_fallible(pm_giveup())` ✓
  - Sleep(29) → `convert_sleepable(pm_sleep(arg0, arg1))` ✓
  - Wildcard → `remote_dispatch_verified(...)` ✓

- **Argument passing**: The original `ipc::recv(tid, pid, arg0 as usize)` passes
  `tid` first, then `pid` — the Verus model `ipc_recv(tid, pid, arg0)` correctly
  preserves this order (line 974). The `as usize` casts are justified on x86-32
  where `usize == u32` (documented at lines 586–589).

- **CondSignal return value**: The original returns `KcallResult::Success(awakened.into())`
  — a count of awakened threads. The Verus postcondition correctly says `value >= 0`
  (not `== 0`), unlike other fallible calls. This is faithful.

- **Minor observation**: The consistency fix report (§3) says "36 extra functions
  and 6 extra structs" but the enumeration in §3 lists 17 external bodies (not 12
  as stated in the bullet point). This is a cosmetic inconsistency in the report
  narrative, not in the code — the actual table at lines 13–48 correctly lists all
  functions. No impact on correctness.

### 4. Does the exec code faithfully represent the original source?

**Yes.** Verified point-by-point:

- **pid/tid retrieval**: Original gets pid first, tid second, with early error
  return. Verus `do_kcall_context` (lines 1062–1073) follows the same sequence.

- **Error construction**: Original uses `KcallResult::Error(e.code.into())`.
  Verus uses `DispatchResult::error(code: i32)` with verified postcondition
  `result@.value == code as int`. The `error_code` type is `i32` matching the
  original `KcallError(i32)`.

- **Success construction**: Original uses `KcallResult::Success(value)` and
  `KcallResult::ok()`. Verus uses `DispatchResult::success(value: i64)` and
  `DispatchResult::ok()` with matching postconditions.

- **ABI encoding**: The original `KcallResult::into::<i64>` returns the value
  field directly for both Success and Error. The Verus `encode_result` (lines
  1147–1152) returns `result.value`, verified to match `spec_encode_result`
  which is also the identity on value. Correct.

- **Killed path**: Original calls `ProcessManager::exit(ErrorCode::Interrupted.into())`
  then `panic!()`. Verus calls `pm_exit_interrupted()` (T4a) then
  `diverge_after_exit()` (T4b), with `ensures false`. The decomposition proves
  exit occurs before divergence — an improvement over an opaque external body.

- **Scoreboard wildcard path**: Original does `ScoreBoard::get_mut()` then
  `scoreboard.dispatch(number, pid, tid, arg0, arg1, arg2, arg3)`. Verus
  `remote_dispatch_verified` (lines 779–818) does `scoreboard_get_mut()` then
  `scoreboard_dispatch_call(number, pid, tid, arg0, arg1, arg2, arg3)` with
  identical argument order. Error routing through `handle_sleep_error` /
  `handle_sleep_error_killed` is correct.

### 5. Does verification still pass?

**Yes.** 66 verified, 0 errors. No regressions.

## Issues Found

### Critical

None.

### Minor

1. **Report narrative inconsistency**: §3 bullet "12 external body functions"
   lists 17 function names in parentheses. The actual code has 15 `external_body`
   functions. This is a documentation-only issue in the fix report, not in the
   verified code.

2. **Trust boundary surface area**: 15 `external_body` functions is a
   non-trivial trust surface. Each assumes postconditions about subsystem
   behavior. While unavoidable for Verus verification of a kernel dispatcher,
   a future CI check diffing the external body postconditions against the actual
   subsystem contracts would reduce drift risk (as recommended in T6 for kcall
   constants).

### Informational

- The `pm_signal_cond` postcondition says `result.value >= 0` on success, which
  is weaker than what the original code provides (`awakened.into()` where
  `awakened: usize` is a count). This is conservative and safe — it could be
  tightened but is not incorrect.

- The `event_resume` external body postcondition only guarantees `result.wf()`,
  not whether the result is success or error. This matches the original which
  returns `KcallResult` directly from the event subsystem without unwrapping.

## Summary

The exec consistency fixes are well-executed. Both MISMATCH functions (`do_kcall`
and `handle_sleep_error`) have sound structural decompositions with thorough
equivalence documentation. All 13 original match arms are faithfully modeled with
correct kcall numbers, argument ordering, and return value semantics. The
verification passes with 66 verified properties and 0 errors. The trust
boundaries are clearly delineated and appropriately scoped. The decomposition
of `handle_sleep_error_killed` into `pm_exit_interrupted` + `diverge_after_exit`
is an improvement over a monolithic external body, as it proves exit-before-divergence
ordering. No correctness issues found.
