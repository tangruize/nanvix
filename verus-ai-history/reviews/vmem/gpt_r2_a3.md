# Review: vmem (gpt-5.1-codex-max)

## Grade: C

## Issues Found

### Critical
- Coverage gap still present: `lookup_page_table`, `lookup_kernel_page_table`, and `Drop` remain unverified/omitted (`external_body` / not modeled); no traversal or cleanup proofs were added.
- Clone semantics still mismatch: `clone` returns `mapping_count == 0`, not mirroring implementation’s user mapping clone/COW behavior; kernel/user share semantics remain abstract.
- Kernel mapping under-specification unchanged: `map_kpage`, `kctrl`, `copy_to_user_unaligned_unchecked`, `load`, and `pgdir` are still `external_body` with no modeled effects or failure paths, leaving kernel-space safety unproved.

### High
- Data-structure capacity mismatch persists: fixed `MAX_USER_PAGES=65536` array vs implementation’s unbounded lists; invariants don’t capture real limits.
- Ownership/return-type mismatch remains: `unmap` returns `usize` instead of `UserFrame`, and `map` takes raw `FrameAddress`; lifecycle/refcount semantics absent.
- Permission model still reduced: only three AccessPermission variants, no PTE-level tracking/enforcement; permission updates are not modeled.
- Region mapping spec remains weak: `spec_user_region_is_mapped` checks only page-aligned offsets, missing partial-page coverage during copies.
- Kernel/user boundary over-approximation persists: `spec_is_kernel_addr` is `!spec_is_user_addr`, marking invalid high addresses as kernel.

### Medium
- Physical bounds and dry-run semantics still missing: `copy_to_user_unaligned` uses single physical-bound check and omits per-page/dry-run behavior present in implementation.
- Mapping uniqueness after swap remains under-specified: `unmap` swap lacks frame freshness/duplicate checks when last entry shares vaddr; frame address sanity absent.
- Page-table coherence still unmodeled: no relation between mapping array and pgdir/pgtables; hardware translation consistency not proved.
- TLB/CR3 effects still absent: `load`/mapping operations do not specify translation updates/flush.

### Low
- Layout constants remain hardcoded instead of referencing config, risking drift.
- Helper proofs remain stubs with empty bodies; no constructive evidence.
- Zero-length region handling still rejects size==0 even where implementation treats zero-length as success, keeping behavioral mismatch.

## Summary
No substantive fixes were applied; prior gaps persist. Kernel path modeling, clone semantics, permission fidelity, capacity alignment, copy coverage, and hardware coherence remain unverified, so the verification is still incomplete and not sound for the real implementation.
