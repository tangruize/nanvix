# Review: zombie_process (gemini-3-pro-preview)

## Grade: A

## Issues Found

### Low
- **Unverified Search Logic:** `find_thread()` and `find_thread_mut()` are marked `external_body` with postconditions that match the spec. The actual executable implementation (linear search via `iter().find()`) is not verified against the spec model. This is acceptable for design verification but leaves a gap where the code could theoretically diverge from the spec.
- **Cross-Module Trust:** `state_mut()` relies on the `process_state` module to preserve PID (`spec_pid() == old(self).spec_pid()`). While this is documented and claimed to be verified in the dependency, it creates a validity dependency that isn't checked by the compiler in this module.

## Positive Observations
- **Explicit Scope:** The documentation clearly defines this as a "Design Verification" (ghost model) and explicitly lists what is and isn't verified. This transparency is excellent.
- **Sound Abstraction:** The modeling of `NonEmptyVecDeque` as `Seq<int>` (ghost) with a parallel `zombie_count` (exec) correctly bridges the gap between the complex kernel type and the verifiable model.
- **Invariant Modeling:** The `wf()` predicate correctly captures the uniqueness invariant of thread IDs, explicitly justifying why it holds (ownership model) despite not being runtime-checked.
- **Immutability Safety:** The verification relies on the immutability of Thread IDs. Inspection of `ThreadState` confirms that `id` is private with no setters, making the `find_thread_mut` safety obligations (that callers must not change ID) structurally sound in the actual codebase.

## Summary
The verification of `zombie_process` is a high-quality design verification. It successfully proves that the state transitions of a zombie process (creation, query, burial) preserve the essential identity properties (PID, thread list, exit status). The use of `external_body` is well-justified and documented, with clear "integration obligations" that define the contract with the executable code. The split between exec, spec, and proof code is clean and idiomatic.
