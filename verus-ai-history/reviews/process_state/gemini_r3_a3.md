# Review: process_state (gemini-3-pro-preview)

## Grade: B-

## Issues Found

### High
- **Abstraction Gap (Storage vs. Accounting):** This issue remains **UNFIXED** in the third iteration. The verified code models the *logic* of resource limits and reference counting but abstracts away the actual *storage* (BTreeMap, Condvar, Mutex). The verified struct uses counters and ghost maps instead of the real kernel types. This means the verification does not prove that objects are actually stored or retrieved correctly, only that the counters and ghost state would behave correctly if they were updated.
- **Function Signature Mismatch:** This issue remains **UNFIXED**. Functions like `get_mutex` and `get_cond` still use ghost parameters (`Ghost<int>`) and oracle booleans (`already_present: bool`) in their signatures, and return ghost values (`Ghost<nat>`) instead of actual objects. This makes the verified module ABI-incompatible with the rest of the kernel. It cannot be used as a drop-in replacement.

### Medium
- **Missing Data Flow in `remove_pmio`:** This issue remains **UNFIXED**. The function still returns `Result<(), Error>` instead of `Result<AnyIoPort, Error>`. The functional behavior of retrieving the removed port is not verified.
- **Dependency on Caller Truthfulness:** This issue remains **UNFIXED**. The correctness of the model relies on the caller providing correct oracle values (e.g., `already_present` must match the ghost state). Since the real storage is absent from the verified module, there is no way to enforce this link within the module itself.

### Low
- **Missing `vmem_mut` / `get_pmio`:** These helpers are still missing/abstracted away.

## Positive Observations
- **Sound Protocol Verification:** The verification of the accounting protocol itself is sound and correct. The specs and proofs correctly handle the logic of capacity limits and reference counting thresholds.
- **Clean Separation:** The code is well-organized into exec/spec/proof files.

## Summary
The prover has submitted the **same code** for three consecutive rounds, addressing none of the issues raised regarding the abstraction gap and ABI mismatch. The verification remains a high-quality "protocol model" (verifying the logic of counters and limits) but fails to verify the actual implementation of `ProcessState` (storage/retrieval of kernel objects). It is not a functional replacement for the original code.
