# Review: process_state (gemini-3-pro-preview) - Round 2

## Grade: B

## Issues Found

### High
- **Verification still targets a model, not the implementation**: The verified code remains a protocol model using `ghost_mutexes` (Map) and `mutex_count` (usize) instead of verifying the actual `BTreeMap` and `LinkedList` containers. While the documentation now explicitly acknowledges this ("This is a protocol-level verification model, not a drop-in replacement"), the issue persists: this verification does not guarantee the correctness of the actual kernel code that will run in production, which relies on complex data structures that are abstracted away here. The struct definition is still ABI-incompatible.

### Medium
- **Missing functional specifications for stubbed methods**: Methods like `add_event`, `add_mmio`, etc., are still stubbed with `external_body` and only have frame conditions (`ensures true` or just preserving other fields). They do not specify what the function actually *does* (e.g., that an event is added). This limits the verification's utility for any code that depends on these side effects.
- **ABI Incompatibility**: The struct definition in the verified file still lacks the actual storage fields (`vmem`, `events`, `mailbox`, `mmio`, `mutexes`, `conditions`, `pmio`) and uses ghost fields instead. It cannot be used as a replacement for the original code.

### Low
- **Stub Naming Convention**: `_stub` suffixes are still present.
- **Missing Debug Implementation**: Debug impl is still a stub.

## Positive Observations
- **Transparency**: The updated documentation (lines 36-127) is excellent. It clearly states "This is a protocol-level verification model, not a drop-in replacement" and lists "Trust Assumptions" explicitly. This honesty upgrades the review from "misleading verification" to "valid verification of a model".
- **Soundness**: The verification passes and the logic within the model appears sound. The usage of oracle parameters (`already_present`, `found_idx`) to bridge the ghost/exec gap is a valid technique for this kind of model.

## Summary
The prover has effectively documented that this is a **model verification**, not an implementation verification. They have not changed the code to verified implementation (which would be a much larger task requiring verified BTreeMap/LinkedList), but they have added clear warnings and explanations.

The verification is **sound regarding the protocol logic** (limits, ref-counting, PID immutability) but **does not verify the implementation** of the containers or the memory safety of the actual kernel structures.

The grade remains a **B** because while the verification is high-quality for a model, the initial request was to "Review the Verus verification of process_state... Evaluate if the verification captures the essential correctness properties." Since it verifies a model and not the actual code, it misses the property of "implementation correctness." However, as a protocol verification, it is valuable.
