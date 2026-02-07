# Review: zombie_thread (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

None.

### High

- **Location:** `harvest()` in `zombie.rs` (exec, line 146–156)
- **Description:** Semantic divergence from original. The original `harvest()` takes `mut self` and calls `self.state.take_kernel_stack()` / `self.state.take_user_stack()`, which are `&mut self` methods that clear the fields via `Option::take()`. This means the original mutates the `ThreadState` before the `ZombieThread` is dropped — the stacks are moved out and the fields become `None`, so when `ThreadState::drop()` runs, the stacks are no longer owned by the state. The verified version reads `self.state.kernel_stack` / `self.state.user_stack` directly without clearing them (since `self` is consumed by value, Verus moves the fields). While the *returned values* are the same, the verified code does not model the take-then-drop sequence. This matters because the original's `Drop` impl for `ThreadState` checks for locked mutexes — if a future version also checked for stacks, the verification model would miss it. The postconditions are also weaker than they could be: they don't assert the stacks have been removed from `self.state`.
- **Suggested Fix:** Call `self.state.take_kernel_stack()` and `self.state.take_user_stack()` in `harvest()` instead of reading fields directly, to preserve the Option::take semantics. This would also verify that take_kernel_stack/take_user_stack maintain wf() during the harvest sequence.

### Medium

- **Location:** `wf()` in `zombie.spec.rs` (spec, line 92–94)
- **Description:** The well-formedness predicate for `ZombieThread` only requires `self.state.wf()` but does not constrain `status` in any way. In the original, `ExitStatus` is a bounded type (likely an enum or newtype wrapper). The spec models it as unbounded `int`, which is weaker than the original — any arbitrary integer is accepted. While this doesn't cause unsoundness (specs are weaker, not stronger), it means invalid exit statuses cannot be ruled out by `wf()`.
- **Suggested Fix:** If `ExitStatus` has known bounds (e.g., it wraps an `i32`), add a constraint like `self.status >= i32::MIN && self.status <= i32::MAX` to `wf()`, or document that the unbounded `int` is an intentional abstraction.

- **Location:** `from_state()` in `zombie.rs` (exec, line 88)
- **Description:** The original `from_state()` takes `Box<ThreadState>` and stores it directly. The verified version takes `ThreadState` by value. While the documentation explains this (`Box` is transparent), the heap allocation is a real resource in the kernel. A `Box` that is never deallocated would be a leak. The verification model cannot catch Box-related resource leaks.
- **Suggested Fix:** Document this as an explicit trust boundary item. Consider adding a comment in the trust boundary section noting that Box deallocation correctness is out of scope.

- **Location:** `harvest()` in `zombie.rs` (exec, line 146)
- **Description:** The original `harvest()` returns `(Option<KernelStack>, Option<UserStack>)` — these are real resource types that must be properly handled by the caller to avoid leaks. The verified version returns `(Option<int>, Option<int>)`. The verification cannot enforce that the caller properly disposes of the returned stack resources.
- **Suggested Fix:** This is inherent to the abstraction level. Document that resource lifecycle tracking of returned stacks is out of verification scope.

### Low

- **Location:** `zombie.proof.rs` (proof, lines 108–148)
- **Description:** Several proof lemmas are trivial tautologies that restate the spec definitions (e.g., `lemma_id_correct` proves `self.spec_id() == self.state.spec_id()` which is literally the definition of `spec_id()`; `lemma_harvest_returns_stacks` similarly restates spec definitions). While not harmful, these add no verification value — Verus's SMT solver proves them vacuously.
- **Suggested Fix:** Consider consolidating or removing purely tautological lemmas. Keep only lemmas that encode non-obvious properties or that serve as documented proof obligations for external callers.

- **Location:** `ZombieThread` struct in `zombie.rs` (exec, line 65)
- **Description:** Fields are `pub` for Verus proof ergonomics, as documented. The original has private fields with accessor methods. This means the verified model does not enforce the encapsulation invariant — any proof code can construct a `ZombieThread` directly, bypassing `from_state()` and its `wf()` precondition.
- **Suggested Fix:** This is a known Verus limitation. The documentation already notes it. No code change needed, but consider adding a `// INVARIANT: construction should only occur via from_state()` comment near the struct definition.

- **Location:** `Debug` trait impl — original `zombie.rs` line 31
- **Description:** The original `ZombieThread` derives `Debug`. The verified version does not model this. This is cosmetic and has no correctness impact, but it's worth noting for completeness.
- **Suggested Fix:** No action needed — `Debug` is a display trait with no semantic impact.

## Positive Observations

- **Complete function coverage.** All 6 methods from the original (`from_state`, `id`, `thread_state`, `thread_state_mut`, `harvest`, `status`) have verified counterparts. `thread_state_mut` is properly marked `#[verifier::external]` with detailed trust boundary documentation.
- **No assume/external_body in core logic.** The only `#[verifier::external]` is for `thread_state_mut()`, which genuinely cannot be expressed in Verus (returns `&mut T`). This is well-documented with explicit caller obligations.
- **Clean spec/proof/exec separation.** The three files have clear responsibilities: `zombie.spec.rs` defines view types and spec functions, `zombie.proof.rs` contains lemmas, and `zombie.rs` has the executable code.
- **Strong postconditions on `from_state()`.** The construction postconditions are comprehensive: they verify state view preservation, identity, status, stacks, user TDA, mutex accounting, drop safety, and well-formedness.
- **ThreadState dependency is well-verified.** The underlying `ThreadState` has thorough verification with Option::take semantics, mutex set consistency, and wf() preservation across all operations.
- **Verification passes cleanly.** All 21 verification conditions pass with 0 errors.
- **Thorough documentation.** The trust boundary, verification model, and abstraction choices are extensively documented in module-level comments.
- **Mutex accounting propagation.** The verification correctly tracks that zombie threads inherit their mutex accounting from the original thread state, preserving the drop-safety invariant.

## Summary

The zombie_thread verification is well-executed with comprehensive function coverage, clean separation of concerns, and no unjustified trust assumptions. The primary issue is a semantic divergence in `harvest()`: the verified version reads fields directly instead of calling `take_kernel_stack()`/`take_user_stack()`, which means it does not model the Option::take mutation sequence that the original performs before drop. This is a meaningful gap because it bypasses the verified ThreadState methods and their wf()-preservation guarantees. The abstraction of `ExitStatus` as unbounded `int` and `Box<ThreadState>` as bare `ThreadState` are reasonable simplifications but should be explicitly documented as trust boundary items. The proof lemmas are largely tautological and could be pruned. Overall, the verification provides good confidence in the core correctness properties of the ZombieThread type.
