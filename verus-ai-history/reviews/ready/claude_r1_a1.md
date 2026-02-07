# Review: ready (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

None.

### High

None.

### Medium

- **Location:** `thread_state_mut()` (exec, line 401)
  **Description:** Marked `#[verifier::external]`, allowing callers to mutate `ThreadState` through the returned `&mut` reference with zero verification. A caller could change the thread identity (`spec_id()`), corrupt the mutex set invariant (`wf()`), or break drop safety, and Verus would not detect it. The documented trust obligations (preserve `wf()` and `spec_id()`) are not machine-checked.
  **Suggested Fix:** This is a Verus limitation (`&mut T` returns not supported). Mitigation: add a wrapper that takes a closure `FnOnce(&mut ThreadState)` inside `verus!`, allowing preconditions/postconditions to be checked. Alternatively, provide specific verified mutation methods (e.g., `set_interrupt_reason_on_ready`) that forward to ThreadState methods with full specs, reducing the surface area of the external escape hatch.

- **Location:** `EXIT_STATUS_INTERRUPTED()` (spec, line 69)
  **Description:** Hardcoded to `0`. The original code uses `ErrorCode::Interrupted.into()`, which maps to a specific OS error code (likely non-zero). If sibling modules (e.g., a verified `ZombieThread` consumer) use a different constant for the same concept, or if other exit statuses are also mapped to `0`, the abstract model loses the ability to distinguish them. Cross-module coordination of this constant is not enforced.
  **Suggested Fix:** Define `EXIT_STATUS_INTERRUPTED()` in a shared spec constants file imported by all modules that reference exit statuses. Alternatively, leave it abstract (no fixed value) and only assert distinctness from other exit status constants. If the real value is known (e.g., from `ErrorCode`), use it for faithfulness.

### Low

- **Location:** `clock_now()` (exec, line 73)
  **Description:** `external_body` with no postcondition at all — the returned `int` is completely unconstrained. This means `admission_time` could be negative or decrease between calls, which doesn't match real `SystemTime` semantics (monotonically non-decreasing, non-negative). While documented as a scheduling property outside verification scope, callers that rely on admission time ordering (e.g., FIFO scheduling) cannot prove anything about time relationships.
  **Suggested Fix:** Add a minimal postcondition `ensures result >= 0` to reflect that `SystemTime` is non-negative. For FIFO scheduling verification, a stronger `clock_now() >= old_clock` monotonicity ghost protocol would be needed, but that is out of scope for this module.

- **Location:** `join_cond()` — omitted entirely (exec)
  **Description:** The original `ReadyThread::join_cond()` is not modeled. While `Condvar` is an opaque sync boundary type, the property that a thread's join condition variable is consistently propagated through state transitions (construction → ready → running → zombie) is a cross-cutting correctness concern for thread joining. Omitting it means no verification that `join_cond` is correctly accessible from a `ReadyThread`.
  **Suggested Fix:** Model `Condvar` as an abstract `int` token (like stacks) in `ThreadState`, and verify that `join_cond()` returns the same token that was set at construction. This would be a small extension to `ThreadState`'s verification model.

- **Location:** `ReadyThread` struct fields (exec, lines 99–104)
  **Description:** Both `state` and `admission_time` are `pub` in the verified model, whereas the original has private fields with getter methods. This is standard Verus practice for spec access, but it means the verified model does not enforce the original's encapsulation invariant — any code within the verification boundary could construct or modify a `ReadyThread` by directly setting fields, bypassing constructor postconditions.
  **Suggested Fix:** No action needed for correctness (Verus requires pub fields for spec functions). Document this as a modeling artifact. If Verus adds support for private fields with spec accessors in the future, switch to that pattern.

- **Location:** Proof lemmas (proof, all lemmas)
  **Description:** All 18 proof lemmas have empty bodies, meaning they are trivially discharged by the SMT solver. While this indicates well-structured specs, the lemmas don't add verification power beyond what the function postconditions already guarantee. They primarily serve as documentation and regression guards.
  **Suggested Fix:** No action needed. The lemmas are valuable as regression tests — if specs change incompatibly, these would fail. Consider adding a non-trivial composite lemma, e.g., proving that `new()` followed by `run()` produces a RunningThread with no interrupt and the original identity, exercising the composition of multiple function specs.

## Positive Observations

- **Complete function coverage:** 7 of 9 original functions are fully verified; the remaining 2 (`thread_state_mut`, `join_cond`) have justified documented exclusions at well-defined trust boundaries.
- **Zero `assume` statements:** The entire verification relies solely on `external_body` (2 instances) and `#[verifier::external]` (1 instance), all of which are well-documented and justified by Verus language limitations or HAL/sync boundaries.
- **Strong state transition specs:** `run()` and `terminate()` have comprehensive postconditions covering identity preservation, interrupt reason extraction/clearing, well-formedness, mutex accounting, and drop safety — the key safety properties for thread lifecycle management.
- **Boundary model pattern:** `RunningThread` and `ZombieThread` are modeled as lightweight boundary types with just enough spec surface to verify ReadyThread's transitions, avoiding full dependency on sibling modules while enabling independent verification.
- **Mutex accounting preservation:** The verification traces mutex set membership and count through all transitions, proving that scheduling operations (ready→running, ready→zombie) do not leak or corrupt lock state.
- **Drop safety propagation:** `spec_drop_safe()` is tracked through construction, `run()`, and `terminate()`, ensuring that the drop-safety invariant is preserved across the thread's ready-state lifecycle.
- **Excellent documentation:** The trust boundary, verification model, and cross-module dependencies are thoroughly documented in module-level comments, making the verification assumptions auditable.
- **Clean three-file separation:** Spec, proof, and exec code are well-separated with clear responsibilities. The spec file is a pure transparent pass-through to ThreadState specs, the proof file contains standalone lemmas, and the exec file has all implementation logic.
- **Verification passes cleanly:** 30 verified obligations, 0 errors.

## Summary

This is a high-quality verification of the `ReadyThread` module. The abstraction choices are well-motivated: `Box` is transparent, `SystemTime` becomes `int`, HAL types (`ContextInformation`, `FpuState`) and sync types (`Condvar`) are correctly identified as out of scope. The core safety properties — identity preservation, well-formedness, drop safety, mutex accounting, and correct state transition semantics for `run()` and `terminate()` — are all proven.

The main trust gap is `thread_state_mut()`, which is a necessary escape hatch due to Verus limitations but creates an unverified mutation surface. The `EXIT_STATUS_INTERRUPTED` constant should be coordinated with sibling modules to prevent cross-module inconsistency. The `clock_now()` function's lack of any postcondition is acceptable for this module but limits what callers can prove about scheduling fairness.

The boundary model pattern for `RunningThread` and `ZombieThread` is well-designed — it enables verifying state transitions without importing full sibling module verification, while clearly documenting the cross-module obligations that must hold when those modules are independently verified. Overall, this verification captures the essential correctness properties of a kernel thread state management module with appropriate abstraction of hardware and synchronization boundaries.
