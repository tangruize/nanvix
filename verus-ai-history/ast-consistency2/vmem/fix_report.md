# Exec Consistency Fix: vmem

## Summary
- Mismatches fixed: 0 (all 19 are documented equivalences)
- Missing functions added: 0 (all 3 are documented omissions)
- Documented equivalences: 22
- Reviewer feedback addressed: 4 minor items (documentation improvements)

## Architecture Note

The verus verification model for `vmem` uses a fundamentally different data representation
than the original implementation. The original uses:
- `LinkedList<Rc<RefCell<(PageTableAddress, PageTable<PageTableStorage>)>>>` for kernel page tables
- `LinkedList<Rc<RefCell<KernelPage>>>` for kernel pages
- `LinkedList<(PageTableAddress, PageTable<PageTableStorage>)>` for user page tables
- `PageDirectory` with hardware-specific page table operations

The verus model replaces all of this with:
- `[PageMapping; MAX_USER_PAGES]` — a fixed-size array of `(vaddr, frame_addr, valid)` triples
- `mapping_count: usize` — count of valid entries

This abstraction is documented extensively in the module header (lines 1-156 of the verus file).
All type differences (`PageAligned<VirtualAddress>` → `usize`, `UserFrame` → `usize`,
`AccessPermission` enum simplification) follow from this fundamental modeling decision.

## Changes

| Function | Action | Justification |
|----------|--------|---------------|
| `Vmem` [struct_Vmem.diff](struct_Vmem.diff) | [struct_Vmem_source.rs](struct_Vmem_source.rs) | [struct_Vmem_verus.rs](struct_Vmem_verus.rs) (struct) | DOCUMENT_EQUIVALENT | Array-based model replaces LinkedList/Rc/RefCell for verification tractability. Kernel mappings not tracked. |
| `PageMapping` [struct_PageMapping_verus.rs](struct_PageMapping_verus.rs) (struct) | DOCUMENT_JUSTIFIED | Extra struct in verus: helper for array-based mapping model. Required for verification. |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | DOCUMENT_EQUIVALENT | Signature simplified: kernel params removed (kernel mappings not modeled). Core logic equivalent: creates empty user mapping space. |
| `clone` [clone.diff](clone.diff) | [clone_source.rs](clone_source.rs) | [clone_verus.rs](clone_verus.rs) | DOCUMENT_EQUIVALENT | Returns Self instead of Result (error path from physical_address() not modeled). Core behavior: cloned Vmem has empty user mappings. |
| `load` [load.diff](load.diff) | [load_source.rs](load_source.rs) | [load_verus.rs](load_verus.rs) | DOCUMENT_EQUIVALENT | `external_body`: hardware CR3 register load cannot be modeled. Return type matches (`Result<(), Error>`). |
| `pgdir` [pgdir.diff](pgdir.diff) | [pgdir_source.rs](pgdir_source.rs) | [pgdir_verus.rs](pgdir_verus.rs) | DOCUMENT_EQUIVALENT | Returns `usize` (raw address) instead of `&PageDirectory`. PageDirectory type not modeled. `external_body` justified. |
| `map_kpage` [map_kpage.diff](map_kpage.diff) | [map_kpage_source.rs](map_kpage_source.rs) | [map_kpage_verus.rs](map_kpage_verus.rs) | DOCUMENT_EQUIVALENT | `external_body`: kernel mapping uses Rc/RefCell linked lists not modeled. Simplified signature removes `page_table_allocator` callback. |
| `is_user_addr` [is_user_addr.diff](is_user_addr.diff) | [is_user_addr_source.rs](is_user_addr_source.rs) | [is_user_addr_verus.rs](is_user_addr_verus.rs) | DOCUMENT_EQUIVALENT | Original: `virt_addr >= USER_BASE && virt_addr < USER_END`. Verus: `vaddr >= USER_BASE && vaddr < USER_END`. Identical logic, local constants match `config::memory_layout`. |
| `is_kernel_addr` [is_kernel_addr.diff](is_kernel_addr.diff) | [is_kernel_addr_source.rs](is_kernel_addr_source.rs) | [is_kernel_addr_verus.rs](is_kernel_addr_verus.rs) | DOCUMENT_EQUIVALENT | Both: `!Self::is_user_addr(addr)`. Identical logic. |
| `is_user_region` [is_user_region.diff](is_user_region.diff) | [is_user_region_source.rs](is_user_region_source.rs) | [is_user_region_verus.rs](is_user_region_verus.rs) | DOCUMENT_EQUIVALENT | Both: reject size==0; `checked_add(size-1)` for overflow; `is_user_addr(start) && is_user_addr(end)`. Identical logic. |
| `is_kernel_region` [is_kernel_region.diff](is_kernel_region.diff) | [is_kernel_region_source.rs](is_kernel_region_source.rs) | [is_kernel_region_verus.rs](is_kernel_region_verus.rs) | DOCUMENT_EQUIVALENT | Same pattern as `is_user_region` [is_user_region.diff](is_user_region.diff) | [is_user_region_source.rs](is_user_region_source.rs) | [is_user_region_verus.rs](is_user_region_verus.rs) with `is_kernel_addr` [is_kernel_addr.diff](is_kernel_addr.diff) | [is_kernel_addr_source.rs](is_kernel_addr_source.rs) | [is_kernel_addr_verus.rs](is_kernel_addr_verus.rs). Identical logic. |
| `is_physical_region` [is_physical_region.diff](is_physical_region.diff) | [is_physical_region_source.rs](is_physical_region_source.rs) | [is_physical_region_verus.rs](is_physical_region_verus.rs) | DOCUMENT_EQUIVALENT | Both: reject size==0; `checked_add(size-1)`; `start < MEMORY_SIZE && end < MEMORY_SIZE`. Identical logic with local `MEMORY_SIZE` constant. |
| `map` [map.diff](map.diff) | [map_source.rs](map_source.rs) | [map_verus.rs](map_verus.rs) | DOCUMENT_EQUIVALENT | Signature simplified (removes `page_table_allocator`, `UserFrame` → `FrameAddress`, `PageAligned<VirtualAddress>` → `usize`). Core logic: validates user addr, page alignment, not already mapped, adds mapping. |
| `unmap` [unmap.diff](unmap.diff) | [unmap_source.rs](unmap_source.rs) | [unmap_verus.rs](unmap_verus.rs) | DOCUMENT_EQUIVALENT | Returns `usize` instead of `UserFrame`. Core logic: validates user addr, finds mapping, removes it (swap-with-last). Original uses page table lookup; verus uses array scan. |
| `find_user_frame` [find_user_frame.diff](find_user_frame.diff) | [find_user_frame_source.rs](find_user_frame_source.rs) | [find_user_frame_verus.rs](find_user_frame_verus.rs) | DOCUMENT_EQUIVALENT | Original iterates `user_page_tables` linked list. Verus scans `mappings` array. Both find physical frame for a virtual address. Type: `PageAligned<VirtualAddress>` → `usize`. |
| `copy_from_user_unaligned` [copy_from_user_unaligned.diff](copy_from_user_unaligned.diff) | [copy_from_user_unaligned_source.rs](copy_from_user_unaligned_source.rs) | [copy_from_user_unaligned_verus.rs](copy_from_user_unaligned_verus.rs) | DOCUMENT_EQUIVALENT | Validation checks identical in order and error codes: size==0→InvalidArgument, !user_region(src)→BadAddress, !kernel_region(dst)→BadAddress. Physical copy loop not modeled (returns Ok after validation). |
| `copy_to_user_unaligned` [copy_to_user_unaligned.diff](copy_to_user_unaligned.diff) | [copy_to_user_unaligned_source.rs](copy_to_user_unaligned_source.rs) | [copy_to_user_unaligned_verus.rs](copy_to_user_unaligned_verus.rs) | DOCUMENT_EQUIVALENT | Both call `copy_to_user_unaligned_unchecked` [copy_to_user_unaligned_unchecked.diff](copy_to_user_unaligned_unchecked.diff) | [copy_to_user_unaligned_unchecked_source.rs](copy_to_user_unaligned_unchecked_source.rs) | [copy_to_user_unaligned_unchecked_verus.rs](copy_to_user_unaligned_unchecked_verus.rs) twice (dry_run=true, then false). Identical control flow. |
| `copy_to_user_unaligned_unchecked` [copy_to_user_unaligned_unchecked.diff](copy_to_user_unaligned_unchecked.diff) | [copy_to_user_unaligned_unchecked_source.rs](copy_to_user_unaligned_unchecked_source.rs) | [copy_to_user_unaligned_unchecked_verus.rs](copy_to_user_unaligned_unchecked_verus.rs) | DOCUMENT_EQUIVALENT | `external_body`: performs unsafe physical memory copies with page-by-page loop. Cannot be modeled without hardware model. Preconditions capture validation requirements. |
| `memset` [memset.diff](memset.diff) | [memset_source.rs](memset_source.rs) | [memset_verus.rs](memset_verus.rs) | DOCUMENT_EQUIVALENT | Original: `find_user_frame` [find_user_frame.diff](find_user_frame.diff) | [find_user_frame_source.rs](find_user_frame_source.rs) | [find_user_frame_verus.rs](find_user_frame_verus.rs) then `__phys_memset`. Verus: validates user addr, page alignment, checks mapped, returns Ok (physical memset not modeled). Validation logic equivalent. |
| `uctrl` [uctrl.diff](uctrl.diff) | [uctrl_source.rs](uctrl_source.rs) | [uctrl_verus.rs](uctrl_verus.rs) | DOCUMENT_EQUIVALENT | Original: validates user addr (type-enforced alignment), page table lookup, `ctrl()`. Verus: validates user addr, explicit alignment check (since `usize` not `PageAligned`), checks mapped. Same logical validation. |
| `kctrl` [kctrl.diff](kctrl.diff) | [kctrl_source.rs](kctrl_source.rs) | [kctrl_verus.rs](kctrl_verus.rs) | DOCUMENT_EQUIVALENT | `external_body`: kernel mapping permission change not modeled. Preconditions capture kernel addr and page alignment requirements. |
| `drop` [drop_source.rs](drop_source.rs) | DOCUMENT_OMISSION | Drop trait deallocates page tables and releases kernel pages. Resource management out of scope for memory safety verification. Documented in module header (lines 91-97). |
| `lookup_page_table` [lookup_page_table_source.rs](lookup_page_table_source.rs) | DOCUMENT_OMISSION | Internal helper iterating linked list to find page table by physical address. Abstracted by array model which maintains mappings directly. Documented in module header (lines 85-90). |
| `lookup_kernel_page_table` [lookup_kernel_page_table_source.rs](lookup_kernel_page_table_source.rs) | DOCUMENT_OMISSION | Internal helper for kernel page table lookup. Kernel mappings not modeled. Documented in module header (lines 85-90). |

## Verification: PASS

```
verification results:: 30 verified, 0 errors
```

## Reviewer Feedback (Minor Issues Addressed)

### Minor 1: `memset` [memset.diff](memset.diff) | [memset_source.rs](memset_source.rs) | [memset_verus.rs](memset_verus.rs) validation divergence
**Action**: Added `# Validation Difference` doc comment explaining that explicit
`is_user_addr` [is_user_addr.diff](is_user_addr.diff) | [is_user_addr_source.rs](is_user_addr_source.rs) | [is_user_addr_verus.rs](is_user_addr_verus.rs) and alignment checks compensate for the type simplification from
`PageAligned<VirtualAddress>` to `usize`. The checks are stricter but sound.

### Minor 2: `copy_from_user_unaligned` [copy_from_user_unaligned.diff](copy_from_user_unaligned.diff) | [copy_from_user_unaligned_source.rs](copy_from_user_unaligned_source.rs) | [copy_from_user_unaligned_verus.rs](copy_from_user_unaligned_verus.rs) dry-run abstraction
**Action**: Replaced generic comment with detailed explanation of the abstraction:
the two-pass loop (dry-run + actual) is replaced by the `spec_user_region_is_mapped`
precondition. Documented the limitation that physical bounds of source frames are
not postconditioned (established at `map()` time by allocator invariant instead).

### Minor 3: `unmap` [unmap.diff](unmap.diff) | [unmap_source.rs](unmap_source.rs) | [unmap_verus.rs](unmap_verus.rs) swap-with-last strategy
**Action**: Added comment explaining that swap-with-last differs from linked-list
removal (which preserves insertion order) but is correct because the spec uses
existential quantifiers — ordering is irrelevant for all invariants and postconditions.

### Minor 4: `clone` [clone.diff](clone.diff) | [clone_source.rs](clone_source.rs) | [clone_verus.rs](clone_verus.rs) returns `Self` not `Result<Self, Error>`
**Action**: Expanded doc comment to explicitly state that the verus model cannot
express failure modes of cloning. Added a concrete `external_body` wrapper example
showing the signature needed for refinement proofs.

## Detailed Justification for Model Differences

### Why No Code Changes Were Needed

All 22 inconsistencies detected by the AST diff are **intentional structural differences**
arising from the verification model's design decisions, not unintentional logic changes.
The differences fall into three categories:

1. **Type Simplifications (13 functions)**: The original uses rich Rust types
   (`PageAligned<VirtualAddress>`, `UserFrame`, `PageDirectory`, etc.) that encode
   invariants in the type system. The verus model uses `usize` with explicit runtime
   checks (alignment, bounds) that the type system would otherwise guarantee. This is
   equivalent because the verus model's explicit checks cover the same preconditions.

2. **External Body Functions (5 functions: load, pgdir, map_kpage, kctrl,
   copy_to_user_unaligned_unchecked)**: These perform hardware operations (CR3 load,
   TLB flush) or operate on kernel-only data structures (Rc/RefCell linked lists).
   They cannot be modeled without a hardware model or linear types. Their contracts
   capture the essential preconditions and postconditions.

3. **Missing Functions (3 functions: drop, lookup_page_table,
   lookup_kernel_page_table)**: These are implementation details that don't affect
   the verified memory safety properties. `drop` [drop_source.rs](drop_source.rs) handles resource cleanup,
   `lookup_*` iterate linked lists. Both are abstracted by the array-based model.
