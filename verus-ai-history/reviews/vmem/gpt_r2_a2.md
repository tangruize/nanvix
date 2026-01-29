# Review: vmem (gpt-5.1-codex-max)

## Grade: C

## Issues Found

### Critical
- Coverage gap persists: `lookup_page_table`, `lookup_kernel_page_table`, and `Drop` remain unverified/omitted with `external_body` abstraction; no new proofs added for traversal or cleanup.
- Semantic mismatch unchanged: `clone` still clears all user mappings (`mapping_count==0`), not modeling the implementation’s user-mapping cloning/copy-on-write behavior.
- Kernel mapping under-specification remains: `map_kpage`, `kctrl`, `copy_to_user_unaligned_unchecked`, `load`, and `pgdir` are still `external_body` with no modeled effects or failure paths, leaving kernel-space safety unproved.

### High
- Data structure mismatch unchanged: model caps at `MAX_USER_PAGES=65536` fixed array while implementation uses unbounded lists; invariants still not aligned with real capacity.
- Ownership/return-type mismatch remains: `unmap` returns `usize` not `UserFrame`, and `map` accepts raw `FrameAddress`; lifecycle/refcount semantics are still absent.
- Permission model still reduced: AccessPermission only has three variants and no PTE-level permission tracking; specs do not enforce permission updates.
- Region mapping spec still weak: `spec_user_region_is_mapped` only checks page-aligned offsets, missing partial-page coverage during copies.
- Kernel/user boundary over-approximation remains: `spec_is_kernel_addr` defined as `!spec_is_user_addr`, so invalid high addresses are still treated as kernel.

### Medium
- Physical bounds and dry-run semantics still absent: `copy_to_user_unaligned` only checks kernel source physical bounds once; no per-page/dry-run behavior modeled as in implementation.
- Mapping uniqueness after swap still weak: `unmap` swap lacks frame freshness/duplicate checks when last entry shares vaddr; no sanity on frame addresses.
- Page-table coherence still missing: model does not relate mapping array to pgdir/pgtables; hardware translation consistency is unproved.
- TLB/CR3 effects still unmodeled: `load`/map operations do not specify translation update/flush behavior.

### Low
- Hardcoded layout constants remain instead of referencing config; risk of drift persists.
- Helper proofs remain stubbed with no bodies; provide no constructive evidence.
- Zero-length region handling still rejects size==0 whereas implementation often treats zero-length as success; behavioral mismatch persists.

## Summary
The prover added rationale comments but did not fix the previously reported gaps. Kernel path modeling, clone semantics, permission fidelity, capacity alignment, copy coverage, and hardware coherence remain unverified, leaving major divergences from the implementation. Verification is still incomplete and cannot be considered sound for the real vmem implementation.
