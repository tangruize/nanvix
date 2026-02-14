# Review: kstack Exec Consistency (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

- None.

### Major

- None.

### Minor

1. **`base()` doc comment inaccuracy**: The original source comment on `base()` (line 88) states "As stacks grow downwards, the base address is the highest address of the stack." This is incorrect — `base` is the *lowest* address (the bottom of the stack memory region). The Verus version correctly describes it as "the lowest virtual address of the stack" (line 286). This is not a consistency defect but worth noting: the Verus version actually *improves* the documentation.

2. **`size()` visibility mismatch**: The original `size()` is `fn size(&self)` (private), while the Verus version is `pub fn size(&self)` (public). Same applies to `base()` which is private in the original but public in the Verus version. This widens the API surface without justification. The consistency report does not document these visibility changes.

3. **`has_room` subtle semantics**: The `has_room` method accepts `current_sp` at the `top()` boundary (via the `||` in the precondition), but `top()` is one-past-the-end and not a valid byte within the stack. While this is intentionally modeling the initial stack pointer position, the postcondition `current_sp - growth >= base` uses integer subtraction that could go negative in the spec (not the exec). The exec implementation `current_sp - self.base_addr >= growth` is safe because the precondition guarantees `current_sp >= base_addr`, avoiding underflow. This is correct but deserves a comment.

4. **Missing `unwrap()` audit in original**: The original `base()` calls `.unwrap()` on `PageAligned::from_raw_value()`, and `top()` also calls `.unwrap()`. The consistency report correctly notes these are replaced by proof-level alignment guarantees, but does not flag that the original violates the Nanvix coding standard ("Do not use `unwrap()`"). This is informational only.

### Observations

1. **Struct abstraction is well-justified**: The replacement of `Vec<KernelPage>` with `(base_addr, num_pages)` is a clean abstraction that captures the observable state. The equivalence argument — that all accessors depend only on `(kpages[0].base(), kpages.len())` — is sound.

2. **`new()` precondition/postcondition design is sound**: Encoding allocator guarantees as preconditions is the correct approach for modular verification. The postcondition proving `result.is_ok()` under preconditions establishes liveness.

3. **`size()` generalization is acceptable**: Parameterizing by `num_pages` instead of hardcoding `KSTACK_SIZE` is a reasonable generalization. The constant `DEFAULT_KSTACK_PAGES = 8` ties it back to the original configuration.

4. **Verification helpers are appropriate**: `contains`, `page_index`, `initial_sp`, and `has_room` are natural operations on a stack abstraction. They do not alter the original API semantics and enable useful proof obligations.

5. **Proof quality is good**: The lemmas (`lemma_page_aligned_add`, `lemma_pages_disjoint`, `lemma_page_in_bounds`) establish meaningful structural properties beyond what the exec code requires, demonstrating thorough reasoning about the abstraction.

6. **`Drop` omission is correctly justified**: Since the Verus struct owns no resources (just two `usize` fields), there is nothing to drop. Resource lifecycle verification belongs to the allocator module.

7. **`fmt::Debug` via derive is acceptable**: Using `#[derive(Debug)]` instead of a custom `fmt::Debug` impl is a pragmatic choice that preserves debug output capability.

## Consistency Checklist

| Criterion | Status | Notes |
|-----------|--------|-------|
| MISMATCH functions restored or documented | ✅ PASS | All 4 mismatches (`KernelStack` struct, `new`, `size`, `base`, `top`) documented with sound equivalence reasoning |
| MISSING functions added or documented | ✅ PASS | `fmt` and `drop` omissions justified (Verus limitations) |
| Equivalence justifications sound | ✅ PASS | Core argument (observable state = base + num_pages) is correct |
| Exec faithfully represents original | ✅ PASS | All original accessors preserved with equivalent semantics |
| Verification passes | ✅ PASS | 18 verified, 0 errors |
| Visibility matches original | ⚠️ MINOR | `size()` and `base()` changed from private to public |

## Summary

The kstack exec consistency fix is well-executed. The core abstraction decision — replacing `Vec<KernelPage>` with `(base_addr, num_pages)` — is sound and well-documented. All original functions are either faithfully represented or have justified omissions due to Verus limitations. The equivalence arguments in the consistency report are thorough and correct. The verification helpers are appropriate additions that strengthen the verified properties. The only gaps are minor: undocumented visibility changes on `size()` and `base()`, and the `has_room` method could benefit from a brief comment about why `current_sp == top()` is valid in the precondition. Overall, this is a high-quality consistency fix that meets the review criteria.
