# Review: running_thread (gemini_r3_a2)

## Grade: A-

## Status
*   **Passed**: YES
*   **Previous Issues Fixed**: Mostly (via documentation/mitigation)

## Analysis of Previous Issues

### 1. Unverified Mutable Access (`thread_state_mut`)
*   **Status**: Mitigated / Acknowledged
*   **Analysis**: The function remains `#[verifier::external]`, leaving a hole in the verification boundary. However, the prover has:
    1.  Clearly documented this as a "Trust Boundary" and listed the manual verification obligations for callers.
    2.  Provided/exposed verified alternatives (`put_mutex_guard`, `take_mutex_guard`) for the most common mutation (mutex accounting), enabling the "replace raw state mutation" suggestion from the previous review.
*   **Verdict**: Acceptable given tool limitations (`&mut T` return). The risk is now managed, though not eliminated.

### 2. Missing API Member (`join_cond`)
*   **Status**: Won't Fix (Documented)
*   **Analysis**: The prover explicitly documented this as omitted/elided.
*   **Verdict**: Acceptable scope limitation. Since the current verification focus is on the state machine transitions and mutex safety, synchronization primitives like `Condvar` can be treated as opaque/external.

### 3. API Divergence (Mutex Release)
*   **Status**: Justified
*   **Analysis**: The prover added specific "Modeling Notes" and "Trust Assumptions" (T2) explaining that the verified model intentionally strengthens the API to forbid releasing unheld mutexes.
*   **Verdict**: Verified. The discrepancy is now an explicit part of the specification contract.

## New Observations

### 1. Cross-Module Verification Obligations
The module uses "boundary models" (`SleepingThread`, `ReadyThread`, `ZombieThread`) defined locally to break cyclic dependencies.
*   **Observation**: This is a robust pattern for decoupling, but it introduces a maintenance risk. The `CROSS-MODULE-CHECK` comments are critical.
*   **Suggestion**: Ensure there is a mechanism (CI check, manual process) to verify that the real definitions in `sleeping.rs` etc. actually satisfy the assumptions made in `running.rs`.

### 2. ZombieThread Exit Safety
The `exit` function correctly mirrors the implementation by *not* requiring `spec_drop_safe()`. The model allows a thread to exit while holding mutexes, transitioning to a `ZombieThread` that (presumably) holds those mutexes. This is a faithful model of a potential kernel panic/error state rather than an idealized one.

## Summary
The verification of `running_thread` is sound within its documented boundaries. The prover has effectively addressed the previous concerns by explicitly defining the trust boundaries and assumptions. The code is well-structured, and the proofs (though mostly automated/empty bodies) cover the defined properties. The remaining "issue" (external mutable access) is a documented limitation of the environment.
