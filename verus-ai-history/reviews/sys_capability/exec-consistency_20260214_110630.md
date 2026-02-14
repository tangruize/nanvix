# Review: sys_capability Exec Consistency (claude-opus-4.6)

## Grade: A-

## Files Reviewed

- **Original source:** `src/libs/sys/src/sys/pm/capability.rs`
- **Verified exec (libs):** `verus/split/libs/sys/sys/pm/capability.rs`
- **Verified spec (libs):** `verus/split/libs/sys/sys/pm/capability.spec.rs`
- **Verified proof (libs):** `verus/split/libs/sys/sys/pm/capability.proof.rs`
- **Consistency report:** `verus-ai-history/ast-consistency/sys_capability_20260214_110630_fix.md`

## Issues Found

### Critical

- None.

### Major

1. **Verification resolves to wrong module.** Running `./verus-ai/scripts/verify.sh sys_capability` resolves to `kernel::pm::sys::capability` (a different file at `verus/split/kernel/pm/sys/capability.rs`), not the libs version at `verus/split/libs/sys/sys/pm/capability.rs` that the consistency report targets. The kernel version has a **different architecture**: it uses `spec_discriminant()` (public open spec) instead of the libs version's `CapabilityView`-based methodology with private closed `spec_discriminant()`, `inv()`, and `View` trait. Crucially, the kernel version's `try_from` still delegates to `Self::try_from_u32(value)` — the exact pattern the libs fix report claims to have fixed. The consistency report says "Verification: PASS — 14 verified, 0 errors" but this result comes from verifying the *kernel* module, not the *libs* module being fixed. The libs version was never independently verified.

### Minor

1. **`PARSE_ERROR_MESSAGE` constant not in original source.** The original uses the string literal `"invalid capability"` directly. The verus version introduces a `pub const PARSE_ERROR_MESSAGE` and uses it in both `try_from` and `try_from_u32`. While runtime-equivalent, this is an undocumented structural addition — the consistency report only documents `try_from_u32` and `to_u32` as additions, not this constant. This is cosmetic since the value is identical, but completeness of the fix report is slightly impaired.

2. **Additional derive traits.** The original derives `Debug, Clone, Copy`. The verus version derives `Debug, Clone, Copy, PartialEq, Eq`. The consistency report does not document this addition, though it is mentioned in the kernel version's module-level doc comments. The libs version's header does not explicitly call this out as a verification addition (the kernel version does).

3. **`u32` literal suffixes.** The fix report correctly notes that `0u32` suffixes are required by Verus and semantically equivalent. This is sound.

## Detailed Assessment

### 1. Were all MISMATCH functions properly restored or equivalence documented?

**Yes.** The single reported mismatch — `try_from` delegating to `try_from_u32` instead of inlining the match — has been properly restored. The libs `try_from` (lines 148–171) now contains the inline `match value { 0u32 => ..., ... }` pattern matching the original source's structure. The fix report accurately describes this change.

### 2. Were MISSING functions added with proper verification?

**Partially.** The report states "Missing functions added: 0" which is correct — no functions from the original source were missing. The two verification auxiliaries (`try_from_u32` and `to_u32`) are properly documented as not present in the original. Both have appropriate postconditions. However, the `PARSE_ERROR_MESSAGE` constant addition is not documented in the fix report.

### 3. Are equivalence justifications sound?

**Yes, with caveat.** The justifications for keeping `try_from_u32` and `to_u32` are sound:
- `try_from_u32` provides a standalone verified conversion usable by proofs without trait dispatch — follows `pid.rs`/`tid.rs` pattern.
- `to_u32` enables round-trip proof `try_from_u32(cap.to_u32()) == Ok(cap)`.

Both are well-justified verification auxiliaries. The caveat is that the verification proving these auxiliaries correct was run against the *kernel* copy, not the *libs* copy.

### 4. Does the exec code faithfully represent the original source?

**Yes.** The exec code in `verus/split/libs/sys/sys/pm/capability.rs` faithfully represents the original:
- Same enum variants in the same order (ExceptionControl, InterruptControl, IoManagement, MemoryManagement, ProcessManagement).
- Same `TryFrom<u32>` implementation with identical match arms and error handling.
- Same error code (`ErrorCode::InvalidArgument`) and message (`"invalid capability"`).
- Additions (`try_from_u32`, `to_u32`, `PARSE_ERROR_MESSAGE`, `PartialEq`/`Eq`) are clearly verification auxiliaries and do not alter runtime behavior.

### 5. Does verification still pass?

**Unclear for the libs version.** The `./verus-ai/scripts/verify.sh sys_capability` command passes with 14 verified / 0 errors, but it verifies `kernel::pm::sys::capability`, not `libs::sys::sys::pm::capability`. The libs version has not been independently verified through this script. The two versions have different spec architectures (the kernel uses `pub open spec fn spec_discriminant` and `wf()`; the libs uses `closed spec fn spec_discriminant` with `CapabilityView`-based `inv()`), so passing one does not guarantee the other passes.

## Spec & Proof Quality (libs version)

The libs spec and proof files are well-structured:
- **Spec:** Follows methodology steps 1–3. `CapabilityView` uses `int`, `view()` is closed, `inv()` is `pub closed spec fn`, `spec_discriminant` is private. `from_discriminant` and `is_valid_discriminant` are properly open.
- **Proof:** 9 lemmas covering invariant satisfaction, discriminant uniqueness, mutual exclusivity, round-trip, inverse, view equality, bounds, and total coverage. All lemma bodies are empty (Verus auto-proves), indicating clean verification.
- **Trust boundary:** No `assume`, `admit`, or `external_body` found in any of the three files.

## Kernel vs Libs Divergence

The kernel version (`verus/split/kernel/pm/sys/capability.rs`) diverges from the libs version in a significant way: its `try_from` implementation (line 163) still delegates to `Self::try_from_u32(value)` rather than inlining the match — the very pattern the libs fix was supposed to correct. This suggests either:
- The fix was intentionally scoped to the libs copy only, or
- The kernel copy was overlooked.

This does not affect the libs review grade but is noted for completeness.

## Summary

The consistency fix for the libs version of `sys_capability` is well-executed: the `try_from` mismatch has been properly restored to inline match form, verification auxiliaries are justified and documented, and the exec code faithfully mirrors the original source. The grade is A- rather than A because (1) the verification command resolves to the wrong module (kernel, not libs), so the libs version's verification is unconfirmed through the standard script, and (2) the `PARSE_ERROR_MESSAGE` constant and `PartialEq`/`Eq` derive additions are not documented in the consistency fix report. These are minor completeness gaps that do not affect correctness.
