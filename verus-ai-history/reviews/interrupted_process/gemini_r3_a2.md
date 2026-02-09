# Review: interrupted_process (gemini-pro)

## Grade: B

## Issues Found

### High
- **Gap between Ghost Model and Executable Code:** The verification remains a "design-level" proof. The `InterruptedProcess` struct in `interrupted.rs` contains only `Ghost` fields. The actual executable Rust code (using `VecDeque`, `Box<ProcessState>`) is completely replaced by this shadow model. The "Integration Obligations" (e.g., `spec_find_thread_integration_obligation`) explicitly state that structural equivalence is assumed but not proven. This leaves the actual kernel implementation unverified against the model.

### Medium
- **`find_thread` Search Logic Unverified:** The executable implementation of `find_thread` uses `iter().find(...)` with a specific search order (interrupted -> sleeping -> zombie). The verified model (`spec_find_thread`) replicates this logic in ghost code, but there is no proof that the Rust iterator chain matches the ghost logic. A bug in the Rust search predicate or ordering would not be caught. `lemma_find_thread_refinement_assumption` documents this but does not solve it.
- **`resume` Side Effects Ignored:** The `resume` function in the model moves thread IDs but does not model the `InterruptedThread::resume()` call which modifies the thread's internal state (setting `interrupt_reason`). The `spec_resume_reason_integration_obligation` acknowledges this but the verification of the actual state change is absent from this module.

### Low
- **PID Linking:** The link between `Ghost<int>` PID and `Box<ProcessState>` PID is now formally defined via `spec_process_state_pid_integration_obligation`, but the enforcement relies on correct usage at the unverified construction site.

## Positive Observations
- **Honest Documentation:** The file headers clearly label this as "Design Verification" and explicitly list the "Trust Boundaries" and "Trust Gaps". This is a significant improvement in transparency.
- **Formalized Obligations:** The use of `spec_*_integration_obligation` functions transforms vague assumptions into precise, machine-readable contracts that future integration proofs can target.
- **Clock Validation:** The `resume_with_valid_clock` wrapper correctly formalizes the dependency on the `clock` HAL via an oracle pattern.

## Summary
The verification is sound within the scope of the **ghost model**, successfully proving that the state transition protocol preserves thread identities and invariants. However, it is **not** a verification of the `interrupted_process` executable code. It verifies a specification of that code. The new "Integration Obligations" improve the rigor of the assumptions but do not close the gap between the model and the implementation. The grade reflects high-quality design verification limited by the lack of executable refinement.
