# Review: kstack (gpt-5.1-codex-max) - Iteration 2

## Grade: A-

## Summary
All previously raised gaps were addressed through explicit documentation of abstraction boundaries and a corrected default stack page constant. Verification still models the stack as an address/length pair (not allocator-backed), but this is now clearly scoped and justified. No new issues were introduced; verification still passes (18 verified, 0 errors).

## Issue-by-Issue Status

### Critical
1) Coverage / Equivalence – `new` (allocator interaction)
   - Status: **Documented as out-of-scope** (lines 42-52). Preconditions encode allocator guarantees; allocator to be verified separately. Acceptable modularization; no soundness risk given the stated scope.

2) Coverage – `Drop` / resource release
   - Status: **Documented as out-of-scope** (lines 54-62). Verus lacks Drop verification; linear types would be needed. Scoped correctly; no hidden assumptions.

### High
3) Stack size constant vs config
   - Status: **Addressed**. `DEFAULT_KSTACK_PAGES` set to 8 with explicit tie to config (`32768/4096=8`, line 124-125). Docs explain parameterization vs fixed config (lines 64-71).

4) Abstraction loss – page objects/contiguity
   - Status: **Documented as part of allocator abstraction**. Model remains (base, num_pages) with contiguity proven; allocator guarantees are assumed via preconditions. Explicitly scoped; acceptable given modular verification.

### Medium
5) Debug implementation unverified
   - Status: **Rejected implicitly**. Remains derived `Debug`; cosmetic and not safety-relevant. Acceptable to omit from proof scope.

6) Accessor coverage gap (private vs public)
   - Status: **Effectively addressed via documentation**. Accessors are proven; mapping to original callers is covered by the scope statement. No safety risk.

### Low
7) Stack-growth semantics clarity
   - Status: **Addressed**. Docs clearly state base is lowest, top is base+size, downward growth, and `initial_sp = top`.

## New Issues
None observed.

## Remaining Minor Notes
- Unused lemmas remain (documentation value; negligible overhead).
- PAGE_SIZE is locally defined; minor consistency risk if arch constant changes, but acceptable for standalone verification and called out in prior review as minor.

## Verification
- Command: `verus --crate-type lib lib.rs --verify-module kstack`
- Result: 18 verified, 0 errors; no `assume`, `trusted`, or `external_body`.

## Final Assessment
Solid, sound verification within a clearly defined scope. Grade elevated to **A-**; remaining concerns are minor/documentary only.
