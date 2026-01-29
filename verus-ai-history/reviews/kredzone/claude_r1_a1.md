# Review: kredzone (claude-opus-4.5)

## Grade: B+

## Issues Found

### Critical
- None

### High
- **Location**: `store()` and `load()` functions (lines 265, 303)
  - **Description**: Both `store()` and `load()` are marked `#[verifier::external_body]`, which means the postconditions are trusted but not verified. The specifications only relate input validity to success/failure, but critically:
    1. `store()` does not specify that it modifies the abstract state (no ghost state parameter)
    2. `load()` does not specify what value is returned (returns placeholder `0`)
    3. No connection between the executable functions and the abstract `KernelRedZoneView` model
  - **Suggested Fix**: Use tracked ghost parameters to thread the `KernelRedZoneGhost` state through `store()` and `load()`, ensuring the specifications connect the concrete operations to the abstract model. This requires:
    ```rust
    pub fn store(index: usize, value: usize, Tracked(ghost): Tracked<&mut KernelRedZoneGhost>) 
        -> (result: Result<(), Error>)
        requires old(ghost).inv(),
        ensures 
            result.is_ok() ==> ghost.view == old(ghost).view.update(index as int, value),
            result.is_err() ==> !spec_is_valid_index(index as int),
    ```

### Medium
- **Location**: `KernelRedZoneGhost` struct (lines 225-239)
  - **Description**: `KernelRedZoneGhost` is defined but never used by the core functions. The ghost state abstraction is orphaned—there's no way to obtain an initial instance or pass it through operations. This makes the entire abstract state model disconnected from verification.
  - **Suggested Fix**: Either remove `KernelRedZoneGhost` if pure functional verification is intended, or integrate it properly with the `store`/`load` functions using tracked parameters.

- **Location**: `load()` return value (line 319)
  - **Description**: The `load()` function returns `Ok(0)` as a placeholder. While acceptable for `external_body`, the specification should still capture that the returned value corresponds to the abstract state at the given index.
  - **Suggested Fix**: Add a ghost parameter to bind the returned value to `spec_load_result(view, index)`.

- **Location**: Constants for `ENTRY_SIZE` (lines 67-74)
  - **Description**: The original uses `mem::size_of::<usize>()` which is computed at compile time. The verified version uses conditional compilation with hardcoded values. While functionally equivalent, this introduces a divergence in how the size is determined and could be fragile if ported to new architectures.
  - **Suggested Fix**: Add a static assertion or proof lemma that `ENTRY_SIZE == size_of::<usize>()` to ensure consistency across platforms.

### Low
- **Location**: Documentation header (lines 5-49)
  - **Description**: The documentation claims several properties are "verified" (bounds safety, element isolation, read-after-write, etc.), but due to `external_body`, these are only proven for the abstract model, not for the actual implementation. This could be misleading.
  - **Suggested Fix**: Clarify in documentation that properties are proven for the abstract specification model, with the implementation trusted via `external_body`.

- **Location**: Error handling (line 274, 313)
  - **Description**: The original implementation uses `error!()` macro for logging before returning errors. The verified version omits this logging. While not a correctness issue, it's a semantic divergence.
  - **Suggested Fix**: Document that logging is omitted in the verified version, or add a note that this is intentional for verification purposes.

- **Location**: Proof tests (lines 473-566)
  - **Description**: The proof tests are comprehensive for the abstract model but do not test any properties about the executable code paths due to `external_body` encapsulation.
  - **Suggested Fix**: Consider adding executable tests (if Verus supports them) or at minimum document that only abstract model properties are tested.

## Positive Observations

1. **Comprehensive Abstract Model**: The `KernelRedZoneView` abstraction correctly models the kernel red zone as a sequence of `usize` values with well-defined update semantics.

2. **Complete Lemma Coverage**: The verification includes all essential lemmas:
   - `lemma_store_then_load`: Read-after-write property
   - `lemma_store_does_not_affect_other`: Element isolation/non-interference
   - `lemma_store_overwrite`: Idempotence under overwrite
   - `lemma_store_commutes`: Commutativity of independent stores
   - `lemma_update_preserves_well_formed`: Invariant preservation

3. **Correct Bounds Checking Logic**: The bounds checking logic (`index >= NUM_ENTRIES`) correctly matches the original implementation's bounds check (`index >= KREDZONE_SIZE / mem::size_of::<usize>()`).

4. **Platform-Aware Constants**: The use of `#[cfg(target_pointer_width = "...")]` for `ENTRY_SIZE` correctly handles both 32-bit and 64-bit targets.

5. **Clean Specification Structure**: The separation of specification constants (`SPEC_NUM_ENTRIES`) from executable constants (`NUM_ENTRIES`) follows Verus best practices.

6. **No `assume` Statements**: The verification contains no unjustified `assume` statements—all trusted code is explicitly marked with `external_body`.

7. **100% Function Coverage**: Both public functions (`store` and `load`) from the original are covered.

## Summary

The verification of `kredzone.rs` demonstrates a well-structured approach to verifying a low-level kernel component. The abstract model (`KernelRedZoneView`) is mathematically sound and the lemmas prove all the essential safety and functional correctness properties for an indexed array abstraction.

**Strengths:**
- All 26 verification conditions pass
- Comprehensive lemma library covering key properties
- No unjustified assumptions
- Clean separation between spec and exec code

**Weaknesses:**
- The use of `external_body` on both core functions (`store` and `load`) means the connection between the implementation and the specification is trusted, not verified
- The `KernelRedZoneGhost` abstraction is defined but unused, leaving the abstract state model disconnected from actual function specifications
- The specifications for `store` and `load` are too weak—they only specify input validity, not the actual effects on state

**Recommendations:**
1. **Primary**: Consider threading ghost state through the API or documenting why this is not feasible (e.g., due to the extern "C" global variable nature)
2. **Secondary**: Strengthen `external_body` postconditions to at least specify that successful operations interact correctly with the abstract model
3. **Documentation**: Update the header documentation to accurately reflect what is trusted vs. verified

The verification provides value by ensuring the abstract model is correct and by documenting intended behavior through specifications, but the trust boundary at `external_body` limits the end-to-end assurance. For a kernel component handling raw memory via extern "C", this is a reasonable engineering tradeoff, but it should be clearly documented.
