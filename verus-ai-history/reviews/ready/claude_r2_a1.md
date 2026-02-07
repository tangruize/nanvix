# Review: ready (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

- None.

### High

- **Boundary models not cross-validated**
  - Location: `RunningThread::from_state()` and `ZombieThread::from_state()` (exec, ready.rs:157–209)
  - Description: The boundary models for `RunningThread` and `ZombieThread` define their own postconditions (e.g., `result.wf()`, identity preservation) but these are self-referential — they are proven trivially by the boundary struct construction, not by the real module implementations. If the real `RunningThread::from_state()` has weaker guarantees (e.g., does not preserve `spec_drop_safe()`), callers relying on the boundary model's postconditions would be unsound.
  - Suggested Fix: Add a `// CROSS-MODULE-CHECK` annotation listing each postcondition that must be confirmed against the real module when verified. Ideally, create a shared trait or spec interface that both the boundary model and real module implement, so the obligation is machine-trackable.

- **`thread_state_mut()` is fully unverified**
  - Location: `ReadyThread::thread_state_mut()` (exec, ready.rs:490)
  - Description: Marked `#[verifier::external]`, this function returns `&mut ThreadState` which allows arbitrary mutation of the thread state. Callers can violate `wf()`, change the thread identity, or corrupt mutex accounting. The trust obligations are documented in comments but are not machine-checked. Since `thread_state_mut()` is used in the kernel for HAL operations (`context_mut()`, `fpu_state_mut()`), any caller bug silently bypasses verification.
  - Suggested Fix: The documentation is excellent and the forwarding methods (`set_interrupt_reason`, `store_mutex_guard`, `take_mutex_guard`) mitigate the risk. Consider adding an audit tag (e.g., `// AUDIT: caller must preserve wf() and spec_id()`) at each call site in the kernel, and track remaining unverified callers in a list.

### Medium

- **`run()` omits raw context pointer — aliasing hazard not modeled**
  - Location: `ReadyThread::run()` (exec, ready.rs:411–432)
  - Description: The original `run()` returns `*mut ContextInformation` which is a raw pointer into the `Box<ThreadState>` now owned by `RunningThread`. This creates an aliasing situation: the caller holds a raw mutable pointer to memory inside `RunningThread`. The verified model omits this pointer entirely. While raw pointer safety is outside Verus scope, the omission means the verification does not capture that `run()` creates a potentially-unsafe aliasing relationship.
  - Suggested Fix: Add a doc comment to the `RunResult` struct or `run()` postconditions noting that the original also returns a raw context pointer with aliasing implications, and that callers must ensure the pointer is not used after the `RunningThread` is dropped.

- **`join_cond()` omitted entirely**
  - Location: Original `ReadyThread::join_cond()` (original, ready.rs:201–203)
  - Description: The `join_cond()` method is omitted from the verified model. While `Condvar` is an opaque sync type, the method is public API and its identity property (returns the same condvar across thread state transitions) is important for correctness of the join/wait protocol.
  - Suggested Fix: If full modeling is infeasible, consider adding a ghost spec function `spec_join_cond_id() -> int` that returns an abstract token, and prove it is preserved across `from_state()` transitions. This would capture the identity invariant without modeling `Condvar` internals.

- **`wf()` does not include admission_time non-negativity**
  - Location: `ReadyThread::wf()` (spec, ready.spec.rs:130–132)
  - Description: The well-formedness predicate only delegates to `state.wf()` and does not include `self.admission_time >= 0`. While `new()` and `from_state()` both call `clock_now()` which ensures non-negativity, a direct struct construction (fields are `pub`) could create a `ReadyThread` with negative admission time that still satisfies `wf()`.
  - Suggested Fix: Either add `self.admission_time >= 0` to `wf()`, or make the `admission_time` field non-public (if Verus supports it via `spec` visibility). Since Verus struct fields are commonly public, the `wf()` approach is preferred.

### Low

- **Struct fields are public in verified model but private in original**
  - Location: `ReadyThread`, `RunningThread`, `ZombieThread` struct definitions (exec, ready.rs:104–151)
  - Description: Original `ReadyThread` has private fields (`state`, `admission_time`) enforcing encapsulation. The verified model has `pub` fields, meaning proof code can construct arbitrary `ReadyThread` values that bypass the constructors. This is a standard Verus modeling pattern, but it means the spec-level invariants must be stated explicitly (they cannot rely on constructor-only creation).
  - Suggested Fix: This is a known Verus limitation. The current approach of using `wf()` preconditions on all methods is correct mitigation. No action needed.

- **Forwarding methods not in original source**
  - Location: `set_interrupt_reason()`, `store_mutex_guard()`, `take_mutex_guard()` (exec, ready.rs:332–396)
  - Description: These three methods are added in the verified model but do not exist in the original `ReadyThread`. They provide verified alternatives to `thread_state_mut()` for common mutations.
  - Suggested Fix: No fix needed — these are beneficial additions that reduce the trust surface. They should be considered for backport to the original source code.

- **EXIT_STATUS_INTERRUPTED hardcoded as 4**
  - Location: `EXIT_STATUS_INTERRUPTED()` (spec, ready.spec.rs:71)
  - Description: The constant 4 is hardcoded based on `EINTR = 4` from `errno.rs:21`. The conversion chain is `ErrorCode::Interrupted = EINTR → ErrorCode as u32 → ExitStatus(4u32)`. This is correct but fragile — if `EINTR` changes, the spec would silently diverge.
  - Suggested Fix: Add a comment citing the full conversion chain: `ErrorCode::Interrupted (lib.rs:47) = EINTR (errno.rs:21) = 4, then ErrorCode as u32 → ExitStatus(4u32)`. Consider a cross-reference test if feasible.

## Positive Observations

- **Excellent documentation**: The trust boundary documentation is outstanding. Every `external_body` and `external` annotation has a clear rationale, the verification model section explains all abstractions, and cross-module dependencies are explicitly noted.
- **Strong state transition verification**: The `run()` and `terminate()` transitions are thoroughly verified — identity preservation, interrupt reason clearing, mutex accounting, drop safety, and well-formedness are all proven across both transitions.
- **Comprehensive proof lemmas**: The proof file includes both atomic lemmas (e.g., `lemma_run_preserves_id`) and composite lemmas (e.g., `lemma_new_then_run`), demonstrating end-to-end correctness of common usage patterns.
- **Clean spec/exec/proof separation**: The three-file split is well-organized. Specs are transparent pass-throughs, the exec code is readable, and proofs are logically grouped by property.
- **Minimal trust surface**: Only two `external_body` functions (`clock_now` and `exit_status_interrupted_value`) and one `external` function (`thread_state_mut`). The forwarding methods reduce reliance on the unverified `thread_state_mut()`.
- **Verification passes cleanly**: 34 verification conditions, 0 errors.
- **Correct constant value**: The `EXIT_STATUS_INTERRUPTED` = 4 is validated against the full `ErrorCode::Interrupted → EINTR → 4 → ExitStatus(4)` conversion chain.
- **View types with ext_equal**: Proper use of `#[verifier::ext_equal]` for structural equality on view types.

## Summary

The verification of `ReadyThread` is thorough and well-engineered. All public functions except `join_cond()` are verified, with state transitions (`run()`, `terminate()`) being the highlight — identity preservation, interrupt semantics, mutex accounting, drop safety, and well-formedness are all proven. The trust surface is minimal and well-documented.

The main areas for improvement are: (1) establishing a mechanism to cross-validate boundary model postconditions against real `RunningThread`/`ZombieThread` implementations, (2) strengthening `wf()` to include admission_time non-negativity, and (3) tracking callers of the unverified `thread_state_mut()`. None of these are correctness bugs in the current verification — they are opportunities to close trust gaps and improve robustness against future changes.
