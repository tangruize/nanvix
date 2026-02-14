# Review: ready Spec Methodology (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

- None.

### High

- **`store_mutex_guard` requires exposes concrete field** (ready.rs:399): The precondition `old(self).state.locked_mutex_count < usize::MAX` directly accesses the concrete `state.locked_mutex_count` field via `self.field` instead of using the abstract view (`old(self)@.locked_mutex_count` or `old(self).spec_locked_mutex_count()`). Per methodology Step 3, public method specs must avoid `self.field` and use `self@.field` or spec functions instead. This leaks implementation details into the public contract.

- **`thread_state` ensures exposes concrete field** (ready.rs:339): The postcondition `result@ == self.state@` accesses `self.state` directly (a concrete field) rather than `self@.state` (the view's abstract field). Should be `result@ == self@.state` to route through the abstraction layer per methodology Step 3.

### Medium

- **`view()` is `open` not `closed`** (ready.spec.rs:228): The methodology recommends `pub closed spec fn view()`. The code justifies this with a comment that the Verus `View` trait requires `open spec fn`, which is a correct and valid justification. The comment at line 222-224 documents this well. No action needed, but noted for completeness.

- **Spec helper functions are `pub open` rather than placed on View type** (ready.spec.rs:84-141): The methodology Step 3 suggests that common spec subexpressions should be `pub open spec fn` on the View type (`ReadyThreadView`), not on the concrete type. Functions like `spec_id()`, `spec_admission_time()`, `spec_has_mutex()`, `spec_drop_safe()` are defined on `ReadyThread` directly as `pub open spec fn`. This is a mild deviation — they are transparent pass-throughs that don't leak implementation details beyond what the view already exposes, but moving them to `ReadyThreadView` would be more methodologically pure.

- **`wf()` used instead of `inv()`** (ready.spec.rs:139): The methodology Step 2 names the invariant function `inv()`. This module uses `wf()` instead. This is consistent with the rest of the codebase (ThreadState also uses `wf()`), so it's a deliberate convention, not a bug. The function is correctly `pub closed spec fn`.

### Low

- **Struct fields are `pub`** (ready.rs:109-114, 124-127, 137-142, 149-156): All struct fields in `ReadyThread`, `RunningThread`, `ZombieThread`, and `RunResult` are `pub`. The comment at line 106-108 justifies this for Verus proof ergonomics (spec access, direct construction in lemmas). This is a pragmatic trade-off, but it does weaken encapsulation. The methodology's intent of hiding internals via `closed` view/inv is partially undermined when fields are directly accessible.

- **Boundary models lack `#[auto]` on forall triggers** (ready.rs:306): The `from_state` ensures clause `forall|a: int| result.spec_has_mutex(a) == state@.has_mutex(a)` generates a low-confidence trigger warning from Verus. Adding `#![auto]` (as done in `run()` at line 475) would suppress the warning.

## External Body / Trust Boundary Assessment

Two `external_body` functions exist, both justified:

1. **`clock_now()`** (ready.rs:74-80): Models `clock::now()`. Postcondition `result >= 0` is minimal and reflects `SystemTime` non-negativity. Justified — monotonicity is outside scope.

2. **`exit_status_interrupted_value()`** (ready.rs:87-93): Bridges spec/exec for `EXIT_STATUS_INTERRUPTED()` constant. Postcondition ties it to the spec constant. Justified — `int` literals cannot be constructed in Verus exec code.

One `#[verifier::external]` function:

3. **`thread_state_mut()`** (ready.rs:554-557): Returns `&mut ThreadState` which Verus cannot express. Well-documented trust obligations with manual audit requirements. Justified — no Verus alternative exists. Verified forwarding methods (`set_interrupt_reason`, `store_mutex_guard`, `take_mutex_guard`) reduce reliance on this escape hatch.

No `assume` or `admit` statements found anywhere. ✓

## Verification Status

**PASSED**: 34 verified, 0 errors.

## Summary

The `ready` module demonstrates strong spec methodology adherence overall. View types use abstract types (`int`, `nat`, `Set<int>`, `Option<int>`), `wf()` is correctly `pub closed spec fn`, `view()` openness is justified by Verus trait constraints, and there are no unjustified `assume`/`admit` statements. Verification passes cleanly.

The two high-severity issues are both instances of public method specs accessing `self.state` (a concrete field) directly instead of routing through the view (`self@.state`). In `store_mutex_guard`, `old(self).state.locked_mutex_count` exposes the raw `usize` field in the public contract; this should use `old(self).spec_locked_mutex_count() < usize::MAX as nat` or similar. In `thread_state`, `self.state@` should be `self@.state`. These are correctness-neutral (they don't affect soundness) but violate the methodology's encapsulation principle that public specs should only reference `self@` and `inv()`/`wf()`.

The codebase convention of `wf()` instead of `inv()` is consistent and deliberate. The `pub` fields are a pragmatic Verus trade-off, well-documented. The external_body functions are minimal and well-justified. The proof file contains comprehensive lemmas covering construction, identity preservation, state transitions, and composition.
