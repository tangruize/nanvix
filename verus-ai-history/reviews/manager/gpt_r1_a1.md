# Review: manager (gpt-5.1-codex-max)

## Grade: C

## Issues Found

### Critical
- Coverage gap across module: verified `manager.rs` omits global init/access (`init`, `get`, `get_mut`), bulk allocators (`alloc_upages`, `alloc_kpages`), and ELF loader (`load_elf`), leaving large portions of original behavior unverified and unchecked.
- `unmap_upage` equivalence mismatch: verified version drops the returned frame instead of freeing it back to the user pool/physman, so capacity and provenance are not restored and leaks are unmodelled.
- `ctrl_upage` spec too weak: lacks precondition that the page is mapped; original relies on vmem failure when unmapped, so verification permits silent success on unmapped addresses.

### High
- `alloc_upage` ignores kernel page-table allocation path: original may allocate kernel frames for page tables and fail with `ResourceBusy`; verified spec only requires user-pool capacity and cannot rule out runtime failures from missing kernel capacity.
- `new_vmem` postcondition forces `mapping_count == 0`, which need not hold for `Vmem::clone` (it clones root/kernel mappings and potentially user mappings), so verified behavior diverges from implementation.
- Shared ownership/borrow failure paths elided: modeling `Rc<RefCell<PhysMemoryManager>>` as single-owner removes the `try_borrow_mut` error path and `ResourceBusy` errors, leaving concurrency/synchronization preconditions unverified.

### Medium
- Kernel allocation API mismatch: verified `alloc_kpage` lacks the `clear` parameter and omits the `ResourceBusy` error case, so behavior and guarantees differ from the original interface.
- Bulk user allocation unsupported: original `alloc_upages` allocates a range with per-page mapping and range-busy TODO; verified code has no equivalent coverage or spec for multi-page allocation ordering and failure atomicity.

### Low
- None.

## Positive Observations
- Pool invariants are preserved and allocation specs require page alignment and user-address ranges.
- Mapping operations increase/decrease `mapping_count` and assert vmem invariants when modeled.
- Frame provenance and capacity are tracked in the abstract view for single-page paths.

## Summary
The verification captures some single-page allocation/mapping safety but leaves major portions of the manager unverified and alters semantics (frame freeing, borrowing, page-table allocation, cloning state). Significant coverage and equivalence gaps remain; extend the Verus model to include the full API surface, restore frame return on unmap, and encode borrow/page-table capacity and mapping preconditions to align with the implementation.
