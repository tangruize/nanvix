# Review: ready (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

_None._

### High

- **Location:** `ReadyThread`, `RunningThread`, `ZombieThread` structs (exec: `ready.rs:104-151`)
  - **Description:** All struct fields are `pub` in the verified model, whereas the original `ReadyThread` has private fields (`state`, `admission_time`) accessed only through methods. This breaks the encapsulation invariant of the original code. In Verus, public fields allow proof code to construct arbitrary instances and access fields directly, potentially bypassing intended API contracts. While this may be motivated by Verus proof ergonomics, it weakens the guarantee that the struct's invariant (`wf()`) is maintained: any caller outside the module can construct a `ReadyThread` with `admission_time < 0` or a non-`wf()` state.
  - **Suggested Fix:** If Verus supports it, use `pub(crate)` or keep fields private with spec accessor functions. Alternatively, add a comment documenting that public fields are a Verus modeling necessity, and that construction should only occur via `new()`/`from_state()` which establish `wf()`.

### Medium

- **Location:** `EXIT_STATUS_INTERRUPTED()` spec constant (spec: `ready.spec.rs:73`)
  - **Description:** The constant is hardcoded to `4`, representing `ErrorCode::Interrupted` → `EINTR`. The derivation chain (`ErrorCode::Interrupted` → `From<ErrorCode> for i32` → `errno as i32` → `EINTR = 4`) is documented in comments but the value is not mechanically linked to the source definition. If the kernel's `EINTR` value or the `From` impl changes, this constant silently becomes incorrect, breaking the soundness of `terminate()` verification.
  - **Suggested Fix:** Add a cross-module verification obligation comment (similar to the RunningThread/ZombieThread boundary notes) requiring periodic audit of this constant against `sys::error::ErrorCode::Interrupted`. Consider adding an `external_body` function that asserts the value matches at runtime in debug builds.

- **Location:** Forwarding methods `set_interrupt_reason`, `store_mutex_guard`, `take_mutex_guard` (exec: `ready.rs:353-417`)
  - **Description:** These three methods do not exist in the original `ReadyThread` (original: `src/kernel/src/pm/thread/ready.rs`). In the original code, callers mutate thread state via `thread_state_mut().set_interrupt_reason(...)` etc. Adding these forwarding methods extends the verified API surface beyond the original, creating a semantic divergence. While the motivation is sound (providing verified paths instead of using the unverified `thread_state_mut()` escape hatch), this means the verified code has a different public interface than the original.
  - **Suggested Fix:** Document these as "verification-only API extensions" more prominently. If the original code is ever refactored to add these forwarding methods, this becomes moot. Otherwise, note that callers in the verified model use a different (safer) interface than the original runtime code.

- **Location:** `clock_now()` external_body (exec: `ready.rs:74-80`)
  - **Description:** The postcondition `result >= 0` is minimal. While non-negativity is correct for `SystemTime`, the absence of a monotonicity guarantee means the verification cannot reason about scheduling ordering properties. For instance, a thread created later could have a smaller `admission_time` than one created earlier, which would break FIFO scheduling assumptions. The documentation acknowledges this is out of scope but does not note which callers might depend on ordering.
  - **Suggested Fix:** This is acceptable for the current scope. If future verification of the scheduler depends on FIFO ordering of ready threads by `admission_time`, strengthen the postcondition to model monotonicity (e.g., via a ghost global counter).

### Low

- **Location:** `join_cond()` omission (exec: `ready.rs`)
  - **Description:** The original `ReadyThread::join_cond()` method is entirely omitted. This is documented and justified (returns opaque `Condvar`, sync boundary). However, `join_cond()` is part of the thread lifecycle — it enables thread join operations. Its omission means the verification cannot reason about join correctness.
  - **Suggested Fix:** No immediate action needed. When the sync subsystem (Condvar) is verified, add a boundary model for `join_cond()` to complete the thread lifecycle verification.

- **Location:** `run()` return type modeling (exec: `ready.rs:144-151`)
  - **Description:** The `RunResult` struct omits the `*mut ContextInformation` raw pointer from the original return type. This is well-justified (unsafe HAL boundary), but the raw pointer is the mechanism for context switching — the most critical operation in the scheduler. The verification cannot make any statement about whether the returned context pointer is valid or correctly points to the thread's pinned context.
  - **Suggested Fix:** No action for the current scope. Document this as a key gap for future HAL-level verification. If Verus gains raw pointer support, model the pointer validity invariant.

- **Location:** `thread_state_mut()` external escape hatch (exec: `ready.rs:520-523`)
  - **Description:** Marked `#[verifier::external]` due to Verus's inability to express `&mut T` return types. The trust obligations are thoroughly documented (must preserve `wf()` and `spec_id()`). The existence of verified forwarding methods mitigates this well. However, any call site using `thread_state_mut()` operates outside the verification boundary.
  - **Suggested Fix:** The existing documentation and mitigation are good. Consider adding a `// AUDIT` comment listing known call sites in the kernel that use this method, so reviewers can track unverified mutations.

- **Location:** Proof lemmas `lemma_run_preserves_id`, `lemma_run_clears_interrupt`, etc. (proof: `ready.proof.rs:112-185`)
  - **Description:** The run() transition lemmas reason about a manually constructed `post_state` rather than the actual `run()` function output. This is because proof lemmas cannot call exec functions. While the lemmas are correct (they prove properties of the state transformation that `run()` performs), there is a semantic gap: the lemmas prove properties of a structural copy `ThreadState { interrupt_reason: None, ..self.state }`, while `run()` actually calls `state.take_interrupt_reason()`. The connection between these two is established by `take_interrupt_reason()`'s postconditions, but this indirection could mask subtle bugs if `take_interrupt_reason()` has side effects beyond clearing the field.
  - **Suggested Fix:** No immediate action — the current approach is standard for Verus proofs. The ThreadState's `take_interrupt_reason()` postconditions are comprehensive and the structural equivalence holds. This is a minor observation about proof style, not a correctness issue.

## Positive Observations

- **Excellent documentation:** Every external_body, verifier::external, and modeling decision is thoroughly documented with rationale, trust obligations, and cross-module verification notes. The module-level doc comments are comprehensive.
- **Sound trust boundary:** The three external_body/external items are well-justified: `clock_now()` (OS time abstraction), `exit_status_interrupted_value()` (int literal bridging), and `thread_state_mut()` (Verus &mut limitation). No unjustified assumes.
- **Comprehensive state transition verification:** Both `run()` and `terminate()` are verified with identity preservation, well-formedness, mutex accounting, drop safety, and correct field extraction/setting. The `run()` spec captures the interrupt-reason-clearing semantics correctly.
- **Good proof organization:** The proof file is logically organized into sections (construction, identity, state transitions, composite lemmas, view equality). Lemmas are focused and individually meaningful.
- **Cross-module boundary awareness:** The RunningThread and ZombieThread boundary models include explicit "CROSS-MODULE-CHECK" comments listing postconditions that must be confirmed when those modules are independently verified.
- **Verification passes cleanly:** 34 verified, 0 errors. The verification is sound and complete within its scope.
- **Forwarding methods are a good mitigation:** Adding verified `set_interrupt_reason`, `store_mutex_guard`, and `take_mutex_guard` methods reduces dependence on the unverified `thread_state_mut()` escape hatch, shrinking the trust boundary in practice.
- **Drop safety modeling:** The connection between the runtime `check_drop_safe()` and the spec-level `spec_drop_safe()` is proven, faithfully modeling the original `Drop::drop()` check.

## Summary

The Verus verification of `ReadyThread` is thorough and well-executed. All 8 of 9 original functions are covered (the omitted `join_cond()` is an opaque sync boundary type, justifiably excluded). The specifications capture the essential correctness properties: identity immutability, well-formedness preservation across all operations, correct state transitions for `run()` and `terminate()`, mutex accounting consistency, and drop safety. The trust boundary is minimal and well-documented (2 external_body functions for OS/language bridging, 1 external function for a Verus language limitation).

The main areas for improvement are: (1) public struct fields weaken encapsulation guarantees — the verification cannot prevent construction of ill-formed instances outside the module; (2) the `EXIT_STATUS_INTERRUPTED` constant is hardcoded without mechanical linkage to the source definition; (3) the three forwarding methods extend the API surface beyond the original code. None of these are critical — they are modeling trade-offs that are well-documented and reasonable for the current verification scope. The overall quality is high, with clear paths identified for future strengthening (monotonic clocks, HAL pointer verification, Condvar modeling).
