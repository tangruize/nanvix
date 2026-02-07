# Review: running_thread (claude-opus-4.6)

## Grade: A-

## Verification Result

- **46 verified, 0 errors** — all obligations discharge successfully.

## Issues Found

### Critical

*None.*

### High

- **Location:** `take_mutex_guard` (exec, running.rs:428–441)
  **Description:** The original `RunningThread::take_mutex_guard` returns `Option<MutexGuard>`, allowing callers to handle the `None` case (mutex address not found). The verified version requires `spec_has_mutex(address@)` as a precondition and returns unit, eliminating the `None` path entirely. This is a precondition strengthening: any caller that legitimately encounters `None` in production is outside the verification boundary. While documented as trust assumption T2, this means the verification does not cover the error-handling path at all.
  **Suggested Fix:** Consider modeling the return as `Option<()>` or a bool, with a postcondition that the result is `Some(())` iff the precondition held. Alternatively, add a spec-only lemma showing that under the precondition, the original `BTreeMap::remove` always returns `Some`. This would make the strengthening a proven consequence rather than an assumption.

### Medium

- **Location:** `thread_state_mut` (exec, running.rs:475–478)
  **Description:** Marked `#[verifier::external]` because Verus cannot express `&mut T` return types. This creates an unverified escape hatch: any caller that obtains `&mut ThreadState` through this method can silently violate `wf()` or `spec_id()` immutability. The doc comments list trust obligations (preserve wf and spec_id), but these are not machine-checked.
  **Suggested Fix:** No immediate fix possible given Verus limitations. Mitigation: add a runtime debug assertion in the original code that checks `wf()`-equivalent conditions after each use of `thread_state_mut()`, or audit all call sites and document which ones are outside the boundary. Consider wrapping specific mutations (e.g., `fpu_state_mut`, `context_mut`) as forwarding methods with contracts, similar to how `put_mutex_guard`/`take_mutex_guard` already work.

- **Location:** `sleep`, `schedule`, `exit` return types (exec, running.rs:283, 308, 368)
  **Description:** The original functions return tuples `(TargetThread, *mut ContextInformation)`. The verified versions return only `TargetThread`, omitting the raw context pointer. While the pointer is a HAL boundary type, the *pairing* — that the returned pointer belongs to the same thread as the returned state — is a safety-relevant property that is not captured.
  **Suggested Fix:** Consider adding a ghost return value (e.g., `Ghost<int>` representing an abstract context token) with a postcondition tying it to the thread identity. This would model the pairing without requiring the actual raw pointer.

- **Location:** Boundary models — `SleepingThread`, `ReadyThread`, `ZombieThread` (exec, running.rs:88–125)
  **Description:** These are standalone boundary models defined within the running module. If the real sibling modules (`sleeping.rs`, `ready.rs`, `zombie.rs`) are verified independently with different specs, the boundary models here could silently diverge. The `CROSS-MODULE-CHECK` comments document this obligation but there is no automated mechanism to enforce it.
  **Suggested Fix:** When sibling modules are verified, add a cross-module consistency check (e.g., a proof-only test that constructs a value via the boundary model and the real model and asserts postcondition equivalence). Alternatively, extract boundary specs into a shared trait or spec file.

### Low

- **Location:** `put_mutex_guard` precondition (exec, running.rs:398)
  **Description:** The precondition includes `old(self).state.locked_mutex_count < usize::MAX`. The original code uses `BTreeMap::insert` which has no such bound. While overflow is practically impossible (a thread would need 2^64 mutexes), this is a modeling artifact that does not exist in the original. It is harmless but represents a subtle gap.
  **Suggested Fix:** Document this as a modeling limitation in the verification model section. No code change needed.

- **Location:** `join_cond` omission (exec/spec)
  **Description:** `join_cond(&self) -> Condvar` is entirely omitted from the verification. While Condvar is an opaque sync primitive, the property that `join_cond()` returns a consistent condvar (same condvar across calls for the same thread) could be relevant for liveness arguments about thread joining.
  **Suggested Fix:** If thread-join correctness is ever verified, add a ghost condvar token to ThreadState with a spec-level identity. Low priority since the sync subsystem is out of scope.

- **Location:** `from_state` visibility (exec, running.rs:255)
  **Description:** The original `from_state` is `pub(super)` (crate-local to the thread module), while the verified version is `pub`. This is a minor visibility discrepancy due to Verus proof needs.
  **Suggested Fix:** No action needed; documented as Verus ergonomic requirement.

## Positive Observations

- **Comprehensive function coverage:** All 10 original `RunningThread` methods are accounted for — 8 fully verified with contracts, 1 marked `#[verifier::external]` with documented trust obligations, and 1 (`join_cond`) intentionally omitted with clear justification.
- **Strong state transition properties:** The core thread lifecycle (Running → Sleeping/Ready/Zombie) is verified with identity preservation, well-formedness preservation, mutex accounting preservation, and drop-safety tracking across every transition.
- **Mutex accounting is well-modeled:** The protocol-only model using `Ghost<Set<int>>` + runtime counter with a `wf()` invariant tying them together is a solid abstraction of `BTreeMap`. The per-address frame conditions (`forall|a: int| a != address@ ==> ...`) faithfully capture `BTreeMap::insert`/`remove` non-interference.
- **Proof library is thorough:** The 30+ lemmas in `running.proof.rs` provide good compositional coverage: per-transition identity/wf/mutex/drop-safety lemmas, composite lemmas (from_state_then_schedule, from_state_then_exit), and the acquire-then-release roundtrip lemma demonstrating inverse relationships.
- **Trust boundary is explicitly delineated:** Every omission, abstraction, and strengthening is documented with rationale. The `CROSS-MODULE-CHECK` annotations provide clear integration obligations.
- **Clean three-file split:** Spec, proof, and exec are cleanly separated. The spec file contains only view types and spec functions. The proof file contains only lemmas. The exec file contains implementations with contracts.
- **No unsound shortcuts:** Zero `assume` statements, zero `external_body` annotations. The single `#[verifier::external]` is justified by a Verus language limitation and is thoroughly documented.

## Summary

This is a high-quality verification of the `RunningThread` module. The core correctness properties — state transition safety, identity immutability, mutex accounting consistency, and drop safety — are all proven. The abstraction choices (eliding Box, raw pointers, Condvar, MutexGuard payloads) are appropriate for the verification scope and well-documented.

The main areas for improvement are: (1) the `take_mutex_guard` precondition strengthening, which converts a runtime `Option` check into a proof obligation without a bridging lemma showing the two are equivalent under the expected protocol; (2) the `thread_state_mut` escape hatch, which is a necessary Verus limitation but leaves mutation of ThreadState unverified; and (3) the absence of ghost context tokens for the omitted `*mut ContextInformation` return values, which means the pairing of context pointer to thread identity is not modeled.

The boundary models for sibling thread types are a pragmatic approach, but they create a cross-module consistency obligation that should be discharged when those modules are independently verified. Overall, the verification captures the essential correctness properties of a running thread's state management protocol with appropriate trust boundaries.
