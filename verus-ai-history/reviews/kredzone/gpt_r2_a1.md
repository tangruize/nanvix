# Review: kredzone (gpt-5.1-codex-max)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Function: store/load specs** — Postconditions only state index validity; they do not relate the stored/loaded value to memory or guarantee the error code matches `InvalidArgument`. The core behavior (write/read of the red-zone slot) is unverified, so functional correctness of the executable code is not captured. Suggested fix: Strengthen specs to require that on `Ok`, the value at `index` equals the argument (store) or returned value (load), and that errors are precisely `InvalidArgument`; align with the concrete volatile operations, possibly by modeling the region as a tracked array and proving equivalence for the non-volatile path while keeping a small trusted shim for volatile.
- **Function: load_with_ghost** — Uses an `assume` to equate the returned value with ghost state (trust assumption T2), leaving read-after-write dependent on an unchecked axiom rather than proof. This weakens soundness and shifts correctness outside the verifier. Suggested fix: Replace the `assume` with a verified link to a modeled memory or isolate the volatile read in a minimal trusted helper whose contract matches the ghost model, then prove the wrapper satisfies read-after-write against that helper.

### Medium
- **Module: Trust boundary size assumption (T1/T4)** — The size and zero-initialization of the external `kredzone` are only documented assumptions; no proof ties `KREDZONE_SIZE` to the actual assembly symbol or establishes zeroed state. Misconfiguration would invalidate the ghost invariants. Suggested fix: Add a checked build-time assertion (e.g., linker script symbol size check or assembly test) and a runtime init routine that zeros the region once, then prove wrappers assuming that init was called.
- **Module: Logging side effects omitted** — The original code logs on bounds errors; the verified version omits logging entirely. This is a semantic gap between spec and implementation that could hide regressions in diagnostics. Suggested fix: Model logging as an allowed side effect or add a postcondition that error paths may emit a log, keeping the behavior aligned.

### Low
- **Module: Architecture fallback** — `ENTRY_SIZE` defaults to 64-bit for unsupported pointer widths, diverging from the runtime `mem::size_of::<usize>()` used in the kernel. While Nanvix targets x86-32, this fallback could hide mismatches during verification on non-target hosts. Suggested fix: Make unsupported architectures a verification error instead of defaulting, or assert equality with `size_of::<usize>()` via a small trusted lemma tied to the host target.

## Positive Observations
- Abstract model clearly states bounds and non-interference properties with accompanying lemmas and tests.
- Ghost wrappers capture well-formedness and propagate bounds reasoning, aligning with the bounds checks in the implementation.
- Verification run passes with no errors, and assumptions are explicitly documented.

## Summary
The verification covers the API surface but leaves core functionality (actual store/load behavior) trusted via `external_body` and an explicit `assume`, so read-after-write and value preservation are not machine-checked. Strengthening the specs to relate values to memory, minimizing trusted code for volatile access, and proving initialization/size linkage would raise assurance and eliminate the key trust gaps. Logging and architecture fallback differences should also be aligned to keep the verified specification semantically closer to the shipped kernel code.
