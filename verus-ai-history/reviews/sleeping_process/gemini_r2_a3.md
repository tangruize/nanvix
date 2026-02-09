# Review: sleeping_process (gemini-3-pro-preview)

## Grade: B-

## Issues Found

### High
- **Implementation Logic Unverified (Model-Only Verification) [UNFIXED]**: The verified code remains a ghost model. The `SleepingProcess` struct uses `Ghost<Seq<int>>` instead of the `NonEmptyVecDeque<SleepingThread>` found in the original source. Consequently, the actual execution logic (list manipulation, pushing, popping, memory allocation) is **not verified**. The verification proves that *if* one implements a process adhering to this state machine, it is safe, but it does not prove the actual implementation does so.
- **`wakeup_alarm` Partitioning Logic Unverified [UNFIXED]**: The `wakeup_alarm` function relies on oracle parameters (`has_expired`, `interrupted_ids`, `remaining_ids`) to determine the result of the partition. The actual logic from the original source—iterating through threads and comparing `now >= alarm`—is completely absent in the verified version. The correctness of the partition is assumed, not proven.

### Low
- **Unverified PID Immutability [UNFIXED]**: `state_mut` is `external_body` and assumes PID preservation, which is not enforced on the underlying `ProcessState`.
- **Divergence from Source [UNFIXED]**: The verified file is structurally different from the source (fields are different, logic is erased), making it impossible to use the verified code as a drop-in replacement for the original.

## Positive Observations
- **Consistent Model**: The abstract model itself is consistent and well-specified. The invariants regarding thread uniqueness and disjointness are rigorously proven within the model.
- **Documentation**: The documentation honestly admits the "Verification Model" simplifies the types.

## Summary
The prover has **not addressed** the issues raised in the previous review. The files submitted for re-review are identical to the previous version. The verification status remains "Model-Only". While the model is high-quality, it leaves a significant gap between the verified artifacts and the executable kernel code. The complex logic of managing the `VecDeque` and the time-based partitioning in `wakeup_alarm` remains entirely outside the scope of this verification.

To improve the grade, the verification must:
1.  Model the `NonEmptyVecDeque` (or a verified wrapper around it) so that list operations are verified.
2.  Implement the `wakeup_alarm` logic (loop and comparison) in the verified code instead of relying on oracles.
