# Review: kcall_dispatcher (claude-opus-4.6)

## Grade: B+

## Issues Found

### Critical

- None.

### High

- **Location:** `do_kcall()` (exec: dispatcher.rs:431-441)
  **Description:** The main dispatcher function `do_kcall` is marked `#[verifier::external_body]` with a tautological postcondition (`result >= i64::MIN && result <= i64::MAX`). This means the core dispatch logic — that the correct handler is actually invoked for each kernel call number — is entirely unverified. The postcondition tells the caller nothing useful about the return value. While external_body is justified due to unsafe global state (`ProcessManager::get()`, `ScoreBoard::get_mut()`), the function could still have a meaningful postcondition relating the result to the classification, e.g., that `GetPid` returns a success result, or that locally-handled calls produce results consistent with `handle_sleep_error` on failure.
  **Suggested Fix:** Add conditional postconditions to `do_kcall`, e.g.:
  - If `classify_kcall_number(number) == LocalImmediate`, the result encodes a success.
  - If a `Generic` sleep error occurs, the error code is preserved.
  Even as an external_body, richer postconditions document the intended contract and enable downstream verification.

- **Location:** `handle_sleep_error()` / `InterruptedKilled` path (exec: dispatcher.rs:395-401)
  **Description:** The original `InterruptedKilled` path calls `ProcessManager::exit()` and then panics — it is a divergent (non-returning) path. The verified model returns `DispatchResult { is_success: false, value: -1 }`, which is a fabricated return value. The value `-1` does not correspond to any defined `ErrorCode`. If downstream code were to rely on this postcondition, it would reason about a return value that never actually occurs. Furthermore, `handle_sleep_error`'s postcondition unconditionally asserts `!result.is_success`, which is correct for Generic and TimedOut but is a modeling fiction for Killed (since the function diverges).
  **Suggested Fix:** Add a precondition `requires sleep_error.kind != SleepErrorKind::InterruptedKilled` or split the function into a version that only handles the two non-divergent cases. Alternatively, mark the Killed branch with an explicit `assume(false)` and document it as a known divergent trust boundary.

### Medium

- **Location:** Spec `spec_handle_sleep_error` (spec: dispatcher.spec.rs:301-316)
  **Description:** The spec comment on line 299 says InterruptedKilled is "modeled as a success with value -1" but the actual spec returns `is_success: false, value: -1`. The comment contradicts the implementation. While the code is correct (is_success is false), the misleading doc-comment could confuse reviewers.
  **Suggested Fix:** Fix the comment to say "modeled as an error result with value -1".

- **Location:** `DispatchResult::error()` well-formedness vs `handle_sleep_error` Generic path (exec: dispatcher.rs:388-391)
  **Description:** In `handle_sleep_error` for the `Generic` case, the result is constructed with `value: sleep_error.error_code` (an i64). The `wf()` predicate on DispatchResult requires error values to fit in i32 range. While the `SleepError::wf()` precondition ensures `error_code` fits in i32 for Generic kind, this relies on the caller always providing a well-formed SleepError. If the wf precondition were ever weakened, the result construction could produce a non-well-formed result. The inline construction `DispatchResult { is_success: false, value: sleep_error.error_code }` bypasses the `DispatchResult::error()` constructor which takes `i32`, losing the type-level guarantee.
  **Suggested Fix:** Use `DispatchResult::error(sleep_error.error_code as i32)` instead of direct struct construction, which would enforce the i32 constraint at the type level.

- **Location:** `DispatchArgs` struct (exec: dispatcher.rs:106-117)
  **Description:** The `DispatchArgs` struct is defined with a constructor and View implementation but is never used by any verified function, proof lemma, or spec function. It is dead code within the verification module. Its `wf()` predicate is trivially `true`, providing no useful constraint.
  **Suggested Fix:** Either remove `DispatchArgs` or use it in a specification for `do_kcall` to connect the arguments to the dispatch behavior.

- **Location:** `KcallResult` → `i64` encoding not modeled (exec: dispatcher.rs)
  **Description:** The original `do_kcall` returns `KcallResult.into()` which converts to i64. `KcallSuccess` wraps i64 directly, while `KcallError` wraps i32 and converts via `result.0 as i64`. The verified `DispatchResult` models this as a flat `(is_success, value)` pair but does not verify that the encoding from `KcallResult` to `i64` (which is the actual ABI contract with user-space) preserves the success/error distinction. If the encoding were to change, the verification would not catch it.
  **Suggested Fix:** Add a spec function `spec_encode_result(r: DispatchResultView) -> int` that models the `Into<i64>` conversion and prove it is injective (success and error results have disjoint i64 ranges).

### Low

- **Location:** Proof lemmas (proof: dispatcher.proof.rs)
  **Description:** Many proof lemmas have empty bodies (e.g., `lemma_defined_kcalls_classified`, `lemma_local_remote_partition`, `lemma_sleepable_implies_local`). While this means Verus can discharge them automatically from the spec definitions, it also indicates the properties may be definitionally true rather than deep invariants. The proofs are sound but provide limited assurance beyond type-checking the specs.
  **Suggested Fix:** No code change needed. This is informational — the value is in the spec design rather than complex proof reasoning.

- **Location:** `handle_sleep_error` signature (exec: dispatcher.rs:380)
  **Description:** The verified `handle_sleep_error` takes `&SleepError` (by reference) while the original takes `SleepError` by value. This is semantically equivalent for the verification model but is a minor divergence from the original API.
  **Suggested Fix:** Consider using `SleepError` by value to match the original signature exactly.

- **Location:** Spec constants as functions (spec: dispatcher.spec.rs:27-123)
  **Description:** KcallNumber values are modeled as spec functions (`KCALL_GET_PID() -> u32`) rather than spec constants. This is a Verus idiom but means each usage is a function call in specs rather than a const, which is slightly less readable.
  **Suggested Fix:** No change needed — this follows Verus conventions for spec-level constants.

## Positive Observations

- **Complete kcall number coverage.** All 33 KcallNumber variants (including Invalid) are modeled as spec constants, and their u32 values exactly match the `#[repr(u32)]` enum in `src/libs/sys/src/sys/number.rs`.
- **Classification correctness is thorough.** The classification function `classify_kcall_number` is verified against the spec, and individual proof lemmas verify each of the 13 locally-handled kcalls and 20 remote kcalls by name. The `lemma_locally_handled_matches_set` proves equivalence between the classification-based predicate and an explicit set enumeration.
- **Clean spec/proof/exec separation.** The three-file split is well-organized: spec contains pure specifications and View types, proof contains lemmas, and exec contains executable functions with contracts. The include-based composition is clean.
- **Good trust boundary documentation.** The module-level documentation clearly identifies four trust boundaries (T1-T4) and explains why `do_kcall` is external_body. The API mapping table is helpful for reviewers.
- **Partition property proven.** The `lemma_local_remote_partition` proves that every u32 is classified as either local or remote (never both), which is a key safety property for the dispatcher.
- **Error code 110 (ETIMEDOUT) is correctly hardcoded.** The `SPEC_ERROR_TIMED_OUT()` value matches the Linux ETIMEDOUT constant used by `ErrorCode::OperationTimedOut`.
- **Verification passes cleanly.** 42 verification conditions pass with 0 errors in 6 seconds.

## Summary

The kcall_dispatcher verification provides solid coverage of the dispatch classification logic and error handling paths. The main strength is the exhaustive verification that every kernel call number routes to the correct handler category, with formal proofs of the local/remote partition property and sleepable subset inclusion.

The primary limitation is that `do_kcall` — the actual entry point — is external_body with trivial postconditions, meaning the end-to-end property "kcall number N produces the correct result" is not verified. The classification and `handle_sleep_error` are verified in isolation, but nothing connects them to the actual dispatch function. The `InterruptedKilled` divergent path modeling as a concrete return value is a known compromise that should be documented more carefully.

**Recommendations (prioritized):**
1. Strengthen `do_kcall` postconditions even as external_body — at minimum, relate the result to the classification for simple cases (GetPid, GetTid).
2. Handle the `InterruptedKilled` divergent path explicitly rather than fabricating a return value.
3. Fix the contradictory doc-comment in `spec_handle_sleep_error`.
4. Remove or utilize the unused `DispatchArgs` struct.
5. Consider modeling the `KcallResult → i64` encoding to verify the ABI contract.
