# Review: process_state (gemini-3-pro-preview)

## Grade: B

## Issues Found

### High
- **Verification targets a model, not the implementation**: The verified `ProcessState` struct differs significantly from the original. It replaces runtime containers (`BTreeMap`, `LinkedList`) and fields (`Vmem`, `Mailbox`) with `Ghost` types and counters. Methods like `get_mutex` return `Ghost<nat>` instead of `Mutex` objects. This means the verification proves the correctness of the abstract *protocol* (ref counting, limits) but not the actual implementation. The verified code is not a drop-in replacement and does not verify the memory safety or functional correctness of the actual kernel code.

### Medium
- **Missing functional specifications for stubbed methods**: Methods such as `add_event`, `add_mmio`, `post_message`, and `copy_from_user_unaligned` are stubbed with `external_body`. Their specifications only assert frame conditions (that they do not modify verified fields like `pid` or `mutex_count`). The specifications do not capture the positive functional behavior (e.g., that an event is actually added to the list), leaving these operations largely unverified.
- **ABI Incompatibility**: The `ProcessState` struct definition in the verified code lacks the actual storage fields (`vmem`, `events`, `mailbox`, `mmio`, `mutexes`, `conditions`, `pmio`). Any attempt to use this struct in the kernel would result in data loss and ABI mismatch, as it only contains the `pid`, `capabilities`, and two `usize` counters at runtime.

### Low
- **Stub Naming Convention**: Stubbed methods use a `_stub` suffix (e.g., `add_event_stub` vs `add_event`), creating a naming mismatch with the original source. This requires manual bridging if the code were ever to be integrated.
- **Missing Debug Implementation**: The `Debug` trait implementation is stubbed, so the formatting logic is unverified.

## Positive Observations
- **Strong Protocol Verification**: The verification rigorously proves the correctness of the resource management protocol, including capacity limits (`MUTEX_MAX`, `COND_MAX`), reference counting logic (modeling `Arc::strong_count`), and PID immutability.
- **Excellent Documentation**: The "Trust Assumptions" section clearly articulates what is trusted (BTreeMap semantics, Arc behavior) vs what is verified.
- **Clean Separation**: The separation of execution code (model), specifications, and proofs into `.rs`, `.spec.rs`, and `.proof.rs` is well-structured and readable.
- **Invariants**: The `wf()` predicate correctly captures the relationship between runtime counters and ghost map sizes, ensuring consistency.

## Summary
The verification of `process_state` is a high-quality **formal model** of the process state management logic, specifically focusing on resource limits and reference counting protocols for mutexes and condition variables. However, it is **not** a verification of the actual implementation. The verified code replaces real data structures with ghost models and counters, rendering it unusable as a functional kernel component. While it provides high confidence in the design of the state management protocol, it leaves the actual execution code (container operations, memory management, opaque field handling) unverified. To achieve implementation verification, the model would need to be refined to use the actual `BTreeMap` and `LinkedList` types (or verified wrappers around them) and verify the `Mutex`/`Condvar` object lifecycles directly.
