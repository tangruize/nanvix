# Review: zombie_thread Spec Methodology (claude-opus-4.6)

## Grade: B+

## Issues Found

### Critical
- None.

### High

1. **`thread_state()` postcondition references `self.state@` (raw field access).**
   - File: `zombie.rs:133`
   - `result@ == self.state@` directly accesses the private field `state` rather than going through the view abstraction.
   - Per methodology Step 3: "never talk directly about the fields of a `Self` parameter. For instance, don't say `self.x`; instead say things like `self@.y`."
   - Fix: Use `result@ == self@.state` (accessing the `state` field of `ZombieThreadView`).

2. **`from_state()` postcondition references `result.state@` (raw field access).**
   - File: `zombie.rs:97`
   - `result.state@ == state@` accesses the concrete struct field `state` of the returned `ZombieThread`.
   - Fix: Use `result@.state == state@` (accessing via the view type).

3. **`id()` and `thread_state()` missing `self.wf()` precondition.**
   - File: `zombie.rs:117-122`, `zombie.rs:130-136`
   - Per methodology Step 3: "for any input `Self` parameters, one of the preconditions should be that `inv` holds."
   - Both `id()` and `thread_state()` take `&self` but have no `requires self.wf()`.
   - Similarly, `status()` at line 174 lacks `requires self.wf()`.

4. **Multiple `pub open spec fn` helpers on `ZombieThread` beyond `view()`.**
   - File: `zombie.spec.rs:46-89`
   - The methodology says (Step 3): "Don't write any further `pub` specification functions in `impl MyType` beyond `inv` and `view`. Any other specification functions in `impl MyType` should be private."
   - There are 9 `pub open spec fn` helpers (`spec_id`, `spec_status`, `spec_locked_mutex_count`, `spec_has_mutex`, `spec_drop_safe`, `spec_kernel_stack`, `spec_user_stack`, `spec_user_tda`, `spec_is_interrupted`).
   - These should either be moved to `ZombieThreadView` (as `pub open spec fn` on the view type, per Step 3's guidance on repeated subexpressions), or made private to `ZombieThread`.

### Medium

5. **`view()` is `open spec fn` — justified but worth noting.**
   - File: `zombie.spec.rs:117`
   - The comment at line 110-113 correctly explains this is a Verus `View` trait constraint. Methodology says `pub closed spec fn`, but the trait mandates `open`. This is properly documented.

6. **Naming: methodology says `inv()`, implementation uses `wf()`.**
   - File: `zombie.spec.rs:101`
   - The methodology document consistently refers to `inv()`. The implementation uses `wf()`. This is a cosmetic deviation; `wf()` is used consistently across all Nanvix verified modules, so it functions as a project-wide convention. No change needed, but worth documenting the convention.

7. **No `ZombieThreadView` spec helper functions.**
   - The methodology Step 3 says: "You may find yourself repeatedly writing the same subexpressions... create a `pub open spec fn` in `MyTypeView` for each such common expression."
   - The 9 helpers on `ZombieThread` (noted in issue #4) are strong candidates for relocation to `ZombieThreadView`, which would allow public method specs to use `self@.spec_id()` etc., cleanly following the methodology pattern.

### Low

8. **`#[verifier::external]` on `thread_state_mut()` is justified and well-documented.**
   - File: `zombie.rs:209-211`
   - The trust boundary documentation is thorough, listing the intended postconditions and preservation obligations. No action needed.

9. **No `assume` or `admit` found anywhere in the verified code.** Clean.

10. **Verification passes: 17 verified, 0 errors.** All proofs check out.

## Summary

The zombie_thread specification is well-structured with strong proof coverage and no trust gaps (no assume/admit, the single `#[verifier::external]` is justified). The `ZombieThreadView` uses abstract types correctly (`int`, `Option<int>`, `ThreadStateView`), and `wf()` is properly `pub closed spec fn`.

The main methodology gaps are: (1) public method specs reference concrete struct fields (`self.state@`, `result.state@`) instead of view-based accessors (`self@.state`, `result@.state`), violating Step 3's encapsulation rule; (2) three public methods (`id()`, `thread_state()`, `status()`) lack `requires self.wf()` preconditions; and (3) nine `pub open spec fn` helpers are defined directly on `ZombieThread` rather than being placed on `ZombieThreadView` or made private, which contradicts the methodology's guidance to limit public spec functions on the concrete type to `inv`/`wf` and `view`. These are structural/encapsulation issues rather than soundness bugs — the proofs are correct and complete.
