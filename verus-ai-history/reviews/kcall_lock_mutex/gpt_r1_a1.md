# Review: kcall_lock_mutex (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Medium
- **Location:** `lock_mutex_model` / `spec_lock_mutex_result` (exec/spec)
  - **Description:** The verification collapses the parsed timeout into a boolean (`has_timeout`) and the spec ignores the concrete timeout value (seconds/nanoseconds). This is too weak to prove that the exact `SystemTime` derived from `(timeout_s, timeout_ns)` is passed to `Mutex::lock`, so correctness for the actual timeout duration is not established.
  - **Suggested Fix:** Model the parsed timeout as an `Option<SystemTimeModel>` (or a `TimeoutView::Finite { seconds, nanoseconds }`) in the exec model and thread it into the lock step; strengthen `mutex_lock_model` and `spec_lock_mutex_result` to reflect the value passed.

- **Location:** `lock_mutex_model` preconditions (exec)
  - **Description:** The original function’s safety contract (caller not the kernel process, no resources held, no ProcessManager reference) is not represented as preconditions or invariants. The current model therefore does not verify the unsafe-use constraints documented in the source.
  - **Suggested Fix:** Add explicit `requires` clauses (or ghost predicates tied to PM state) capturing these safety constraints and, if available, link them to existing PM invariants.

### Low
- **Location:** `lock_mutex_model` signature (exec)
  - **Description:** `pid`/`tid` parameters are omitted, so the verified function is not signature-equivalent to the original. This is safe today because the parameters are only logged, but it leaves a gap if future logic uses them.
  - **Suggested Fix:** Include `pid`/`tid` as parameters (even if unused) or add a lemma asserting their irrelevance to behavior to keep the model aligned with the source.

## Positive Observations
- The pipeline short-circuiting, error propagation, and success conditions are thoroughly captured with clear spec functions and supporting lemmas.
- Timeout parsing is precisely modeled, including the MAX/MAX infinite case and invalid nanosecond handling, and error code linkage is proven.
- Trust boundaries are documented and the exec model structure closely mirrors the original control flow.

## Summary
Overall, the verification is strong on control-flow and error propagation but weak on value-level timeout correctness and the unsafe-use contract. Strengthening timeout value modeling and adding explicit safety preconditions would materially improve assurance while preserving the current clean split between exec/spec/proof.
