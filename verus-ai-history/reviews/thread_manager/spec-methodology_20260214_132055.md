# Review: thread_manager Spec Methodology (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

- None.

### High

1. **`view()` is `open spec fn`, not `pub closed spec fn` (thread_manager.spec.rs:120,130)**
   The guidelines (Step 1) require `pub closed spec fn view(&self) -> MyTypeView` so that
   users cannot see the internals of the implementation through the view mapping.
   Both `ThreadManager::view()` and `ReadyThread::view()` are declared as
   `open spec fn view(...)` via the `View` trait impl. This exposes the internal
   mapping from concrete fields to abstract fields.

   **Mitigation:** In Verus, the `View` trait's `view()` method signature is
   `open spec fn view(&self) -> Self::V`, which Verus requires to be `open` when
   implementing the trait. This is a Verus framework constraint, not a spec design
   flaw. The practical impact is low because the View types already use abstract
   types (`int`, `ThreadStateView`), so the exposed mapping reveals minimal
   internal structure. Still, this is a deviation from the guideline's letter.

2. **`wf()` and `spec_next_id()` are `pub open` instead of `pub closed` (thread_manager.spec.rs:48,62)**
   The guidelines (Step 2) specify that `inv()`/`wf()` should be `pub closed spec fn`
   to hide implementation internals. Both `ThreadManager::wf()` and
   `ThreadManager::spec_next_id()` are `pub open spec fn`, meaning callers can
   see that `wf()` checks `self.next_id.spec_value() >= 1` and that
   `spec_next_id()` delegates to `self.next_id.spec_value()`. This leaks
   implementation details.

   Similarly, all ReadyThread spec functions (lines 73–110) are `pub open spec fn`.

3. **Public method `create_thread` uses `self.next_id.value` directly in requires (thread_manager.rs:365)**
   The `create_thread` requires clause contains `old(self).next_id.value < i32::MAX`,
   which directly accesses the concrete field `next_id.value` rather than using
   the abstract view `old(self)@.next_id` or a spec function. The guidelines (Step 3)
   say public method specs should never talk directly about fields of a `Self`
   parameter—use `self@.field` instead.

   **Suggested fix:** Replace with `old(self).spec_next_id() < i32::MAX as int`
   or `old(self)@.next_id < i32::MAX as int`.

### Medium

4. **Struct fields are `pub` instead of private (thread_manager.rs:100,109)**
   Both `ReadyThread::state` and `ThreadManager::next_id` are declared `pub`.
   The Nanvix coding standards require struct fields to be private with
   getter/setter access. In the verification model context, public fields
   simplify proof lemmas that directly construct struct values (e.g.,
   `ThreadManager { next_id: ThreadIdentifier { value: 1 } }` in proofs),
   but this deviates from the encapsulation principle.

5. **Proof lemmas access `self.next_id.value` directly (thread_manager.proof.rs:72,76,91,95,125,129,142)**
   Multiple proof lemmas reference `self.next_id.value` directly rather than
   going through spec functions. While proof code (private, internal) is
   permitted to access fields per Step 4 of the guidelines, this creates
   coupling to the concrete representation. If `ThreadManager`'s internal
   representation changed, all proof lemmas would need updating.

### Low

6. **No `inv()` function — uses `wf()` naming instead**
   The guidelines reference `inv()` as the canonical name for the invariant
   function. The code uses `wf()` (well-formedness). This is a naming
   preference difference, not a functional issue. The `wf()` name is used
   consistently throughout the Nanvix verification codebase.

7. **View types use `ThreadStateView` (abstract) — correct**
   `ThreadManagerView` uses `int` for `next_id` and `ReadyThreadView` uses
   `ThreadStateView` for `state`. Both correctly use abstract types rather
   than concrete ones. ✅

8. **No `assume`, `admit`, or unjustified `external_body` in thread_manager files**
   The thread_manager.rs, thread_manager.spec.rs, and thread_manager.proof.rs
   files contain no `assume`, `admit`, or `#[verifier::external_body]`
   annotations. The `external_body` annotations found in the broader
   `thread/` directory (ready.rs, sleeping.rs) are in sibling modules with
   documented justifications (HAL boundary types, `clock_now()`). ✅

9. **Public methods properly require/ensure `wf()` for self parameters** ✅
   - `create_thread`: requires `old(self).wf()`, ensures `self.wf()`.
   - `new` (private): ensures `result.1.wf()`.
   - `init` (public): ensures `result.1.wf()`.
   - All return ReadyThread with `result.wf()` ensured.

10. **Verification passes: 22 verified, 0 errors** ✅

## Summary

The thread_manager verification model is well-structured with thorough proofs
covering ID uniqueness, monotonicity, well-formedness preservation, and
dispatch semantics. Verification passes cleanly (22/0). The View types correctly
use abstract types (`int`, `ThreadStateView`), and there are no remaining
`assume`/`admit`/unjustified `external_body` annotations.

The primary deviations from the spec methodology guidelines are:
1. `wf()` and spec functions are `pub open` rather than `pub closed`, exposing
   implementation details to callers.
2. One public method (`create_thread`) accesses `self.next_id.value` directly
   in its requires clause instead of using the abstract view.
3. `view()` is `open` due to Verus `View` trait constraints (unavoidable).

These are methodology adherence issues rather than correctness issues — the
verified properties are sound and comprehensive. Fixing issues #2 and #3
(making spec fns `closed` and using `self@.next_id` in public specs) would
bring the code into full compliance with the guidelines while maintaining
verification success.
