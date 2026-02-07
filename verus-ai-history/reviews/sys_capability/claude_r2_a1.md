# Review: sys_capability (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

None.

### High

None.

### Medium

- **Location:** `spec_from_discriminant` (spec file, line 52–66)
  **Description:** When the input value is outside the valid range (not 0–4), the function silently defaults to `Capability::ProcessManagement`. Although a `recommends` clause signals the intended domain, `recommends` is not enforced. If this spec function is ever used outside contexts where `spec_is_valid_discriminant` has been established (e.g., in a future downstream proof), the arbitrary default could mask a logic error.
  **Suggested Fix:** Either add an explicit `requires Self::spec_is_valid_discriminant(value)` precondition, or use an `arbitrary()` return for out-of-range inputs to make the unspecified behavior explicit.

### Low

- **Location:** `to_u32` (exec file, line 111–123)
  **Description:** The function `to_u32` does not exist in the original source (`src/libs/sys/src/sys/pm/capability.rs`). The original kernel code relies on Rust's `as u8` enum discriminant cast (in `Capabilities::set/clear/has`). While adding `to_u32` is reasonable for enabling round-trip proofs, it extends the API surface beyond the original source.
  **Suggested Fix:** Document in the exec file header that `to_u32` is an auxiliary function added for verification purposes only. Alternatively, annotate it to indicate it is not part of the original API.

- **Location:** `CapabilityView` (spec file, line 14–18)
  **Description:** The `CapabilityView` struct and `View` trait implementation are defined but underutilized. The proof lemmas primarily operate directly on `spec_discriminant` rather than through the view abstraction. Only `lemma_view_equality` uses the `@` operator. The view adds little value if proofs don't consistently leverage it.
  **Suggested Fix:** Either remove `CapabilityView` if view-based reasoning is not needed, or refactor the proof lemmas to use `@` consistently so the view abstraction provides genuine value (e.g., for composability with larger verified modules that reason about capability sets via views).

- **Location:** `Capability` enum (exec file, line 47)
  **Description:** The verified version derives `PartialEq, Eq` while the original only derives `Debug, Clone, Copy`. This is needed for proof-level equality reasoning (`*a == *b`), but subtly expands the trait surface. If the original intentionally omitted `PartialEq`/`Eq`, this could be a semantic divergence.
  **Suggested Fix:** Confirm that the original codebase does not intentionally avoid `PartialEq`/`Eq` on `Capability`. If the original is simply missing them, this is a non-issue. If intentional, guard the derives with `#[cfg(verus)]` or equivalent.

- **Location:** Missing `#[repr]` annotation (exec file)
  **Description:** The original enum and the kernel's `Capabilities` struct use `capability as u8` for bitfield operations, implying the discriminant layout matters. The verified code's `to_u32`/`try_from_u32` manually define the mapping, but there is no `#[repr(u32)]` annotation to guarantee the Rust compiler's discriminant assignment matches. The specs are self-consistent, but the absence of `#[repr]` means the verified discriminant values are assumptions about compiler behavior, not enforced guarantees.
  **Suggested Fix:** This is an inherent limitation of verifying enum discriminants without `repr`. Document this assumption explicitly in the trust boundary section.

## Positive Observations

- **Full verification with no trusted code.** The module contains zero `assume`, `external_body`, or `trusted` annotations. All 14 verification conditions pass cleanly. The Error module's `external_body` usage is limited to logging functions and does not affect correctness reasoning.

- **Comprehensive postconditions.** Both `try_from_u32` and the `TryFrom<u32>` trait implementation carry detailed postconditions covering the success path (valid discriminant, correct variant, matches `spec_from_discriminant`) and the error path (invalid discriminant, correct error code and message). This is thorough for a conversion function.

- **Strong proof coverage.** The proof file includes 8 lemmas covering: well-formedness of all variants, discriminant uniqueness, discriminant disjointness (injectivity), round-trip properties, left-inverse property, view equality, discriminant bounds, and total coverage. This is more than sufficient for the type's complexity.

- **Clean spec/proof/exec separation.** The three-file split is well-organized: specs define the abstract model, proofs establish structural properties, and exec code carries the verified implementation with postconditions. The `include!` pattern keeps the files properly integrated.

- **Faithful semantic equivalence.** The `TryFrom<u32>` implementation correctly delegates to `try_from_u32` and carries identical postconditions. The match arms in `try_from_u32` are identical to the original source. The error path constructs the same `ErrorCode::InvalidArgument` with the same message string.

- **Good documentation.** The exec file header clearly lists the verified properties and trust boundary, making the verification scope immediately apparent.

## Summary

The Verus verification of `sys_capability` is thorough and sound for a simple enum conversion module. All functions from the original source are covered, with the `TryFrom<u32>` trait faithfully verified through delegation to the fully specified `try_from_u32`. The proof lemmas establish all essential structural properties (uniqueness, injectivity, round-trip, bounds, coverage) without any trusted assumptions.

The primary medium-priority concern is the `spec_from_discriminant` default case, which could silently return `ProcessManagement` for out-of-range inputs if used outside its intended context. The low-priority issues are cosmetic: the added `to_u32` function, underutilized `CapabilityView`, and implicit reliance on compiler discriminant layout.

Overall, this is a clean, well-structured verification that appropriately matches the simplicity of the underlying type. The grade reflects the high quality of the work with minor deductions for the spec default-case concern and the gap between the view abstraction's promise and its actual utilization.
