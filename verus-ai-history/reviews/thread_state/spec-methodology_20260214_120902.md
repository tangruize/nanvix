# Review: thread_state Spec Methodology (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

- None.

### High

- **`store_mutex_guard` requires clause uses `self.locked_mutex_count` directly (line 323).**
  The `requires` clause `old(self).locked_mutex_count < usize::MAX` accesses the concrete struct field directly instead of using the spec accessor `old(self).spec_locked_mutex_count()`. Per the guidelines ("never talk directly about the fields of a `Self` parameter"), this should be `old(self).spec_locked_mutex_count() < usize::MAX as nat` or an equivalent abstraction. The ensures clauses correctly use `self.spec_locked_mutex_count()`, making this inconsistent.

- **`view()` is `open spec fn` instead of `pub closed spec fn`.**
  The guidelines specify `view()` should be `pub closed spec fn`. The code includes a comment (line 144) justifying this as required by the Verus `View` trait. This is a known Verus constraint and the justification is documented, but it means implementation details of `ThreadState` leak through the view function. The impact is mitigated because the spec functions (e.g., `spec_id()`, `spec_kernel_stack()`) that callers should use are `pub open spec fn`, and `wf()` is correctly `pub closed spec fn`.

### Medium

- **Spec functions are `pub open` rather than being placed on `ThreadStateView`.**
  The guidelines suggest: "create a `pub open spec fn` in `MyTypeView` for each such common expression." Currently, all spec helper functions (`spec_id`, `spec_kernel_stack`, `spec_has_mutex`, etc.) are defined on `ThreadState` rather than on `ThreadStateView`. This means callers use `self.spec_id()` instead of `self@.id` in specifications. While functionally equivalent since the functions correctly abstract the fields, the guideline preference is to put common spec expressions on the View type and have public method specs use `self@.field` notation. However, the current approach is internally consistent and all public method specs do use spec functions rather than raw fields (except the one noted above).

- **Struct fields are all `pub`.**
  The `ThreadState` struct has all fields marked `pub` (lines 111–130). The original Rust source has private fields. While Verus may require `pub` fields for proof access, this weakens encapsulation. Consider whether `pub(crate)` or proof-only access could limit exposure.

### Low

- **Loop invariants in `take_mutex_guard` use direct field access (lines 407–417).**
  The `while` loop invariants reference `self.locked_mutex_set@`, `self.id`, `self.kernel_stack`, etc. directly. These are internal implementation details used within a method body, which is acceptable for private/internal reasoning per the guidelines ("no hiding is necessary" for private methods). However, since `take_mutex_guard` is `pub`, the loop is technically part of a public method's implementation — the invariants are not part of the public contract (only `requires`/`ensures` are), so this is acceptable.

- **`ThreadStateView` has `locked_mutex_count: nat` as a separate field from `locked_mutex_set: Set<int>`.**
  Since the View is an abstraction, the count is redundant with `locked_mutex_set.len()`. The View could omit `locked_mutex_count` since it can be derived from the set. This is a minor abstraction concern — the count is present because the exec-level model separates them, but the View should ideally be minimal.

## Verification

- **47 verified, 0 errors.** Verification passes cleanly.
- **No `assume`, `admit`, or unjustified `external_body`** found in the state files.
- Trust assumptions (T1: no double-locking, T2: release-what-you-hold) are clearly documented and justified.

## Summary

The `thread_state` verification is well-structured and thorough. The View type correctly uses abstract types (`int`, `Set<int>`, `Option<int>`, `nat`) rather than concrete types (`usize`, `Vec<u64>`). The `wf()` predicate is properly `pub closed spec fn` and is consistently required/ensured on all public mutating methods. The `view()` being `open` is justified by Verus trait requirements and documented. Public method specs correctly use spec accessors rather than direct field access in all `ensures` clauses, with one exception in the `requires` of `store_mutex_guard` where `locked_mutex_count` is accessed directly. The proof lemmas are comprehensive, covering construction, ID immutability, Option semantics, mutex roundtrips, drop safety, and well-formedness preservation. No `assume`/`admit`/`external_body` are present. Overall, this is a strong verification with minor methodology deviations.
