# Review: vmem (gemini-3-pro-preview)

## Grade: B

## Issues Found

### High
- **Control Flow Divergence in `copy_to_user_unaligned`**: The verified implementation of `copy_to_user_unaligned` does not match the structure of the original code. The original delegates to `copy_to_user_unaligned_unchecked` (twice: for dry-run and execution), whereas the verified version implements checks inline and returns `Ok(())` without calling the unchecked primitive.
- **Unused External Body**: The `copy_to_user_unaligned_unchecked` function is defined as `external_body` in the verified module but is never called by the verified `copy_to_user_unaligned`. This means the verification does not model the actual delegation to the low-level copy routine.
- **Abstraction Gap (Linked List vs Array)**: The verification uses a `mappings` array (max 1024 pages) to model the page tables, while the implementation uses a linked list of `PageTable` objects. This abstracts away significant complexity (allocation, ownership, traversal) which remains unverified. While documented as a simplification, it means the verification is checking a *model*, not the *code*.

### Medium
- **Handling of Unmapped Pages**: The verified `copy_to_user_unaligned` requires the destination region to be mapped as a precondition (`requires ... spec_user_region_is_mapped`). The original implementation handles unmapped pages at runtime (panicking in non-dry-run mode). The verification proves safety *assuming* the caller ensures mapping, but does not verify that the function itself handles unmapped pages correctly (e.g. by panicking or returning an error).
- **Missing `Drop` Logic**: The `Drop` implementation is missing from the verified code. While memory safety properties are the focus, resource leaks (failing to free page tables) are a potential issue in the original complex linked-list implementation that is not covered by this model.

### Low
- **Redundant Logic in Original**: The original `copy_to_user_unaligned` calls `unchecked` twice, performing the same region checks twice. The verified code simplifies this to a single set of checks, which is cleaner but technically not equivalent behavior.
- **Limited Scope of Properties**: The verification proves address space separation and bounds checking, but does not prove functional correctness (i.e., that data is actually copied, or that the correct data is written to the correct physical address).

## Positive Observations
- **Clear Separation of Concerns**: The use of `external_body` for hardware-specific operations (`load`, `pgdir`) is well-justified and documented.
- **Strong Invariants**: The `inv()` predicate effectively captures the essential safety properties: disjoint user/kernel spaces, valid address bounds, and mapping uniqueness.
- **Documentation**: The verified file contains excellent documentation explaining the abstraction decisions (Array vs Linked List) and the scope of verification.
- **Precondition Discovery**: The verification correctly identifies that `copy_to_user_unaligned` is only safe if the destination is mapped, enforcing this via preconditions.

## Summary
The `vmem` verification is a solid **abstract model verification**. It proves that if the memory management logic follows the specified rules (separation of user/kernel, bounds checking), it is safe. However, due to the significant structural differences between the verified model (array-based, flat) and the actual implementation (linked-list, hierarchical page tables), the verification does not guarantee the correctness of the actual code's logic (pointer manipulation, allocation, error handling paths). The divergence in `copy_to_user_unaligned` is the most significant discrepancy, as the verified model effectively mocks out the operation rather than verifying the actual control flow.
