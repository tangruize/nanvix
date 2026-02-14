# Review: interrupted Spec Methodology (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

- None.

### High

- **Spec functions defined on `InterruptedThread` instead of `InterruptedThreadView` (methodology Step 3 violation):** The methodology states: "Don't write any further `pub` specification functions in `impl MyType` beyond `inv` and `view`." However, `InterruptedThread` defines 10 `pub open spec fn` functions (`spec_id`, `spec_reason`, `spec_valid_reason`, `spec_is_killed`, `spec_is_timed_out`, `spec_state_interrupt_reason`, `spec_locked_mutex_count`, `spec_drop_safe`, `spec_has_mutex`, `spec_kernel_stack`, `spec_user_stack`) directly on the concrete type. These access `self.state` and `self.reason` (concrete fields) rather than going through `self@`. Per methodology, common spec expressions used in public method contracts should be `pub open spec fn` on `InterruptedThreadView`, referenced as `self@.some_fn()`. This leaks implementation details into the specification layer. Affected: `interrupted.spec.rs` lines 93–159.

- **`thread_state()` postcondition references `self.state@` directly (methodology Step 3 violation):** At `interrupted.rs` line 175, the ensures clause says `result@ == self.state@`. This directly accesses the concrete field `self.state` rather than using `self@.state` (the view's field). The methodology says: "never talk directly about the fields of a Self parameter. For instance, don't say `self.x`; instead say things like `self@.y`."

### Medium

- **Missing `wf()` preconditions on `id()` and `thread_state()` (methodology Step 3 violation):** The methodology states: "for any input `Self` parameters, one of the preconditions should be that `inv` holds." The `id()` method (line 159) and `thread_state()` method (line 172) lack `requires self.wf()`. While these are simple getters that happen to work without the precondition, the methodology requires it for all public methods taking `Self` parameters. This also means callers aren't contractually required to maintain well-formedness when calling these methods, weakening the invariant discipline.

### Low

- **`view()` is `open` instead of `closed` (justified but noted):** Both `View` trait impls use `open spec fn view()`. This is correctly justified in comments (Verus `View` trait requires it). Abstraction is preserved because the View types only expose abstract types. No action needed.

- **`thread_state_mut()` uses `#[verifier::external]` (justified trust boundary):** The function returning `&mut ThreadState` cannot be verified because Verus doesn't support `&mut T` return types. The trust boundary is thoroughly documented with intended postconditions, known call sites, and migration plan. No action needed, but this remains an unverified gap.

## Summary

The `interrupted` module demonstrates strong verification work with 21 verified properties and zero errors. The View types correctly use abstract types (`int`, `ThreadStateView`), `wf()` is properly `pub closed spec fn`, there are no `assume`/`admit` statements, and the single `#[verifier::external]` annotation is well-justified with comprehensive trust boundary documentation.

The main methodology gap is that spec functions are defined on the concrete `InterruptedThread` type rather than on `InterruptedThreadView`, causing public method contracts to indirectly reference concrete fields through these spec functions. The methodology recommends placing reusable spec expressions on the View type and using `self@` in public contracts to fully decouple the specification from implementation details. Additionally, `id()` and `thread_state()` are missing `wf()` preconditions, and `thread_state()` directly references `self.state@` in its postcondition.

These are structural methodology issues rather than correctness bugs — the verified properties are sound and the proofs are clean. Addressing them would improve abstraction discipline and alignment with the methodology guidelines.
