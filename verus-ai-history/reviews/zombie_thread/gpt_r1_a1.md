# Review: zombie_thread (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- **Location:** `ZombieThread::thread_state_mut` (exec, `verus/split/kernel/pm/thread/zombie.rs`)  
  **Description:** Implemented as `#[verifier::external]` with no machine-checked postconditions. This creates a trust hole: callers can mutate `ThreadState` and violate `wf()`, change the thread id, or indirectly affect the semantics of `status()`, contradicting the documented invariants and “id immutability” claims. Any proofs that rely on `wf()` or identity preservation become unsound when this API is used.  
  **Suggested Fix:** Replace with a verified wrapper API that exposes only specific safe mutations with explicit `requires/ensures`, or introduce an `external_body` wrapper with a spec that enforces preservation of `wf()`, `spec_id()`, and `spec_status()`. If Verus cannot express `&mut` returns, consider making this function private and provide verified mutation methods instead.

### Medium
- **Location:** `ZombieThread` fields are `pub` (exec, `verus/split/kernel/pm/thread/zombie.rs`)  
  **Description:** The verified model allows arbitrary construction and direct field mutation, which the original Rust type forbids (private fields). This breaks encapsulation and can bypass the `from_state()` preconditions and invariants, making proofs about well-formedness and identity preservation brittle.  
  **Suggested Fix:** Restrict field visibility (e.g., `pub(crate)`/`pub(super)`) and provide spec accessors for proofs, or add a `wf()`-guarded invariant and make direct construction impossible outside the module.

### Low
- **Location:** `ZombieThread::wf` / `spec_status` (spec, `verus/split/kernel/pm/thread/zombie.spec.rs`)  
  **Description:** `ExitStatus` is modeled as an unconstrained `int` and `wf()` does not bound it to the concrete type’s range. This is a deliberate abstraction, but it weakens equivalence if any downstream code assumes range or validity constraints.  
  **Suggested Fix:** Add an optional predicate (e.g., `spec_status_valid()`) and strengthen `wf()` to encode the `ExitStatus` range when those properties matter.

## Positive Observations
- Full function coverage exists for all original APIs (`from_state`, `id`, `thread_state`, `thread_state_mut`, `harvest`, `status`).
- Specs for `from_state` and `harvest` capture the key resource-transfer behavior (stacks are preserved and returned) and preserve ThreadState invariants.
- Proofs are cleanly separated into `.spec.rs` and `.proof.rs`, with clear documentation of trust boundaries and model simplifications.

## Summary
The verification captures the core safety behavior of constructing and harvesting zombie threads, but relies on a significant external trust boundary (`thread_state_mut`) and weakened encapsulation (public fields). Tightening these two areas would materially improve soundness and equivalence to the original code while keeping the existing proofs mostly intact.
