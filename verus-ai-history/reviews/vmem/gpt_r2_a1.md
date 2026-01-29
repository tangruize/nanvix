# Review: vmem (gpt-5.1-codex-max)

## Grade: C

## Issues Found

### Critical
- Coverage gap: missing functions `lookup_page_table`, `lookup_kernel_page_table`, and `drop` (resource cleanup) have no verified counterparts; kernel mapping internals are abstracted away with `external_body`, so core traversal/cleanup logic is unverified.
- Semantic mismatch: `clone` in verified model drops all user mappings (`mapping_count == 0`), while original clones kernel tables and keeps user mappings; proof does not represent implementation behavior.
- Under-specification of kernel mapping operations: `map_kpage`, `kctrl`, `copy_to_user_unaligned_unchecked`, `load`, and `pgdir` are all `external_body` with high-level pre/post but no model of effects or failure paths, leaving kernel-space safety unproven.

### High
- Data structure abstraction shrink: verified model bounds user mappings to `MAX_USER_PAGES=65536` array, far below implementation’s unbounded linked lists; invariants and capacity checks no longer reflect real limits.
- Return-type and ownership mismatch: `unmap` returns raw `usize` instead of `UserFrame`, losing ownership/cleanup semantics; `map` accepts raw `FrameAddress` not `UserFrame`, so frame lifecycle and refcounts are unspecifed.
- Permission model weakened: AccessPermission reduced to three variants, omitting caching/write-through/no-exec bits; specs do not capture PTE permission semantics or enforce that `uctrl/kctrl/map` update permissions correctly.
- Region mapping spec too weak: `spec_user_region_is_mapped` only checks page-aligned offsets and assumes alignment; misses partially covered pages, so copy preconditions allow gaps.
- Kernel/user boundary spec differs: verified `is_kernel_addr` is just `!is_user_addr`, while original uses explicit kernel-range constants; over-approximates kernel space and could mark invalid high addresses as kernel.

### Medium
- Physical bounds assumptions: `copy_to_user_unaligned` only checks source physical bounds, assumes identity mapping for kernel dest; original checks physical bounds per page and does dry-run. Verified model omits per-page frame bounds and dry-run semantics.
- Mapping uniqueness proof ignores replacement behavior: `unmap` swaps last mapping into hole but invariant lacks frame validity check after swap (e.g., duplicates allowed if last entry shared vaddr); no frame address sanity or freshness checks.
- Invariants omit kernel/user table coherence: model doesn’t relate mapping array to pgdir/pgtables, so no proof that hardware tables reflect spec state.
- Lack of TLB flush/CR3 effects: `load`/map operations do not state that address translation updates take effect; possible stale mappings not modeled.

### Low
- Alignment/config constants hardcoded in verified model (USER_BASE/END/MEMORY_SIZE) rather than referencing config, risking drift from implementation.
- Helper proofs are stubs (no bodies) providing no constructive evidence; relies on constant comments rather than derived facts.
- `spec_is_kernel_region`/`spec_is_user_region` require `size>0`; zero-length regions return false, whereas original often treats zero as success (e.g., memcpy no-op), reducing equivalence.

## Positive Observations
- Core user mapping operations (`map`, `unmap`, `find_user_frame`, `uctrl`, `memset`, copies) include meaningful preconditions (user-space, alignment, capacity, mapped pages) and maintain a uniqueness invariant over virtual addresses.
- Address-space separation and overflow checks are captured explicitly in specs and helper lemmas.
- External hardware-touching functions are at least wrapped with pre/postconditions and marked as such, making their abstraction boundary explicit.

## Summary
The verification captures some user-space safety properties (bounds, alignment, uniqueness) but misses significant portions of the implementation and weakens behavior: kernel mapping logic, page-table traversal, and cleanup are unverified; `clone` semantics and capacity/ownership differ; permission and region-mapping specs are too coarse. Extensive refinement is needed to align the model with the real implementation and cover kernel paths, resource management, and permission/translation coherence before considering the verification trustworthy.
