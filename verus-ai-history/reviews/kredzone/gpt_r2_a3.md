# Review: kredzone (gpt-5.1-codex-max)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Function: store/load specs** — Postconditions still only assert index validity; they do not relate stored/loaded values to memory, nor do they guarantee the precise `InvalidArgument` error on failure. Core behavior (write/read of the red-zone slot) remains unverified in `external_body`, so functional correctness is still trusted rather than proved.
- **Function: load_with_ghost** — The wrapper still uses an `assume` to connect the returned value to the ghost state (trust assumption T2). Read-after-write relies on this unchecked axiom, leaving soundness dependent on the assumption rather than a verified link to modeled memory.

### Medium
- **Module: Trust boundary size/zero assumptions (T1/T4)** — Size and zero-initialization remain documented assumptions without proof or runtime/compile-time checks tying `KREDZONE_SIZE` to the assembly symbol or establishing the initial zeroed state. Misconfiguration would invalidate the ghost invariants.
- **Module: Logging side effects omitted** — Verified spec omits the error-path logging present in the shipped kernel module. Behavior divergence persists and could mask diagnostic regressions.

### Low
- **Module: Architecture fallback** — Unsupported pointer widths still default to 64-bit via cfg fallback, diverging from `size_of::<usize>()` on non-target hosts. This may hide mismatches during verification on non-target platforms.

## Positive Observations
- Abstract model and ghost wrappers remain clear about bounds and non-interference properties with supporting lemmas and tests.
- Bounds checking is verified and well-documented; trust assumptions are explicitly stated.

## Summary
No substantive fixes are present relative to the prior review: value-level correctness of store/load is still trusted (not verified), the ghost/load bridge still uses an unchecked assumption, and environmental assumptions (size/zeroing, logging parity, architecture fallback) remain unaddressed. Assurance level is unchanged; key functional properties depend on external assumptions rather than proof.
