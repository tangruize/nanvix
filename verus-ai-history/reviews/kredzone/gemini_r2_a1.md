# Review: kredzone (gemini-3-pro-preview)

## Grade: B+

## Issues Found

### High
- **Unverified Bounds Checks**: The `store` and `load` functions are marked `#[verifier::external_body]`. This means Verus ignores their bodies entirely. Consequently, the bounds checks (`if index >= KREDZONE_SIZE...`) are **not verified**; they are merely asserted to exist by the postconditions. If the implementation were to contain an off-by-one error or omit the check, Verus would not detect it.
    - **Suggested Fix**: Refactor the code to separate the verified logic from the unsafe primitives. Create a verified wrapper that performs the bounds check and then calls a smaller, trusted `external_body` function for the raw volatile access.
    ```rust
    pub fn store(index: usize, value: usize) -> Result<(), Error> 
        ensures ... 
    {
        if index >= NUM_ENTRIES { return Err(...); }
        unsafe_raw_store(index, value);
        Ok(())
    }
    
    #[verifier::external_body]
    fn unsafe_raw_store(index: usize, value: usize) ...
    ```

### Medium
- **Implicit Trust on Hardware/Compiler**: The `load_with_ghost` function uses `assume(res.unwrap() == spec_load_result(...))` to bridge the implementation and ghost state. This effectively trusts that the `volatile` read returns the last written value. While necessary given Verus's limitations with `volatile`, it should be explicitly noted that functional correctness relies on this assumption holding (Assumption T2 in docs).
    - **Suggested Fix**: Ensure this assumption is tracked in a top-level "Assumptions" document for the verified kernel, as it cannot be proven.

### Low
- **Initialization Assumption**: `create_initial_ghost` assumes the memory is zero-initialized (Assumption T4). If the startup assembly (`start.S`) or linker script fails to zero the `.bss` (or wherever `kredzone` lives), the ghost state will be out of sync with reality.
    - **Suggested Fix**: Add a runtime check or an explicit `init_kredzone()` function called during boot that zeroes the memory, ensuring the state matches the ghost model, rather than relying on implicit BSS behavior.

## Positive Observations
- **Comprehensive Abstract Model**: The `KernelRedZoneView` and associated lemmas provide a complete and rigorous model of the component's behavior, including proofs for read-after-write, non-interference, and commutativity.
- **Detailed Documentation**: The file includes excellent documentation explaining the verification strategy, the "specification-first" approach, and the specific trust assumptions (T1-T4).
- **Verified Wrapper Layer**: The `_with_ghost` functions provide a clean way for verified client code to reason about the state of the red zone, bridging the gap between the unverified implementation and the proven model.
- **Cross-Architecture Support**: The conditional compilation for `ENTRY_SIZE` and `lemma_entry_size_matches_target` demonstrates good attention to detail for multi-platform correctness.

## Summary
The `kredzone` module demonstrates a high-quality "specification-first" verification approach. The abstract model and its properties are rigorously proven. However, the use of `external_body` for the entire `store`/`load` functions leaves the actual executable code (specifically the bounds checks) unverified. Refactoring to verify the bounds checks while keeping only the raw memory access trusted would significantly strengthen the guarantee. The current verification proves that *if* the implementation respects the contract, *then* the system is safe, but it does not prove the implementation respects the contract.
