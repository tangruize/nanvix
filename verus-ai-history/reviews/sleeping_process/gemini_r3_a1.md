# Review: sleeping_process (gemini-3-pro-preview)

## Grade: A

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- None.

### Low
- **Oracle Dependence**: The `wakeup` and `wakeup_alarm` functions rely on oracle parameters (`found`, `has_expired`, `interrupted_ids`) to determine the operation result, rather than verifying the underlying `VecDeque` search/filter logic or `SystemTime` comparisons. This verifies the *correctness of the state transition* given the search result, but not the *correctness of the search itself*.
- **Model vs. Implementation**: The verified code is an abstract model (using `Ghost<Seq<int>>` instead of `VecDeque` and `Box<ProcessState>`), not the actual executable code with annotations. While appropriate for high-level logic verification, it relies on the unverified assumption that the real data structure operations map correctly to the sequence operations.

## Positive Observations
- **Strong Invariants**: The `wf()` predicate enforces critical structural properties: non-empty sleeping threads, no duplicate thread IDs, and disjointness between sleeping and zombie thread sets.
- **Sophisticated Partitioning Spec**: The `wakeup_alarm` specification uses `spec_is_subsequence` to elegantly model the stable partitioning of threads, ensuring that order is preserved even after splitting the list.
- **Clean Separation**: Specifications, proofs, and executable models are well-organized into separate files, making the verification logic easy to follow.
- **Conservation Laws**: The specs explicitly enforce thread conservation (count and content) across all transitions, preventing threads from being lost or spontaneously created.

## Summary
The verification of `sleeping_process` is excellent. It creates a robust abstract model of the process state machine and proves that all transitions (termination, wakeup, alarm expiration) preserve essential invariants like thread uniqueness and conservation. While it abstracts away the low-level details of collection manipulation and time comparison via oracles, this is a reasonable design choice for verifying high-level kernel logic.
