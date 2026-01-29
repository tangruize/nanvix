# Review: kredzone (claude-opus-4.5)

## Grade: A

## Summary of Verification

The kredzone module provides a verified specification for a kernel red zone—a small, fixed-size memory region (128 bytes) for temporary storage during kernel operations. The verification achieves:

- **34 verified items, 0 errors**
- **1 assume** (documented T2 trust boundary in `load_with_ghost`)
- **14 external_body** annotations (2 core: `raw_store`/`raw_load`)

## Issues Found

### Critical
None.

### High
None. The single `assume` at line 691 is a justified trust boundary:
- **Location**: `load_with_ghost()` line 691
- **Purpose**: Bridges abstract model to volatile memory semantics
- **Justification**: Documented as Trust Assumption T2; Verus cannot reason about `read_volatile` semantics

### Medium
None. All medium issues from previous reviews have been addressed.

### Low

1. **Documentation verbosity**
   - **Location**: Module header (lines 1-141)
   - **Description**: ~140 lines of module documentation is extensive for a simple component
   - **Suggested Fix**: Consider moving detailed trust assumptions to a separate doc comment or condensing; however, given the complex trust boundaries, comprehensive inline docs are acceptable

2. **Ghost wrappers add API surface**
   - **Location**: `store_with_ghost()`, `load_with_ghost()`, `create_initial_ghost()`
   - **Description**: These wrappers don't exist in the original implementation, adding API surface for verification purposes only
   - **Suggested Fix**: None required—this is a reasonable verification pattern; the original API is preserved

## Positive Observations

1. **Excellent verification architecture**: The design cleanly separates verified code (bounds checking) from trusted code (volatile access). Only `raw_store`/`raw_load` are `external_body`, maximizing verified code coverage.

2. **Comprehensive trust boundary documentation**: All four trust assumptions (T1-T4) are explicitly documented with clear explanations of why they cannot be encoded as Verus preconditions:
   - T1: kredzone region size (linker/assembly)
   - T2: Volatile read/write semantics (hardware)
   - T3: Single-threaded execution (kernel design)
   - T4: BSS zero-initialization (loader)

3. **Complete coverage of all functions**:
   - `store()` → verified bounds check + trusted `raw_store()`
   - `load()` → verified bounds check + trusted `raw_load()`
   - Both public functions from original source are fully covered

4. **Rich algebraic properties proven**:
   - Read-after-write (`lemma_store_then_load`)
   - Non-interference (`lemma_store_does_not_affect_other`)
   - Store overwrite (`lemma_store_overwrite`)
   - Store commutativity (`lemma_store_commutes`)
   - Invariant preservation (`lemma_store_preserves_invariant`)

5. **Semantic equivalence preserved**: The verified code matches the original:
   - Same constant `KREDZONE_SIZE = 128`
   - Same bounds check logic `index >= NUM_ENTRIES`
   - Same error code `InvalidArgument`
   - Entry size correctly computed via conditional compilation matching `size_of::<usize>()`

6. **Clear API guidance**: Bold warnings in `store`/`load` docs direct users to ghost wrappers for functional correctness verification.

7. **Platform-aware design**: Uses conditional compilation for 32-bit vs 64-bit with lemmas proving correctness (`lemma_entry_size_matches_target`).

8. **Thorough test coverage**: Nine proof-mode tests covering:
   - Basic view properties
   - Update operations
   - Read-after-write
   - Non-interference
   - Store overwrite
   - Commutativity
   - Bounds checking
   - Well-formedness preservation
   - Ghost wrapper reasoning

## Detailed Analysis

### COVERAGE: ✅ Complete
Both public functions (`store`, `load`) from the original source have verified counterparts. The verification adds supporting infrastructure (ghost state, wrappers, lemmas) that enriches the verification.

### SPECIFICATIONS: ✅ Appropriate
- **Preconditions**: `store_with_ghost` requires `ghost.inv()` (well-formed ghost state)
- **Postconditions**: Both functions ensure `spec_is_valid_index(index)` on success and `!spec_is_valid_index(index)` on error
- Specs are neither too weak (they capture bounds safety) nor too strong (they don't over-constrain implementation)

### SOUNDNESS: ✅ Justified
- **1 assume**: In `load_with_ghost`, assumes volatile read returns expected value (T2 trust assumption)
- **2 core external_body**: `raw_store` and `raw_load` wrap volatile pointer operations
- All are clearly documented with justifications

### EQUIVALENCE: ✅ Verified
The verified code is semantically equivalent:
- Same error handling path
- Same bounds condition: `index >= KREDZONE_SIZE / size_of::<usize>()`
- Same success/error return types
- Logging omitted (documented divergence)

### INVARIANTS: ✅ Sufficient
- `is_well_formed()`: `len() == SPEC_NUM_ENTRIES`
- Ghost state invariant: `inv()` requires well-formedness
- Proven: All operations preserve invariants

### PROPERTIES: ✅ Comprehensive
Key safety/correctness properties proven:
1. **Memory safety**: Bounds checked before access
2. **Functional correctness** (abstract model):
   - Read-after-write
   - Non-interference
   - Commutativity
   - Idempotence
3. **Invariant preservation**: All operations maintain well-formedness

## Summary

This is an exemplary verification of a kernel memory component. The prover has:

1. **Maximized verified code**: Bounds checking is fully verified; only raw volatile operations are trusted
2. **Documented all trust assumptions**: T1-T4 explicitly listed with rationale for why each cannot be encoded
3. **Proven comprehensive algebraic properties**: The abstract model satisfies all expected memory semantics
4. **Provided a bridge for callers**: Ghost wrappers allow verified code to reason about sequences of operations

The verification represents the maximum achievable assurance for this component given Verus's limitations with volatile memory and extern C statics. The trust boundary is minimal, clearly documented, and justified.

**Recommendation**: Accept this verification as complete. No actionable issues remain.
