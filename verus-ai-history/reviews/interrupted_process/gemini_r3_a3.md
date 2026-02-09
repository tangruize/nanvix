# Review: interrupted_process (gemini-pro)

## Grade: B

## Issues Found

### High
- **Gap between Ghost Model and Executable Code (PERSISTENT):** The verification remains strictly a "design-level" proof. The `InterruptedProcess` struct and its methods operate entirely on `Ghost` fields. The actual executable Rust code (using `VecDeque`, `Box<ProcessState>`, and standard library iterators) is completely bypassed. The header explicitly states: "Structural equivalence ... is assumed." This means the *implementation* of the OS kernel component is not verified, only a shadow model of it.

### Medium
- **`find_thread` Search Logic Unverified (PERSISTENT):** The `find_thread` function is modeled as a spec function (`spec_find_thread`) that assumes the executable's `iter().find(...)` logic works identically. There is no proof that the Rust iterator chain (checking interrupted, then sleeping, then zombie lists) matches the ghost logic. A bug in the implementation's search order or predicate would go undetected. `lemma_find_thread_refinement_assumption` documents this gap but does not close it.
- **`resume` Side Effects Ignored (PERSISTENT):** The model ignores the side effects in the executable `InterruptedThread::resume()` method (specifically `self.state.set_interrupt_reason(self.reason)`). While `spec_resume_reason_integration_obligation` acknowledges this, the verification of this state change is absent.

### Low
- **PID Linking Enforcement:** The `spec_process_state_pid_integration_obligation` defines the requirement that the ghost PID matches the real PID, but this relies on unverified code at the construction sites (`new`, `from_sleeping`) to satisfy it.

## Positive Observations
- **Projection Lemma Added:** The addition of `lemma_project_to_runnable_boundary` provides a useful bridge for integration proofs, allowing the richer `InterruptedProcess` model (which includes sleeping threads) to be projected onto the simpler `RunnableProcess` boundary model used by the sibling module.
- **Clear Trust Boundaries:** The documentation clearly enumerates the "Trust Gaps" and "Integration Obligations". This transparency allows auditors to understand exactly what is and isn't proven.
- **Sound Design Proof:** Within the scope of the ghost model, the proofs for invariants (uniqueness, disjointness) and state transitions are sound and pass verification.

## Summary
The verification has stabilized as a high-quality **design specification** that is disconnected from the implementation. The prover has added useful integration lemmas (like the projection lemma) but has effectively declined to verify the executable code itself, citing it as "future work". As a result, the grade remains a **B**. The verification proves the *protocol* is correct but provides no guarantee that the *code* implements that protocol correctly.
