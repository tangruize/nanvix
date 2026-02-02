# Review: manager (claude-opus-4.5)

## Grade: B

## Verification Status

- **Command**: `./verus-ai/scripts/verify.sh manager`
- **Result**: 10 verified, 0 errors
- **Status**: PASSED

## Issues Found

### Critical

None.

### High

1. **Location**: `unmap_upage()` function (lines 398-421)
   - **Description**: The verified function does NOT free the frame back to the user pool after unmapping. The original (line 259-260) calls `vmem.unmap(vaddr)?` and then `self.physman.borrow_mut().free_user_frame(uframe)`. The verified version discards the frame address: `let _frame_addr: usize = vmem.unmap(vaddr)?;`. This is explicitly acknowledged as a "simplified model" but represents a **memory leak vulnerability** that is unverified.
   - **Suggested Fix**: Add `self.upool.free(frame_addr)` call after unmap, and add postcondition `self@.upool_free_count == old(self)@.upool_free_count + 1` to verify the frame is returned.

2. **Location**: `alloc_upages()` function - MISSING
   - **Description**: The original has `alloc_upages()` (lines 263-306) which allocates multiple contiguous user pages with a single call. This batch allocation is missing from the verified code. The `spec_can_alloc_upages()` spec exists but no executable implementation.
   - **Suggested Fix**: Add verified `alloc_upages(vmem, vaddr, nframes, access)` that proves: (a) all `nframes` pages are allocated from user pool, (b) all mappings are contiguous, (c) invariants preserved after full operation.

3. **Location**: `alloc_kpages()` function - MISSING
   - **Description**: The original has `alloc_kpages()` (lines 372-388) for batch kernel page allocation. The verified code only has `alloc_kpage()` for single-page allocation.
   - **Suggested Fix**: Add verified `alloc_kpages(clear, count)` with precondition `self@.has_kpool_capacity_for(count)` and postcondition that all returned pages satisfy their invariants.

### Medium

4. **Location**: `alloc_upage()` - `clear` parameter missing
   - **Description**: The original `alloc_upage()` (line 197) has signature `alloc_upage(&mut self, vmem, vaddr, access, clear: bool)` where `clear` triggers page zeroing via `vmem.memset()` (lines 231-235). The verified version (line 343) omits this security-critical parameter.
   - **Suggested Fix**: Add `clear: bool` parameter. For `clear=true`, call `vmem.memset()` and verify the memory is zeroed. Page zeroing prevents information leakage between processes.

5. **Location**: `new()` constructor - semantic difference
   - **Description**: The original `new()` (lines 166-182) returns `Result<(Vmem, Self), Error>` and creates a root Vmem internally via `Vmem::new()`. The verified `new()` (line 259) takes pre-initialized pools and returns just `Self`. This changes the initialization contract significantly.
   - **Suggested Fix**: Document this as a refinement abstraction. Consider adding a `new_with_vmem()` that returns `(Vmem, Self)` to match original semantics, or add documentation explaining the equivalence.

6. **Location**: `ctrl_upage()` - missing mapped precondition
   - **Description**: The function can only succeed if the page is already mapped (the underlying `vmem.uctrl()` will fail on unmapped addresses). The preconditions (lines 449-455) require alignment and user address, but NOT `old(vmem).spec_is_mapped(vaddr as int)`. Callers cannot statically prove success.
   - **Suggested Fix**: Add precondition `old(vmem).spec_is_mapped(vaddr as int)` to match the implicit requirement.

7. **Location**: `load_elf()` function - MISSING
   - **Description**: The original has `load_elf()` (lines 391-397) which maps ELF segments into user space. This is security-critical as it controls what code/data enters the user address space. The module header acknowledges this is intentionally not modeled.
   - **Suggested Fix**: Acceptable as documented, but note this is a coverage gap. A future enhancement could add a specification-level model verifying: (a) only user addresses are mapped, (b) allocation bounds are checked.

### Low

8. **Location**: `new_vmem()` postcondition (line 304)
   - **Description**: The postcondition states `result.mapping_count == 0`, but this models a "clone" operation. If the clone should preserve mappings, this postcondition is incorrect. If it should create an empty vmem that shares kernel mappings, this is correct but unclear.
   - **Suggested Fix**: Add a comment clarifying that `Vmem::clone()` intentionally creates an empty user mapping set (with shared kernel mappings).

9. **Location**: `VirtMemoryManagerView::pools_valid()` (lines 145-148)
   - **Description**: This specification function is defined but never used in any precondition, postcondition, or the `inv()` invariant. It's dead specification code.
   - **Suggested Fix**: Either integrate into `inv()` or remove it.

10. **Location**: `ctrl_upage()` mutability - `&self` vs `&mut self`
    - **Description**: The original uses `&mut self` (line 323) but verified uses `&self` (line 443). While technically correct (no mutation occurs), this semantic difference could complicate refinement proofs.
    - **Suggested Fix**: Change to `&mut self` for API parity, or document the intentional difference.

11. **Location**: `upool_id` missing from view
    - **Description**: `VirtMemoryManagerView` includes `kpool_id` (line 112) but not `upool_id`. For symmetry and complete provenance tracking, both pool IDs should be available.
    - **Suggested Fix**: Add `pub upool_id: int` to the view structure.

## Positive Observations

1. **No soundness holes**: The module has no `assume()` statements or `external_body` markers on verified functions. All proofs are justified.

2. **Well-documented abstractions**: The module header (lines 18-77) thoroughly explains why global state, Rc<RefCell<>>, PhysMemoryManager wrapping, and ELF loading are not modeled. These are reasonable verification scope decisions.

3. **Strong invariant preservation**: All operations verify that both `self.inv()` and `vmem.inv()` are maintained through the operation.

4. **Good precondition design**: `alloc_upage()` correctly requires:
   - Pool capacity: `old(self)@.has_upool_capacity()`
   - Vmem capacity: `old(vmem).has_mapping_capacity()`
   - Alignment: `vaddr as int % PAGE_SIZE as int == 0`
   - User address: `spec_is_user_addr(vaddr as int)`
   - No double-mapping: `!old(vmem).spec_is_mapped(vaddr as int)`

5. **Compositional verification**: Correctly delegates to verified sub-modules (Kpool, Upool, Vmem, KernelPage) following modular verification best practices.

6. **Clean mapping uniqueness**: The precondition `!old(vmem).spec_is_mapped(vaddr as int)` prevents double-mapping, a critical memory safety property.

7. **Frame provenance design**: Pool identifiers are tracked in the view for provenance verification.

8. **Verification passes cleanly**: All 10 verification conditions pass without errors.

## Coverage Analysis

| Original Function | Verified | Notes |
|-------------------|----------|-------|
| `init()` | ❌ | Global state - documented as out of scope |
| `get()` | ❌ | Global state - documented as out of scope |
| `get_mut()` | ❌ | Global state - documented as out of scope |
| `new()` | ⚠️ | Different signature/semantics |
| `new_vmem()` | ✅ | Verified |
| `alloc_upage()` | ⚠️ | Missing `clear` parameter |
| `unmap_upage()` | ⚠️ | Missing frame deallocation |
| `alloc_upages()` | ❌ | Not modeled |
| `ctrl_upage()` | ⚠️ | Missing mapped precondition |
| `alloc_kpage()` | ⚠️ | Missing `clear` parameter |
| `alloc_kpages()` | ❌ | Not modeled |
| `load_elf()` | ❌ | Documented as out of scope |

**Coverage**: 2/12 functions fully verified, 4/12 partially verified, 6/12 not verified

## Summary

The verification of `manager.rs` is **sound but incomplete**. The verified portions have no soundness holes and the specifications are appropriately strong. The abstraction decisions are well-documented.

**Key Gaps**:
1. **Memory leak**: `unmap_upage()` doesn't verify frame deallocation (HIGH priority)
2. **Batch operations**: `alloc_upages()` and `alloc_kpages()` missing (HIGH priority)
3. **Security property**: Page zeroing (`clear` param) not verified (MEDIUM priority)
4. **Incomplete precondition**: `ctrl_upage()` doesn't require mapping (MEDIUM priority)

**What IS verified**:
- Single-page allocation adds exactly one mapping
- Unmapping reduces mapping count by one
- Pool capacity is checked before allocation
- Address alignment and user-space bounds are verified
- Invariants are preserved across all operations
- No double-mapping occurs

**Recommendations** (by priority):
1. Add frame deallocation to `unmap_upage()` with proper postcondition
2. Add `old(vmem).spec_is_mapped(vaddr)` precondition to `ctrl_upage()`
3. Implement `alloc_upages()` and `alloc_kpages()`
4. Add `clear` parameter to allocation functions

With these additions, this would achieve an A-grade verification. The current state provides partial assurance - the modeled operations are correct, but critical resource management (deallocation) is unverified.
