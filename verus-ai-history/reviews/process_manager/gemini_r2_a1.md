# Review: process_manager (gemini-3-pro-preview)

## Grade: B+

## Issues Found

### Medium
- **Control Flow Verification (exec/process_manager.rs):** The outer `ProcessManager::terminate` function contains logic that decides whether a process moves to the zombie queue or stays in ready (based on `process.terminate()` result). This control flow is not modeled in a unified `outer_terminate` function. Only the leaf transitions (`terminate_ready`, `terminate_ready_stays_ready`) are verified. A bug in the `mod.rs` `match` block (e.g., dropping a process) would not be caught by the current verification.
- **Incomplete Schedule Model (exec/process_manager.rs):** The `full_schedule` function models `resume_all_interrupted` + `schedule`, but omits `check_alarm` which is part of the original `schedule`. This means the alarm expiry logic (moving suspended processes to interrupted based on time) is not covered by the `full_schedule` model, requiring the caller to manually model it.

### Low
- **PID Overflow Check (exec/process_manager.rs):** The verified `create_process` requires `spec_can_create_process()` (next_pid < i32::MAX) as a precondition. The original `mod.rs` implementation does not check this bound and simply increments the PID, potentially leading to overflow (though unlikely in practice).
- **Implicit Type Equivalence (spec/process_manager.spec.rs):** The verification models the `LinkedList<Process>` queues as `Ghost<Set<int>>`. The validity of this abstraction relies on the Rust affine type system (uniqueness of `Process` handles) to prevent duplicates. This assumption is sound but implicit; there is no formal proof connecting the `LinkedList` implementation to the `Set` model.

## Positive Observations
- **Soundness:** The verification is exceptionally sound. There are no `external_body` or `assume` directives in the verified module, meaning the model itself is internally consistent and rigorous.
- **Invariant Coverage:** The `wf()` predicate comprehensively captures all critical safety properties: queue disjointness, kernel liveness (PID 0 safety), PID monotonicity, and count consistency.
- **Transition Coverage:** Every state transition in the original code (creation, scheduling, sleep, exit, termination, harvest) has a corresponding verified function that proves preservation of the `wf()` invariant.
- **Clean Separation:** The split verification structure is clean and well-organized, with clear boundaries between execution code, specifications, and proofs.

## Summary
The `process_manager` verification provides a high degree of confidence in the correctness of the kernel's process state machine. It successfully proves that valid state transitions preserve system invariants like process uniqueness and kernel liveness. The primary limitation is the "Split Verification" approach, where the verification targets a model (`ProcessManagerInner` with integer fields) rather than the actual implementation (`ProcessManagerInner` with `LinkedList`). While the model logic closely mirrors the implementation, the lack of formal linkage means that bugs in the "glue code" (like control flow in `terminate` or `schedule`) might escape detection. However, the core safety properties of the process management subsystem are well-verified.
