# Review: manager (claude-opus-4.5)

**Verification Status**: PASSED (10 verified, 0 errors)  
**Cheating Patterns**: None (no assume, no external_body)

## Grade: B+

## Issues Found

### Critical

(None)

### High

- **Location**: `unmap_upage()` (lines 398-421)
- **Description**: The verified implementation does NOT free the user frame back to the pool after unmapping. The original implementation (line 260) calls `self.physman.borrow_mut().free_user_frame(uframe)`. This means the verified model does not capture frame deallocation, which is critical for verifying that:
  1. Frames are properly returned to the pool (no memory leaks).
  2. The user pool free count increases after unmap.
  3. Double-free prevention is correctly modeled.
  The comment on lines 393-397 admits this is a "simplification" but this is a significant semantic gap that breaks the key property of resource conservation.
- **Suggested Fix**: Either (a) modify `vmem.unmap()` to return a `UserFrame` instead of `usize`, allowing `upool.free()` to be called, or (b) add explicit frame tracking and prove that `self@.upool_free_count == old(self)@.upool_free_count + 1` on successful unmap.

---

- **Location**: `alloc_upages()` function (original lines 263-306)
- **Description**: The original source has `alloc_upages()` that allocates multiple user frames and maps them contiguously. This function is NOT modeled in the verified implementation. This is important for verifying:
  1. Multi-page allocation atomicity (or lack thereof - failure rollback).
  2. Correct address arithmetic (line 302: `vaddr.into_raw_value() + mem::PAGE_SIZE`).
  3. Pool capacity checks for batch operations.
  Batch allocation has different failure semantics - partial failure leaves some pages allocated.
- **Suggested Fix**: Add a verified `alloc_upages()` function that allocates and maps multiple frames with appropriate preconditions for batch capacity.

---

- **Location**: `alloc_kpages()` function (original lines 372-388)
- **Description**: The original source has `alloc_kpages()` for allocating multiple kernel pages. This is NOT modeled in the verified implementation. This is important for:
  1. Batch kernel allocation correctness.
  2. Proper construction of multiple KernelPage objects.
- **Suggested Fix**: Add a verified `alloc_kpages()` function with precondition `self@.has_kpool_capacity_for(count)`.

### Medium

- **Location**: `init()`, `get()`, `get_mut()` global state functions (original lines 87-154)
- **Description**: The global state management via `static mut MEMORY_MANAGER` is not modeled. While the documentation (lines 34-38) explains this is intentional (concurrency reasoning out of scope), the `init()` function contains important initialization logic including:
  1. One-time initialization check (line 93).
  2. Loading the root address space (line 174 in `new()`).
  3. The relationship between Vmem creation and manager initialization.
- **Suggested Fix**: Document these omissions more explicitly in the module header, or consider adding a ghost variable tracking initialization state.

---

- **Location**: `new()` constructor (verified lines 259-271 vs original lines 166-182)
- **Description**: The verified `new()` takes `(Kpool, Upool)` directly while the original takes `(LinkedList<KernelPage>, LinkedList<PageTable>, PhysMemoryManager)` and creates a `Vmem`. This semantic difference means:
  1. The original returns `(Vmem, Self)` but verified only returns `Self`.
  2. The Vmem creation logic (line 171) is not verified.
  3. The `root.load()` call (line 174) is not modeled.
- **Suggested Fix**: Consider adding a more faithful constructor or add documentation explaining the refinement relationship.

---

- **Location**: `alloc_upage()` clear parameter (original line 232)
- **Description**: The original `alloc_upage()` has a `clear: bool` parameter that triggers `vmem.memset(vaddr, 0)` (lines 232-235). The verified version does not model this parameter. Page clearing is security-critical to prevent information leakage.
- **Suggested Fix**: Add the `clear` parameter and verify that if `clear` is true, the postcondition reflects the page is zeroed.

---

- **Location**: `load_elf()` function (original lines 391-397)
- **Description**: The `load_elf()` function is not modeled. While the documentation (lines 54-58) explains ELF parsing is out of scope, this function is the main entry point for loading programs and calls `alloc_upages()` internally via `elf32_load()`.
- **Suggested Fix**: Consider adding at least a stub with appropriate preconditions/postconditions that capture the memory safety properties (e.g., all allocated pages are in user space, entry point is valid).

### Low

- **Location**: `new_vmem()` (verified lines 298-307)
- **Description**: The verified `new_vmem()` takes `&self` (immutable) while the original (line 185) takes `&self` as well, but the verified version returns a `Vmem` with `mapping_count == 0`. The original clones the vmem which would include existing user mappings.
- **Suggested Fix**: Clarify whether the postcondition `mapping_count == 0` is intentional or if it should preserve mappings from the source.

---

- **Location**: `ctrl_upage()` precondition (verified lines 443-460)
- **Description**: The verified `ctrl_upage()` does not require `vmem.spec_is_mapped(vaddr)` as a precondition, but the original (line 329) calls `vmem.uctrl()` which would fail if the page is not mapped. This could lead to inconsistent error handling models.
- **Suggested Fix**: Add `old(vmem).spec_is_mapped(vaddr as int)` as a precondition to match the original semantics.

---

- **Location**: Page table allocator closure (original lines 214-227, 274-287)
- **Description**: The original implementation allocates kernel frames for page tables dynamically via a closure. This is not modeled in the verified implementation, meaning:
  1. Page table memory consumption is not tracked.
  2. Potential allocation failures for page tables are not captured.
- **Suggested Fix**: Document this limitation or model page table allocation separately.

---

- **Location**: `VirtMemoryManagerView.pools_valid()` (lines 145-148)
- **Description**: The invariant checks `0 <= kpool_free_count <= kpool_capacity` but does not verify that `kpool_capacity > 0` and `upool_capacity > 0` at the view level. The underlying pool invariants ensure this, but it's not visible in the manager's abstract view.
- **Suggested Fix**: Add `kpool_capacity > 0 && upool_capacity > 0` to `pools_valid()` or document that it follows from pool invariants.

## Positive Observations

- **No unjustified `assume` or `external_body`**: The manager.rs module has no `assume` statements and no `external_body` functions. All 10 verified functions pass with full proofs.

- **Clean separation of concerns**: The manager properly delegates to Kpool, Upool, and Vmem modules, each with their own verified invariants. This compositional approach is good practice.

- **Strong preconditions and postconditions**: The `alloc_upage()` function has comprehensive preconditions including:
  - Pool capacity check (`old(self)@.has_upool_capacity()`)
  - Vmem capacity check (`old(vmem).has_mapping_capacity()`)
  - Address alignment (`vaddr as int % PAGE_SIZE as int == 0`)
  - User space check (`spec_is_user_addr(vaddr as int)`)
  - No double-mapping (`!old(vmem).spec_is_mapped(vaddr as int)`)

- **Invariant preservation**: All operations maintain `self.inv()` and `vmem.inv()` in postconditions.

- **Documentation quality**: The module header (lines 1-77) provides excellent rationale for abstraction decisions, including explicit notes about what is NOT modeled and why.

- **Specification functions**: The `spec_can_alloc_kpage()`, `spec_can_alloc_upage()`, etc. provide clean specification-level queries.

- **Proof functions**: The `proof_new_manager_invariant` and `proof_alloc_decreases_free` proofs demonstrate formal reasoning about the manager's properties.

## Summary

The verification of `manager.rs` is solid for the core allocation and mapping operations that are modeled. The implementation achieves B+ grade due to good specification coverage for the primary use case (single page allocation/unmap) but has notable gaps:

1. **Critical gap**: Frame deallocation in `unmap_upage()` is not modeled, meaning memory leak prevention is not verified.

2. **Coverage gaps**: `alloc_upages()`, `alloc_kpages()`, and `load_elf()` are not modeled. While ELF loading is justifiably out of scope, the batch allocation functions are important for completeness.

3. **Parameter omissions**: The `clear` parameter in `alloc_upage()` is not modeled, missing security-critical page zeroing verification.

4. **Semantic differences**: The verified constructor has different semantics than the original, returning only the manager instead of `(Vmem, Manager)`.

**Recommendations**:
1. Prioritize adding frame deallocation to `unmap_upage()` - this is the most impactful improvement.
2. Add `alloc_upages()` with batch allocation verification.
3. Add the `clear` parameter to `alloc_upage()`.
4. Consider adding `ctrl_upage()` precondition for mapped pages.

The verification successfully proves that pool invariants are preserved and that allocation/mapping operations have consistent pre/postconditions. The abstractions are reasonable and well-documented.
