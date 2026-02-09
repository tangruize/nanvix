# Review: interrupted_process (gemini-3-pro-preview)

## Grade: A-

## Issues Found

### High
- **Verification is Model-Only (Ghost Code)**: The verified file `verus/split/kernel/pm/process/state/interrupted.rs` defines a ghost-only model of `InterruptedProcess` (fields are `Ghost<Seq<int>>`, functions return `Ghost<...>`). It does not verify the actual executable Rust code (`src/kernel/src/pm/process/state/interrupted.rs`).
    - **Implication**: The verification proves the correctness of the *design* and *state transitions*, but provides no guarantees that the implementation faithfully executes this design (e.g., correct `VecDeque` method calls, memory safety, or absence of panics in the actual code). The semantic equivalence is assumed, not proven.
    - **Suggestion**: Explicitly document this as a "Design Verification" rather than "Implementation Verification". In the future, verify the executable code by using `external_body` for the standard library containers and `exec` functions for the logic.

### Medium
- **Unverified Search Logic (Refinement Assumption)**: The `find_thread` function relies on `lemma_find_thread_refinement_assumption`, which assumes the executable linear search (interrupted → sleeping → zombie) matches the spec.
    - **Implication**: A bug in the implementation's search order or predicate (e.g., searching zombie before sleeping) would not be caught.
    - **Suggestion**: This is a known Verus limitation (iterators/references). The current mitigation (explicit assumption + integration obligation) is acceptable, but leaves a gap.

### Low
- **ProcessState Abstraction Gap**: The model abstracts `Box<ProcessState>` to a `Ghost<int>` (PID).
    - **Description**: There is no verification that the `ProcessState` actually contains the PID claimed by the ghost field. The `state_mut` function in the verified model preserves the PID, but the actual code allows mutation of the `ProcessState` contents.
    - **Suggestion**: Add an invariant that links the ghost PID to the `ProcessState`'s internal PID field if `ProcessState` becomes verifiable.

## Positive Observations
- **Strong Trust Boundary Management**: The module explicitly defines integration obligations (`spec_resume_reason_integration_obligation`, `spec_find_thread_integration_obligation`) for properties that cannot be verified locally (e.g., per-thread state mutation, reference returns). This is excellent modular verification practice.
- **Correct State Machine Logic**: The `resume` function correctly models the transition of a thread from interrupted to ready, preserving all other threads and invariants.
- **Sound Handling of Optional Lists**: The verification correctly models the behavior of `NonEmptyVecDeque` becoming empty (returning `None`), matching the type safety of the original code.
- **Admission Time Oracle**: The use of an oracle parameter for `admission_time` in `resume` correctly handles the side-effect of reading the clock, with a wrapper (`resume_with_valid_clock`) to enforce validity at the call site.

## Summary
The verification of `interrupted_process` is of high quality, providing a rigorous proof of the state machine's logical correctness. It successfully proves that process identity is preserved, threads are not lost or duplicated, and state transitions satisfy well-formedness invariants. The primary limitation is that it verifies a ghost model rather than the executable code itself, relying on structural equivalence assumptions. However, within the scope of verifying the kernel's process lifecycle logic, this is a robust and well-structured artifact.
