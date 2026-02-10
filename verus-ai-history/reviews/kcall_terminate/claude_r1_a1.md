# Review: kcall_terminate (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

_None._

### High

- **Location:** `terminate_model` (exec — terminate.rs:225)
  **Description:** The verified model does not capture the mutation of `ProcessManager` state. The original function takes `&mut ProcessManager`, meaning it modifies the process manager (e.g., removing the process from internal tables, updating scheduler state). The model's `process_manager_terminate(pid: u32)` is a stateless function that returns a result without modeling any state transition. This means the verification cannot prove that the PM is left in a consistent state after a successful terminate, or that double-terminate on the same PID would fail.
  **Suggested Fix:** Introduce a ghost `ProcessManagerView` state parameter that is threaded through the model. Add a postcondition on `process_manager_terminate` relating the input state to an output state (e.g., the terminated PID is no longer present). This would allow proving properties like idempotency failure and state consistency. The documentation acknowledges this as out-of-scope, but for a kernel call that mutates critical state, this is the most significant verification gap.

### Medium

- **Location:** `process_manager_terminate` (exec — terminate.rs:192)
  **Description:** The `external_body` postcondition for `process_manager_terminate` is very weak — it only ensures the result is a valid variant and that error codes are positive. It does not relate the `pid` input to the outcome in any way. The model allows any pid to succeed or fail arbitrarily, which means the verification proves nothing about the relationship between input PID values and terminate outcomes. For instance, there is no postcondition that terminating PID 0 (kernel process) always fails.
  **Suggested Fix:** Add postconditions that capture known contracts from the real `ProcessManager::terminate`, such as: terminating the kernel process (PID 0) always returns an error; the error code on failure is one of a known set (e.g., `InvalidArgument`, `NoSuchProcess`).

- **Location:** `try_from_process_identifier` (exec — terminate.rs:173)
  **Description:** No postcondition relates a successful parse result back to the input `arg0`. On `PidOk { pid }`, there is no guarantee that `pid == arg0` (or any defined relationship). While this doesn't matter for the current proofs (since `pid` is passed to `process_manager_terminate` which is also external_body), it weakens the overall specification — the model could silently accept a PID parser that returns a different PID than requested.
  **Suggested Fix:** Add a postcondition: `result.spec_view() matches PidParseOutcomeView::PidOk { pid } ==> pid == arg0 as nat`. This captures the identity property of the parsing step.

### Low

- **Location:** `USIZE_MAX_X86_32()` (spec — terminate.spec.rs:34-36)
  **Description:** The spec constant `USIZE_MAX_X86_32()` is defined but never used anywhere in the spec, proof, or exec files. This is dead code that adds confusion.
  **Suggested Fix:** Remove the unused constant.

- **Location:** `terminate_model` (exec — terminate.rs:225)
  **Description:** The model uses `pid: u32` when calling `process_manager_terminate`, whereas the original code passes a `ProcessIdentifier` (a typed wrapper). This loses the type-level guarantee that only successfully-parsed PIDs reach the terminate call. The model relies on control flow (PidOk match arm) rather than type safety to ensure this invariant.
  **Suggested Fix:** Consider defining a ghost `ValidPid` type or adding a spec predicate `spec_is_valid_pid(pid: u32)` to make this invariant explicit in the postconditions.

- **Location:** Original source line 26-27 (terminate.rs in src/kernel/)
  **Description:** The original code logs the error with `error!("{error:?}")` before returning on PID parse failure. The verified model does not model this logging. While logging has no functional effect, in a kernel context, the absence of logging on error paths could be operationally significant (e.g., debugging). The model cannot verify that errors are always logged before being returned.
  **Suggested Fix:** This is acceptable for functional verification. If logging discipline is important, consider adding a ghost flag to track that `log()` was called on error paths.

## Positive Observations

- **Clean three-file split.** The spec/proof/exec separation is exemplary. Spec files contain only view types and spec functions, proof files contain only lemmas, and exec code is focused on the verified model with external bodies. The `include!()` composition is well-organized.

- **Comprehensive documentation.** The module-level doc comment in the exec file is outstanding — it documents verified properties, out-of-scope properties, trust boundaries, and an API mapping table. This is best-practice for verification modules.

- **Strong error propagation proofs.** The suite of 9 lemmas thoroughly covers the error propagation pipeline: short-circuit behavior (`lemma_pid_parse_short_circuit`), error code preservation for both steps, biconditional success, and result exhaustiveness. These are the core correctness properties for a dispatch function.

- **Well-justified trust boundaries.** The two `external_body` functions are clearly documented with what they model and where the real implementations are verified. The choice to treat PID parsing and PM terminate as trust boundaries is architecturally sound for a layered verification approach.

- **Verification passes cleanly.** All 11 verified items pass with zero errors, confirming the proofs are mechanically checked.

- **Error code linkage.** The `lemma_error_code_matches` proof that connects the spec constant `ERROR_CODE_INVALID_ARGUMENT()` to the concrete `ErrorCode::InvalidArgument as int` value is a good practice that prevents spec/impl drift.

## Summary

This is a well-executed verification of a kernel call dispatch function. The `terminate` kcall is essentially a two-step pipeline (parse PID → call PM terminate), and the verification thoroughly proves the error propagation and composition properties of this pipeline. The three-file split is clean, documentation is excellent, and all proofs pass.

The main gap is the **absence of state modeling** for the `ProcessManager` mutation. The original function takes `&mut ProcessManager`, implying significant state changes, but the model treats it as a stateless operation. This is the most impactful limitation because it means the verification cannot reason about whether the terminate operation actually changes the system state correctly. The documentation explicitly acknowledges this as out-of-scope (deferred to PM module verification), which is a reasonable architectural choice for a layered approach, but it does mean this module in isolation only verifies the dispatch logic, not the semantic effect of termination.

The `external_body` postconditions could also be strengthened: the PID parser should guarantee identity (parsed PID equals input), and the PM terminate should capture known failure conditions (e.g., kernel process cannot be terminated).

**Recommendation:** The verification is appropriate for its stated scope. To elevate to A/A+, add a ghost state parameter to model PM state transitions, strengthen the external_body postconditions to capture input-output relationships, and remove the dead `USIZE_MAX_X86_32` constant.
