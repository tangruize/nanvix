# Review: process_state (gemini-3-pro-preview) - Round 3

## Grade: B

## Issues Found

### High
- **Verification targets a model**: The verification is explicitly for a protocol model (`ghost_mutexes` map, `mutex_count` counter) and not the actual implementation (`BTreeMap`, `LinkedList`). This is now well-documented in the code ("This is a protocol-level verification model, not a drop-in replacement"), but it means the verification does not cover memory safety or functional correctness of the actual container operations used in the kernel. The struct remains ABI-incompatible.

### Medium
- **Missing functional specifications for stubbed methods**: Methods like `add_event`, `add_mmio`, `post_message` remain stubbed with `external_body` and only have frame condition specifications (ensuring they don't modify the verified fields). They lack positive functional specifications (e.g., asserting that an event is actually added), limiting the verification's scope to non-interference.
- **ABI Incompatibility**: The struct definition uses ghost fields instead of actual storage, preventing use as a replacement.

### Low
- **Stub Naming Convention**: `_stub` suffixes persist.
- **Missing Debug Implementation**: `Debug` impl is a stub.

## Positive Observations
- **Honest Documentation**: The documentation clearly states the limitations ("protocol-level verification model"), which is crucial for correct interpretation of the results.
- **Sound Protocol Verification**: The logic for the protocol itself (capacity limits, reference counting, PID immutability) is soundly verified within the model.
- **Clean Separation**: Code, specs, and proofs are well-organized.

## Summary
The verification status is unchanged from the previous round. The prover has documented that this is a model verification, which is acceptable given the complexity of verifying `BTreeMap` and `LinkedList` in Verus. The verification successfully proves the correctness of the *protocol* (limits, ref-counting) but does not verify the *implementation*. The grade remains B, reflecting high-quality model verification but acknowledging it is not implementation verification.
