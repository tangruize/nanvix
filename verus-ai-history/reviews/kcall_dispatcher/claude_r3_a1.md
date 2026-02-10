# Review: kcall_dispatcher (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

- None.

### High

- **`handle_sleep_error_killed` uses `ensures false` with `external_body`**
  - Location: `handle_sleep_error_killed()` (exec, line 518–526)
  - Description: This function has `#[verifier::external_body]` with `ensures false`, the strongest possible postcondition. It models the divergent path where `ProcessManager::exit()` is called and the thread panics. While the `panic!()` in the body guarantees divergence at runtime, the `ensures false` contract allows the verifier to treat any post-call code as dead code. If the original code were ever modified to handle the Killed path without panicking (e.g., graceful thread termination), this would become unsound — the verifier would still assume unreachability. The trust boundary T4 documents this assumption, but the blast radius of `ensures false` is high.
  - Suggested Fix: Add a prominent `// SOUNDNESS NOTE` comment at the call sites (`convert_sleepable`, `remote_dispatch_verified`) emphasizing that any change to the Killed path in the original must be reflected here. Alternatively, if Verus gains support for the `!` (never) type, migrate to that.

### Medium

- **Magic numbers in exec dispatch logic instead of spec constants**
  - Location: `classify_kcall_number()`, `do_kcall_dispatch()`, `do_kcall_context()`, `do_kcall()` (exec)
  - Description: The exec functions use raw `u32` literals (e.g., `1u32`, `2u32`, `3u32`, `22u32`) rather than referencing the spec constants (`KCALL_GET_PID()`, `KCALL_EXIT_THREAD()`, etc.). While correctness is guaranteed by the postcondition `result =~= spec_classify_kcall(number)` linking to the spec, the raw numbers hurt readability and maintainability. If a new kcall is added or numbers change, every if-else branch must be updated manually with no named-constant guidance.
  - Suggested Fix: Define `exec` constants (e.g., `const EXEC_KCALL_GET_PID: u32 = 1;`) or use Verus `spec_fn` values in ghost context to bridge to the exec level. This is a maintainability improvement, not a correctness issue.

- **No postcondition constraining CondSignal success value**
  - Location: `pm_signal_cond()` (exec, line 645) and `do_kcall_dispatch()` postconditions (exec, line 862)
  - Description: In the original code, `pm::signal_cond` returns `Ok(awakened)` where `awakened` is a count of woken threads (an `i64` ≥ 0). The verified `pm_signal_cond` external body only ensures `result.wf()` but places no constraint on the success value. The `do_kcall_dispatch` postconditions also omit CondSignal (number 26) from the ok()-returning set, which is correct since its success value is not 0, but there is no alternative postcondition constraining the value (e.g., `result.value >= 0`).
  - Suggested Fix: Add `result.succeeded ==> result.value >= 0` to `pm_signal_cond` ensures clause, and add `args.number == 26u32 && result.is_success ==> result.value >= 0` to `do_kcall_dispatch` postconditions.

- **`ScoreboardDispatchOutcome` success path bypasses result constructors**
  - Location: `remote_dispatch_verified()` (exec, lines 734–738)
  - Description: When the scoreboard dispatch succeeds, a `DispatchResult` is constructed directly from fields (`DispatchResult { is_success: ..., value: ... }`) instead of using `DispatchResult::success()` or `DispatchResult::error()`. While this verifies (the `wf()` postcondition passes due to `ScoreboardDispatchOutcome.wf()`), it bypasses the constructor postconditions and spec-level `result@ == spec_success_result(...)` / `spec_error_result(...)` linkage. This means the returned `DispatchResult@` view is not explicitly linked to spec constructors for remote dispatch results.
  - Suggested Fix: Split into two branches: `if dispatch_outcome.result_is_success { DispatchResult::success(dispatch_outcome.result_value) } else { DispatchResult::error(dispatch_outcome.result_value as i32) }`. This provides stronger postcondition linkage.

### Low

- **Representation gap: `as usize` vs `u32` in subsystem boundary functions**
  - Location: External bodies (exec, lines 536–693)
  - Description: The original code passes `arg0 as usize` to many subsystem calls (e.g., `ipc::recv(tid, pid, arg0 as usize)`, `pm::lock_mutex(pid, tid, arg0 as usize, ...)`). The verified external bodies accept `arg0: u32` directly. On the target 32-bit x86 platform, `u32` and `usize` are identical, so this is semantically equivalent. However, the conversion is not explicitly documented as a trust assumption. If the code were ever ported to a 64-bit target, `usize` would differ from `u32`.
  - Suggested Fix: Add a brief comment to the external body section documenting that `u32 == usize` on x86-32.

- **`ProcessIdentifier`/`ThreadIdentifier` are `i32` but modeled as `i64`**
  - Location: `pm_get_pid()`, `pm_get_tid()` (exec, lines 542–554)
  - Description: In the original, `ProcessIdentifier(i32)` and `ThreadIdentifier(i32)` are i32-wrapped types. The verified model uses `FallibleOutcome.value: i64` to represent them, widening the type. While this is safe (i32 fits in i64), the `ensures result.value >= 0` postcondition could be tightened to `result.value <= i32::MAX as i64` to match the actual range.
  - Suggested Fix: Add upper-bound constraint: `result.succeeded ==> result.value <= i32::MAX as i64`.

- **`do_kcall` entry point has no `requires` clause but `do_kcall_encoded` requires `args.wf()`**
  - Location: `do_kcall()` (exec, line 1016) vs `do_kcall_encoded()` (exec, line 1085)
  - Description: `do_kcall` has no precondition (any DispatchArgs accepted), while `do_kcall_encoded` requires `args.wf()`. Since `args.wf()` is always `true`, this is technically harmless, but the inconsistency may confuse maintainers about whether `wf()` is meaningful.
  - Suggested Fix: Either remove `requires args.wf()` from `do_kcall_encoded` or add it to `do_kcall` for consistency.

- **Proof lemmas are trivially discharged by Verus SMT solver**
  - Location: Most lemmas in `dispatcher.proof.rs` have empty bodies
  - Description: Nearly all proof lemmas (e.g., `lemma_defined_kcalls_classified`, `lemma_local_remote_partition`, `lemma_getpid_is_immediate`, etc.) have empty bodies `{}`. This means the SMT solver discharges them automatically. While this is fine for correctness, it suggests the lemmas are documenting properties rather than proving non-trivial ones. This is not a bug — it's actually positive evidence that the spec is well-structured.
  - Suggested Fix: No change needed. Consider adding a note that these serve as regression checks: if the spec changes, Verus will flag which properties break.

## Positive Observations

- **Complete function coverage.** Both original functions (`do_kcall`, `handle_sleep_error`) are fully modeled. The verification decomposes `do_kcall` into `do_kcall` → `do_kcall_context` → `do_kcall_dispatch` for cleaner proof structure, which is good verification engineering.

- **Clean spec/proof/exec separation.** The split is textbook-quality: spec file contains view types, spec constants, and spec functions; proof file contains proof lemmas; exec file contains structures, implementations, and verified functions. The `include!()` mechanism ties them together cleanly.

- **All 33 kcall constants verified for consistency.** `lemma_kcall_constants_consistency` explicitly asserts all 33 spec constant values match the original `#[repr(u32)]` enum. Cross-verified against `src/libs/sys/src/sys/number.rs` — all values match exactly.

- **Well-documented trust boundaries.** Five trust boundaries (T1–T5) are clearly identified in the module documentation, covering ProcessManager access, ScoreBoard access, PM subsystem calls, divergence, and ABI representation. Each external body references its trust boundary.

- **Dispatch routing fully verified.** The core value proposition — that every kcall number routes to the correct handler — is proven. The 13 locally-handled kcalls and the remote wildcard path are exhaustively classified and proven correct.

- **Error handling chain verified end-to-end.** The `handle_sleep_error` function correctly preserves Generic error codes and maps TimedOut to ETIMEDOUT (116). The divergent Killed path is properly isolated. The `convert_sleepable` and `convert_fallible` helper functions are verified to correctly route errors.

- **Strong success-value postconditions.** The verification proves that `ok()`-returning calls (Recv, MutexLock, CondWait, Sleep, MutexUnlock, SchedulerYield) return value 0 on success, JoinThread returns a non-negative value, and GetPid/GetTid return non-negative identifiers. These are meaningful functional correctness properties.

- **ABI encoding explicitly modeled.** The `encode_result` function and `do_kcall_encoded` bridge the gap between the typed `DispatchResult` model and the raw `i64` C ABI return value, with proofs that error encodings fit in i32 range and large success values are distinguishable.

- **All 61 verification conditions pass.** `./verus-ai/scripts/verify.sh kcall_dispatcher` reports 61 verified, 0 errors in 7 seconds.

## Summary

This is a high-quality verification of a kernel call dispatcher that focuses on the right property: **dispatch routing correctness**. The verification model correctly identifies that `do_kcall` is fundamentally a routing function — it classifies kcall numbers and dispatches to subsystem handlers — and verifies that classification and routing are total, correct, and exhaustive.

The trust boundaries are well-placed: subsystem calls (ProcessManager, ScoreBoard, IPC, event, PM) are external bodies whose internal correctness is out of scope, while the routing logic connecting them is fully verified. The one aggressive trust assumption (`ensures false` on the Killed divergent path) is justified by the `panic!()` in the body and documented clearly.

The spec is appropriately strong without being over-specified. It captures the essential correctness properties (routing correctness, error code preservation, success value constraints, terminal call error guarantee) while leaving subsystem-internal behavior to their respective verification modules.

**Key recommendations:**
1. Replace magic numbers in exec code with named constants for maintainability.
2. Add `result.value >= 0` constraint to `pm_signal_cond` success case.
3. Tighten pid/tid value range to `<= i32::MAX` to match actual types.
4. These are all polish items — the verification is sound and captures the essential correctness properties of the dispatcher.
