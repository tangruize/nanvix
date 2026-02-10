# Review: kcall_dispatcher (claude-opus-4.6)

## Grade: B+

## Verification Result

58 verified, 0 errors. All proofs pass cleanly.

## Issues Found

### Critical

1. **Wrong ETIMEDOUT constant value (110 vs 116)**
   - **Location:** `SPEC_ERROR_TIMED_OUT()` in `dispatcher.spec.rs:291`; hardcoded `110i32` in `handle_sleep_error()` in `dispatcher.rs:492`
   - **Description:** The spec defines `SPEC_ERROR_TIMED_OUT() -> int { 110 }` with the comment "ErrorCode::OperationTimedOut = 110 (ETIMEDOUT in Linux)". However, Nanvix does **not** use Linux errno values. The actual Nanvix value is `ETIMEDOUT = 116` (defined in `src/libs/sysapi/src/errno.rs:209`), and `ErrorCode::OperationTimedOut = ETIMEDOUT` (in `src/libs/error/src/lib.rs:235`). The value 110 is the Linux ETIMEDOUT, not the Nanvix one. The exec code `DispatchResult::error(110i32)` produces a different error code than the original `KcallResult::Error(ErrorCode::OperationTimedOut.into())` which produces 116.
   - **Impact:** The verified code returns a different error code for timeout errors than the original source. This is a semantic equivalence violation.
   - **Suggested Fix:** Change `SPEC_ERROR_TIMED_OUT()` to return `116` and change the hardcoded `110i32` in `handle_sleep_error` to `116i32`. Update the comment accordingly.

### High

*(none)*

### Medium

1. **Tautological postcondition in `do_kcall_dispatch`**
   - **Location:** `do_kcall_dispatch` postcondition, `dispatcher.rs:879-883`
   - **Description:** The postcondition `(args.number == 9u32 || ... || args.number == 20u32) && !result.is_success ==> !result.is_success` is of the form `P ∧ Q ⟹ Q`, which is trivially true. It provides no verification value. This was likely intended to specify a meaningful property about error paths for sleepable/fallible calls (e.g., error code preservation or range constraints).
   - **Suggested Fix:** Either remove this tautological clause or replace it with a meaningful postcondition, such as constraining error values to a valid range or asserting error code preservation from the subsystem call.

2. **CondSignal boolean argument not modeled**
   - **Location:** `pm_signal_cond` external body, `dispatcher.rs:638-641`; cf. original `dispatcher.rs:117`
   - **Description:** The original code calls `pm::signal_cond(pid, tid, arg0 as usize, arg1 != 0)` where the 4th argument is a `bool` (the `broadcast` flag). The verified model passes `arg1: u32` directly to the external body `pm_signal_cond(pid, tid, arg0, arg1)`, losing the boolean conversion semantics. While this doesn't affect soundness (the external body is opaque), it means the verification model doesn't capture the `arg1 != 0` coercion, reducing modeling fidelity.
   - **Suggested Fix:** Change the external body signature to accept `arg1_nonzero: bool` and add a wrapper that computes `args.arg1 != 0` before the call, or document this as an intentional abstraction gap.

3. **`handle_sleep_error_killed` uses `ensures false` as `external_body`**
   - **Location:** `handle_sleep_error_killed()`, `dispatcher.rs:518-526`
   - **Description:** This function has `#[verifier::external_body]` with `ensures false`, the standard Verus idiom for modeling divergence. While the documentation and panic in the body are correct, `ensures false` on an `external_body` is the strongest possible unsound assumption — if this function could somehow return, the verifier would derive arbitrary conclusions. The trust assumption is that the `panic!()` (or the preceding `ProcessManager::exit()` in the original) guarantees divergence. This is well-documented (T4 trust boundary) but worth flagging as a necessary soundness assumption.
   - **Suggested Fix:** No code change needed, but consider adding an assertion or compile-time check in the test suite that verifies the original `handle_sleep_error` actually diverges on `Interrupted(Killed)`.

### Low

1. **`spec_dispatch_result_constrained` is trivially true for 5 of 6 categories**
   - **Location:** `spec_dispatch_result_constrained` in `dispatcher.spec.rs:574-585`
   - **Description:** The constraint only meaningfully restricts `LocalTerminal` (must be error). For all other categories (`LocalImmediate`, `LocalSleepable`, `LocalFallible`, `LocalDirect`, `Remote`), it returns `true` unconditionally. The stronger per-call postconditions live on `do_kcall_dispatch`, making this spec function mostly ceremonial at the top-level `do_kcall` postcondition.
   - **Suggested Fix:** Consider enriching this spec to propagate more per-category guarantees (e.g., `LocalSleepable` success implies non-negative value), or document explicitly that the per-call guarantees are the primary postconditions and this is just a classification-level constraint.

2. **Magic numbers in `classify_kcall_number` and `do_kcall_dispatch`**
   - **Location:** `classify_kcall_number` (`dispatcher.rs:389-407`), `do_kcall_dispatch` (`dispatcher.rs:886-928`)
   - **Description:** These functions use raw integer literals (1, 2, 3, 5, 9, 20, 22, etc.) instead of the spec constants (`KCALL_GET_PID()`, etc.). While Verus's spec constants are `open spec fn` and cannot be called in exec code directly, the lack of named constants in the exec path makes it harder to spot if a numeric value is wrong.
   - **Suggested Fix:** Define `pub const` exec-level constants mirroring the spec constants and use them in the if-chains, or add inline comments (already partially done) to cross-reference.

3. **16 external bodies is a large trust surface**
   - **Location:** `dispatcher.rs:518-687` (16 `#[verifier::external_body]` functions)
   - **Description:** The verification relies on 16 external bodies for subsystem calls. While each is individually justified (T1-T4 trust boundaries), the cumulative trust surface is significant. The postconditions on these external bodies are the foundation of the entire verification, and any incorrect postcondition would silently invalidate the verification.
   - **Suggested Fix:** Consider adding runtime assertion tests that validate the external body postconditions hold for the actual implementations (e.g., testing that `pm_exit` always returns an error, that `pm_get_pid` success implies non-negative value).

## Positive Observations

- **Excellent structural coverage:** All functions from the original `do_kcall` and `handle_sleep_error` are modeled, including both the dispatch match structure and the pid/tid retrieval flow.
- **Clean spec/proof/exec separation:** Specifications are purely in `dispatcher.spec.rs`, proofs in `dispatcher.proof.rs`, and executable verification models in `dispatcher.rs`. The `include!` pattern keeps them logically separated.
- **Thorough trust boundary documentation:** Five trust boundaries (T1-T5) are explicitly identified, justified, and documented in the module-level doc comments. The ABI representation gap (T5) is particularly well-explained.
- **Comprehensive classification proofs:** The proof file verifies totality, partition, subset relationships, and per-call classification for all 32 defined kcall numbers plus the undefined/invalid range.
- **Sound divergence modeling:** The `InterruptedKilled` path is correctly identified as divergent and separated from the non-divergent error handling paths via `spec_sleep_error_returns`.
- **Strong functional postconditions:** `do_kcall_dispatch` and `do_kcall_context` carry meaningful per-call postconditions: GetPid/GetTid return non-negative values on success, ok()-returning calls have value 0, JoinThread success is non-negative, terminal calls always error.
- **Encoding model:** The `spec_encode_result` and related lemmas provide a useful model of the i64 ABI encoding, including the key property that large success values are distinguishable from errors.
- **Full verification success:** 58 items verified with 0 errors, indicating the proof obligations are dischargeable and the model is internally consistent.

## Summary

The verification is well-structured and provides meaningful correctness guarantees for the kernel call dispatcher's routing logic. The spec/proof/exec split is clean, trust boundaries are well-documented, and the dispatch match structure is faithfully modeled with strong per-call postconditions.

The critical issue is the incorrect `ETIMEDOUT` value (110 instead of 116), which means the verified model produces a different error code for timeout errors than the actual Nanvix implementation. This is a straightforward constant fix but demonstrates the risk of manual cross-referencing between the verified model and the source — the value was taken from Linux rather than the Nanvix-specific errno header.

The medium issues (tautological postcondition, boolean argument abstraction gap) are modeling fidelity concerns that don't invalidate the verification but reduce confidence in the spec's completeness. The external body count (16) is inherent to the dispatcher's role as a routing function over many subsystems, and the postconditions on each are reasonable.

**Recommendations:**
1. Fix `SPEC_ERROR_TIMED_OUT()` to `116` and update the hardcoded value.
2. Replace or remove the tautological postcondition.
3. Consider adding a CI step that cross-checks spec constants against the actual enum values (e.g., a Rust test that asserts `ErrorCode::OperationTimedOut.get() as i64 == 116`).
