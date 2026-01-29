# Review: kredzone (gemini-3-pro-preview)

## Grade: B

## Issues Found

### High
- **Functional Correctness Unverified**: The `store` and `load` functions are marked `#[verifier::external_body]`, meaning their implementation logic is not verified by Verus. The specifications only guarantee that `Result::Ok` is returned if and only if the index is within bounds. There is no postcondition ensuring that `store` actually updates memory or that `load` returns the previously stored value.
    - **Suggested Fix**: While `volatile` operations on `extern C` statics cannot be verified directly, the specification could be strengthened by introducing a ghost variable (e.g., `tracked_static_ghost`) to model the global state. The `external_body` specs could then update this ghost state, allowing callers to verify sequences of operations (e.g., store-then-load) against the model, effectively lifting the trust boundary from "entire logic" to "synchronization of ghost vs concrete".

### Medium
- **Disconnected Abstract Model**: The module includes a rich abstract model (`KernelRedZoneView`) and proves several lemmas about it (idempotence, commutativity). However, this model is completely disconnected from the executable `store` and `load` functions. A caller cannot use these lemmas to prove anything about the actual code execution because the `store`/`load` functions do not operate on or return `KernelRedZoneView` instances.
    - **Suggested Fix**: If modifying the function signature to accept `Tracked<&mut KernelRedZoneGhost>` is not possible due to ABI/usage constraints, consider creating a verified wrapper layer that manages the ghost state and calls the raw `store`/`load` functions. This would allow verified code to use the safe wrapper while keeping the raw API available for legacy/C callers.

### Low
- **Hardcoded `ENTRY_SIZE`**: The use of `cfg` attributes to define `ENTRY_SIZE` (4 or 8) and the associated lemmas is a workaround for Verus's inability to reason about `mem::size_of` in const contexts. While correct for standard x86/x64, it adds maintenance burden if new architectures are added.
    - **Suggested Fix**: No immediate fix available in Verus, but this should be monitored. The current `lemma_entry_size_matches_target` approach is a reasonable mitigation.

## Positive Observations
- **Clear Trust Boundaries**: The documentation explicitly details why `external_body` is used (volatile/extern C) and lists the specific trust assumptions (T1, T2, T3). This makes the limitations transparent to auditors.
- **Robust Bounds Checking**: The verification successfully proves that the bounds checks are correct and sufficient to prevent out-of-bounds access, which is the primary safety concern for this component.
- **Rich Abstract Properties**: Although disconnected, the `KernelRedZoneView` proofs demonstrate that the design is sound in theory (e.g., proving non-interference between indices).
- **100% Coverage**: All public API functions and constants are covered by the verification module.

## Summary
The `kredzone` verification module achieves a solid **B** grade. It successfully verifies the safety-critical bounds checking logic, ensuring that invalid indices cannot trigger unsafe memory access. However, it stops short of verifying functional correctness (that values are correctly stored and retrieved) due to the challenges of modeling `extern C` and `volatile` memory in Verus. The reliance on `external_body` for the main logic means the verification is essentially a contract check for the API surface, rather than a full proof of implementation correctness. The abstract model provided is high-quality but currently serves only as documentation/specification rather than a tool for verifying client code.
