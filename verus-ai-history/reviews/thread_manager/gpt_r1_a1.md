# Review: thread_manager (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Coverage gap for ThreadRef/ThreadRefMut** (mod.rs exec): `ThreadRef` / `ThreadRefMut` enums and their `thread_state()` / `thread_state_mut()` methods are explicitly omitted from the Verus model. This violates the coverage criterion that all functions in the original source are verified. **Suggested Fix:** Add minimal boundary models for these enums with specs that delegate to the underlying thread state, or create a separate verified module and explicitly link it here.

### Medium
- **Overflow behavior not modeled** (thread_manager.rs exec `create_thread`): The spec requires `next_id.value < i32::MAX`, but the original code has no overflow check. This restricts behavior and makes ID uniqueness rely on an unverified system-wide assumption. **Suggested Fix:** Either add an explicit runtime check/error path in exec code, or model the overflow behavior and adjust uniqueness properties accordingly.
- **Resource preservation not specified** (thread_manager.rs exec/spec): `create_thread` passes `kernel_stack`, `user_stack`, and `user_tda` to `ReadyThread::new`, but the boundary model omits postconditions that these values are preserved. The spec is weaker than actual behavior and does not prove that the thread owns the intended resources. **Suggested Fix:** Extend the boundary `ReadyThread::new` ensures to include stack/TDA preservation (as in ready.rs), or reuse the ready module’s ReadyThread spec.
- **Admission time omitted from boundary model** (thread_manager.rs exec): The boundary ReadyThread omits `admission_time`, which is set in the real constructor and used for scheduling. This weakens semantic equivalence for callers that rely on admission-time properties. **Suggested Fix:** Either include `admission_time` in the boundary model or explicitly tie thread_manager to the ready module’s verified ReadyThread type.

### Low
- **OOM behavior elided** (thread_manager.rs exec docs): The model assumes allocation succeeds (Box elided), so potential panic paths on OOM are not captured. **Suggested Fix:** Add an explicit spec note about fatal OOM behavior or model a failure path if relevant to system safety.

## Positive Observations
- Core ID allocation properties (monotonicity, uniqueness, kernel ID distinctness) are specified and proven with clear lemmas.
- The exec/spec/proof split is clean, with specs in a dedicated file and proofs as separate lemmas.
- No `assume` or `external_body` is used in the thread_manager module itself; trust boundaries are documented.

## Summary
The verification captures the core ID-allocation logic but omits ThreadRef/ThreadRefMut coverage and weakens several semantic properties (resource preservation, admission time, overflow behavior). Strengthening the boundary model or reusing the ready module’s specs would close the largest gaps and improve equivalence. Overall, the module is close but incomplete with respect to the requested coverage and semantic fidelity.
