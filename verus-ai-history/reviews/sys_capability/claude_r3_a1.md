# Review: sys_capability (claude-opus-4.6)

## Grade: A

## Issues Found

### Critical

None.

### High

None.

### Medium

1. **CapabilityView struct is unused beyond View impl**
   - **Location**: `capability.spec.rs:15-18` (spec)
   - **Description**: `CapabilityView` is defined with a public `value` field and a `View` impl, but no spec function, proof lemma, or downstream module actually consumes it. The `lemma_view_equality` proof references `a@` and `b@` but the view is just a wrapper around `spec_discriminant()`, which is already available directly. If no downstream module depends on this view type, it is dead specification weight.
   - **Suggested Fix**: If downstream verified modules (e.g., process table, capability manager) will use `CapabilityView` for composability, this is fine — add a comment noting which modules will consume it. Otherwise, consider removing it to keep the spec minimal.

2. **Public fields on `CapabilityView` and `Error`**
   - **Location**: `capability.spec.rs:17` (spec), `libs/error/lib.rs:31-32`
   - **Description**: The project coding standards state "Member fields in structs must be private and accessed via getter/setter methods." Both `CapabilityView.value` and `Error.code`/`Error.reason` have public fields. For `CapabilityView`, this is a spec-only type where Verus reasoning requires field access, so it is somewhat justified. For `Error`, the ensures clause in the exec `try_from_u32` directly accesses `result->Err_0.code` and `result->Err_0.reason`, which depends on fields being public.
   - **Suggested Fix**: Document that public fields on spec/proof-only structs are a deliberate Verus convention. For `Error`, this is a shared dependency and out of scope for this module, but worth noting.

### Low

1. **Additional derives (`PartialEq`, `Eq`) change the original public API surface**
   - **Location**: `capability.rs:58` (exec)
   - **Description**: The original `Capability` enum derives only `Debug, Clone, Copy`. The verified version adds `PartialEq, Eq`. While documented in the module header and necessary for Verus equality reasoning, this widens the public API surface (users can now compare capabilities with `==`). This is unlikely to cause issues — it's a natural trait for an enum — but it is a behavioral difference.
   - **Suggested Fix**: Already documented in the header comments. No action needed.

2. **`PARSE_ERROR_MESSAGE` constant is new public API**
   - **Location**: `capability.rs:78` (exec)
   - **Description**: The original uses an inline string literal `"invalid capability"`. The verified version introduces a `pub const PARSE_ERROR_MESSAGE`. This is a minor API surface addition, but it enables the post-condition to reference the error message symbolically, which is good practice.
   - **Suggested Fix**: None needed. This is a good verification pattern.

3. **`to_u32` auxiliary function is new public API**
   - **Location**: `capability.rs:127-139` (exec)
   - **Description**: `to_u32` is a verification auxiliary not present in the original. It is marked `pub` and documented as not in the original source. This is clean and well-handled.
   - **Suggested Fix**: None needed. Already properly documented.

## Positive Observations

- **Full verification with zero errors**: 14 properties verified, 0 errors, 0 assumptions, 0 external bodies in the core module. This is a clean trust boundary.
- **Comprehensive specifications**: The postconditions on `try_from_u32` fully characterize both success and error paths, including the specific error code and message. The spec is neither too weak (it pins the exact variant and discriminant) nor too strong (it doesn't over-constrain internal representation).
- **Strong proof coverage**: The proof file establishes 9 lemmas covering well-formedness, uniqueness, disjointness, round-trip conversion, inverse relationships, view equality, bounds, and total coverage. This is thorough for a simple enum.
- **Clean spec/proof/exec separation**: The three-file split is well-organized. Spec functions are `open spec fn` (transparent to callers), proofs are isolated in the proof file, and exec code is straightforward.
- **Faithful semantic equivalence**: The verified `try_from_u32` matches the original `try_from` exactly in control flow and error behavior. The `TryFrom` trait impl correctly delegates to `try_from_u32`.
- **No soundness holes**: No `assume`, `external_body`, or `trusted` annotations in any of the three capability files. The only `external_body` usage is in the shared `Error` module for logging functions, which is appropriate.
- **Excellent documentation**: The module header clearly documents what properties are verified, what additions were made for verification, and the trust boundary.
- **Deterministic error path**: The error ensures clause pins both `ErrorCode::InvalidArgument` and the specific reason string, preventing silent behavioral drift.

## Summary

This is a high-quality verification of a simple but foundational kernel type. The original module defines a 5-variant enum with a `TryFrom<u32>` conversion — the verified version faithfully reproduces this logic and adds comprehensive specifications and proofs covering well-formedness, discriminant uniqueness, round-trip correctness, and error behavior.

The verification is sound (no assumptions or external bodies), complete (all original functions covered plus the trait impl), and the specs are well-calibrated — strong enough to catch real bugs (wrong discriminant mapping, missing variant, wrong error code) without being brittle.

The only substantive observation is that `CapabilityView` may be premature abstraction if no downstream module consumes it, but this is a minor concern that doesn't affect correctness. The additional derives and helper functions are well-documented and justified.

**Recommendation**: This module is ready for integration. No changes required.
