# Review: kstack (claude-opus-4.5) - Iteration 4

## Grade: A-

## Summary
No code changes since Iteration 3. Verification still passes (18 verified, 0 errors) with no `assume`, `trusted`, or `external_body`. Documentation continues to scope allocator interaction and Drop as out-of-scope; `DEFAULT_KSTACK_PAGES` remains 8. No new issues observed.

## Previous Issues Status
- **Allocator abstraction (`new` signature)**: Documented out-of-scope with rationale (lines 42-52). Modularization acceptable.
- **Drop / cleanup**: Documented out-of-scope (lines 54-62); lifecycle still unmodeled. A verified `release()` would strengthen coverage.
- **Stack size vs config**: Addressed; parameterized docs plus `DEFAULT_KSTACK_PAGES = 8`.
- **Return type abstraction**: Documented; alignment proven in postconditions.
- **Helper methods**: Documented as verification helpers.
- **Debug formatting**: Cosmetic; acceptable.

## Remaining Issues (Minor)
1. **Unused lemmas** (`lemma_well_formed_has_aligned_top`, `lemma_pages_disjoint`, `lemma_page_in_bounds`) remain unused; minor proof overhead.
2. **PAGE_SIZE source**: Locally defined, not imported from arch; small consistency risk if arch changes.
3. **No `release()` alternative**: Lifecycle gap remains; adding a verified release path would close it without Drop support.

## Verification
- Command: `verus --crate-type lib lib.rs --verify-module kstack`
- Result: 18 verified, 0 errors.

## Conclusion
Sound within declared scope; only minor/documentary items remain. Grade stays **A-**.
