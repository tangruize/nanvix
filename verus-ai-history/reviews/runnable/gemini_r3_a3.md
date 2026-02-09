# Review: runnable (gemini-3-pro-preview)

## Grade: C

## Issues Found

### Critical
- **Ghost Implementation Persists (Unresolved)**
  - **Location**: `RunnableProcess` struct and all functions.
  - **Description**: The "fix" did not address the fundamental issue that the verified code is a ghost model, not the actual implementation. The `RunnableProcess` struct still uses `Ghost<Seq<int>>` instead of concrete thread collections (`NonEmptyVecDeque`). The `RunningProcess` return type is still entirely ghost.
  - **Verification**: Reviewed `runnable.rs`. Struct fields are still `Ghost<...>` (lines 123-131). Functions like `new`, `run`, and `terminate` operate almost exclusively on ghost state.
  - **Impact**: The verification proves the correctness of the *abstract state machine protocol*, but completely bypasses memory safety, data structure integrity, and runtime behavior of the actual kernel code. It cannot replace the original implementation.

### High
- **Missing Exec Functions (Unresolved)**
  - **Location**: `find_thread`, `find_thread_mut`, `earliest_admission_time`.
  - **Description**: These functions are still missing from the verified exec module.
  - **Verification**: Confirmed absence in `runnable.rs`. Only spec versions exist in `runnable.spec.rs`.
  - **Impact**: Functionality present in the original code is missing in the verified version, making it an incomplete replacement.

### Medium
- **Oracle Parameter in `wakeup` (Mitigated but still present)**
  - **Location**: `wakeup` function.
  - **Description**: The `found: bool` parameter remains.
  - **Verification**: Line 498 of `runnable.rs`: `pub fn wakeup(self, tid: Ghost<int>, found: bool)`.
  - **Assessment**: The documentation (lines 481-490) argues this is necessary because the search target (sleeping list) is ghost. This confirms the critical issue: because the state is ghost, runtime operations like searching must be faked with oracles.

### Low
- **Implicit Return Types (Persists)**
  - **Location**: `run` function.
  - **Description**: `interrupt_reason` and HAL context info are still abstracted away or ghost.
  - **Verification**: `RunningProcess` struct (lines 146-163) uses `Ghost<int>` for `interrupt_reason` and omits context info.

## Positive Observations
- The documentation has been improved to clearly state the "Verification Model" and "Trust Boundary" (lines 30-77 of `runnable.rs`), explicitly acknowledging that thread lists are modeled as ghost sequences. This transparency is good, but it confirms the verification is model-only.

## Summary
The prover has **not** addressed the core critical issue: the verification is of an abstract ghost model, not the concrete implementation. The module successfully verifies the *logic* of the state transitions (protocol correctness), but it substitutes actual data structures with ghost sequences. This means the verified code cannot execute or replace the original kernel code. The grade remains **C** because while the model is internally consistent and verified, it fails the "Equivalence" and "Coverage" criteria for a verifying the *implementation*.

To advance, the prover must implement the concrete `NonEmptyVecDeque` and thread types in Verus, or explicitly scope this as a "spec-only model" and not a "verified implementation".
