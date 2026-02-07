# Review: sys_capability (claude-opus-4.6)

## Grade: A

## Issues Found

### Critical

None.

### High

None.

### Medium

- **Location:** `spec_from_discriminant` (capability.spec.rs, lines 52–66)
  - **Description:** The catch-all `else` branch maps *any* value ≥ 4 (not just 4) to `Capability::ProcessManagement`. Although a `recommends` clause restricts expected callers to valid discriminants (0..=4), the spec is technically total and silently maps invalid inputs (e.g., 99) to a valid variant. This weakens reasoning if the function is ever used without the precondition—a consumer could accidentally prove `spec_from_discriminant(99) == ProcessManagement`.
  - **Suggested Fix:** Either (a) restructure as a chain of `if/else if` that explicitly matches value == 4 for the last branch with an arbitrary/unreachable default, or (b) use `arbitrary()` for the out-of-range case to make misuse detectable. Example:
    ```rust
    } else if value == 4 {
        Capability::ProcessManagement
    } else {
        arbitrary()
    }
    ```

### Low

- **Location:** `to_u32` (capability.rs, lines 111–123)
  - **Description:** `to_u32()` is a new public method not present in the original source. While it is correctly verified and serves the round-trip proof, it extends the API surface beyond the original. Consumers of the verified module get an interface that doesn't exist in the real implementation.
  - **Suggested Fix:** If the method is only needed for verification, consider making it a `spec` function or documenting it as a verification-only helper. Alternatively, confirm that the original codebase should adopt this method.

- **Location:** `CapabilityView` (capability.spec.rs, lines 14–18)
  - **Description:** The `CapabilityView` struct and `View` trait implementation are defined but not used by any proof lemma or postcondition in this module. All proofs use `spec_discriminant` directly. The view is available for external consumers but adds unused abstraction within the module.
  - **Suggested Fix:** No action required unless downstream modules use it. If unused project-wide, consider removing to reduce complexity.

- **Location:** `PARSE_ERROR_MESSAGE` constant (capability.rs, line 67)
  - **Description:** The original uses an inline string `"invalid capability"` in the error path. The verified code extracts this to a named constant. While semantically equivalent and arguably better style, it is a minor structural divergence from the original.
  - **Suggested Fix:** No fix needed; this is a neutral-to-positive refactor. Just note the divergence for traceability.

## Positive Observations

- **Full verification with zero trust assumptions.** No `assume`, `external_body`, or `trusted` markers appear in the capability module. All 13 verification conditions pass cleanly.
- **Comprehensive proof lemmas.** The proof file establishes 9 distinct properties covering well-formedness, uniqueness, injectivity, round-trip correctness, view consistency, bounds, and total coverage. For a simple enum, this is thorough.
- **Strong postconditions on exec functions.** Both `try_from_u32` and `to_u32` have rich ensures clauses that tie exec behavior back to spec functions. The error path is also fully specified (correct error code and reason string).
- **Clean spec/proof/exec separation.** The `include!` pattern cleanly separates concerns while keeping everything in a single compilation unit. The `TryFrom` trait wrapper is correctly placed outside the `verus!` block and delegates to the verified method.
- **Faithful semantic equivalence.** The exec code matches the original match arms exactly (same discriminant values 0–4, same error code). The `TryFrom<u32>` trait impl delegates directly to the verified `try_from_u32`.
- **Added `PartialEq` and `Eq` derives** enable the disjointness lemma (`lemma_discriminants_disjoint`) and view equality lemma, which are important structural properties for downstream use.

## Summary

The Verus verification of `sys_capability` is excellent for the scope of the module. All original functionality is covered with faithful semantic equivalence. The specifications are precise—postconditions tie exec results to spec-level definitions, and the error path is fully characterized. The proof suite is comprehensive for an enum type, establishing uniqueness, injectivity, round-trip, bounds, and coverage properties.

The only medium-priority issue is the `spec_from_discriminant` catch-all branch, which could silently map out-of-range values to `ProcessManagement`. While the `recommends` clause mitigates this for well-behaved callers, tightening the spec with `arbitrary()` for invalid inputs would make misuse structurally impossible.

Overall, this is a clean, well-structured verification that serves as a good model for other simple enum modules in the codebase.
