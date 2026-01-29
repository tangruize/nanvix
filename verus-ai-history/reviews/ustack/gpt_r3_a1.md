# Review: ustack (gpt-5.1-codex-max)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- API equivalence drift in accessors: `base()`/`top()` in the verified module return raw `usize` instead of `PageAligned<VirtualAddress>` and rely on postconditions for alignment. Callers lose the original type-level guarantee unless they explicitly use the added `base_aligned()`/`top_aligned()` helpers, which are not direct replacements. Suggested Fix: Make the primary accessors return `PageAlignedAddr` (or mirror the original signature) and keep raw versions as helpers, or strengthen equivalence lemmas to show postconditions are fully substitutable for the original type-level contract at call sites.

### Low
- Constructor signature mismatch: The verified `new` returns `Result` with alignment/overflow preconditions, while the original constructor is infallible for `PageAligned<VirtualAddress>`. Although the ensures clause states `result.is_ok()` under the preconditions, clients in proofs still see a fallible API and must unwrap, which is not semantically identical. Suggested Fix: Expose an infallible constructor under the proven preconditions (or add a wrapper) so the verified API matches the original total behavior.
- Debug formatting not modeled: The original implements a custom `fmt::Debug`, but the verified version derives `Debug` without showing equivalence. Suggested Fix: Add a spec/ghost proof or comment acknowledging the difference, or match the original formatting to maintain coverage parity.

## Positive Observations
- Core safety properties are covered: alignment of base/top, fixed size, contiguity of pages, top>base, and overflow avoidance are all specified and proven without `assume` or `external_body` in the core module.
- Additional helper functions (`contains`, `page_index`, `initial_sp`, `has_room`) strengthen the verified safety story and are consistent with the stack model.
- Constants are internally validated and linked to CI to detect drift versus kernel configuration.

## Summary
Verification passes and captures key memory-safety properties with solid invariants, but there are API equivalence gaps: accessors drop type-level alignment, and the constructor is fallible where the original is total. Tightening the public interface to mirror the kernel types (or providing infallible wrappers with proofs) would bring the verified module to full equivalence. Debug formatting coverage is a minor gap.
