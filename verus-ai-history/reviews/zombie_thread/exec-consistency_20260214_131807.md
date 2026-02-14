# Review: zombie_thread Exec Consistency (claude-opus-4.6)

## Grade: A

## Files Reviewed

- Original: `src/kernel/src/pm/thread/zombie.rs`
- Exec: `verus/split/kernel/pm/thread/zombie.rs`
- Spec: `verus/split/kernel/pm/thread/zombie.spec.rs`
- Proof: `verus/split/kernel/pm/thread/zombie.proof.rs`
- Consistency report: `verus-ai-history/ast-consistency/zombie_thread_20260214_131807_fix.md`

## Verification Status

**PASS**: 17 verified, 0 errors (9s).

## Function-by-Function Comparison

### 1. `from_state` — ✅ Equivalent

| Aspect | Original | Verus |
|--------|----------|-------|
| Visibility | `pub(super)` | `pub` |
| Param `state` | `Box<ThreadState>` | `ThreadState` |
| Param `status` | `ExitStatus` | `int` |
| Return | `Self` | `ZombieThread` (with ensures) |
| Body | `Self { status, state }` | `ZombieThread { status: status, state: state }` |

Field shorthand vs explicit assignment — semantically identical. Type substitutions (`Box<T>` → `T`, `ExitStatus` → `int`) are documented modeling decisions. Ghost `proof { reveal(ZombieThread::wf); }` has no runtime effect. Visibility widened for Verus module structure. **Sound equivalence.**

### 2. `id` — ✅ Equivalent

Body is identical: `self.state.id()`. Named return syntax and `requires`/`ensures` are ghost annotations only. **Exact match.**

### 3. `thread_state` — ✅ Equivalent

Body is identical: `&self.state`. The original returns `&ThreadState` through `Box` deref; Verus returns `&ThreadState` directly. Consistent with `Box<T>` → `T` modeling. **Exact match.**

### 4. `thread_state_mut` — ✅ Handled via `#[verifier::external]`

Body is identical: `&mut self.state`. Placed outside `verus!` block with `#[verifier::external]` because Verus cannot express `&mut T` return types. Trust boundary is well-documented with explicit caller obligations (preserve `wf()`, `spec_id()`, `spec_status()`). **Appropriate handling.**

### 5. `harvest` — ✅ Equivalent

| Aspect | Original | Verus |
|--------|----------|-------|
| Self | `mut self` | `self` (then `let mut state = self.state`) |
| Return | `(Option<KernelStack>, Option<UserStack>)` | `(Option<int>, Option<int>)` |
| Body | `(self.state.take_kernel_stack(), self.state.take_user_stack())` | Sequential let-bindings then tuple |

Original uses `mut self` with direct field access; Verus moves `self.state` into a local `let mut state`, then calls `take_kernel_stack()` / `take_user_stack()` sequentially. Both consume ownership and produce the same result. Expression decomposition into let-bindings is semantically identical. Return type uses documented modeling abstractions. **Sound equivalence.**

### 6. `status` — ✅ Equivalent

Body is identical: `self.status`. Return type `ExitStatus` → `int` is documented modeling. **Exact match.**

### 7. `#[derive(Debug)]` — ⚠️ Not modeled

The original derives `Debug`. The Verus version does not. This is expected — Verus does not support derive macros in the same way, and `Debug` has no semantic effect on correctness. **Acceptable omission**, but the consistency report does not mention it.

## Criteria Assessment

### 1. Were all MISMATCH functions properly restored or equivalence documented?

**Yes.** The consistency report documents 5 equivalences for 5 functions. All are accurately characterized. The type substitutions (`Box<ThreadState>` → `ThreadState`, `ExitStatus` → `int`, `KernelStack`/`UserStack` → `int`) are consistent across all functions and documented in the module header.

### 2. Were MISSING functions added with proper verification?

**Yes.** `thread_state_mut` was the only function that could not be verified within `verus!` and is correctly handled via `#[verifier::external]` with comprehensive trust boundary documentation. No other functions are missing.

### 3. Are equivalence justifications sound?

**Yes.** Each justification correctly identifies the differences (type modeling, visibility, Verus syntax) and explains why they preserve semantic equivalence. The `harvest` decomposition analysis is accurate — move-then-mutate vs `mut self` direct access are equivalent ownership patterns.

### 4. Does the exec code now faithfully represent the original source?

**Yes.** All 6 original functions are present. The exec bodies match the original logic. All differences are attributable to documented modeling decisions or Verus syntax requirements. The struct fields are `pub` (vs original private) — documented as necessary for proof ergonomics with an invariant note that construction should go through `from_state()`.

### 5. Does verification still pass?

**Yes.** 17 verified, 0 errors.

## Issues Found

### Critical

- None.

### Minor

1. **`#[derive(Debug)]` omission undocumented.** The original struct derives `Debug`; the Verus version does not. While this has no correctness impact, the consistency report should note it for completeness.

2. **Struct field visibility change undocumented in consistency report table.** The consistency report's "Struct: `ZombieThread`" table documents the type changes but does not explicitly call out that `pub` visibility is a change from the original's private fields. The module-level documentation in `zombie.rs` does document this (`Fields are pub for Verus proof ergonomics`), but the consistency report table would benefit from a visibility column.

## Spec and Proof Quality (Bonus Assessment)

- **Spec (`zombie.spec.rs`):** Well-structured with proper `ZombieThreadView` type, dual spec functions (on view and concrete type), and `closed` `wf()`. The `wf()` predicate correctly delegates to `state.wf()`.
- **Proof (`zombie.proof.rs`):** 12 lemmas covering construction, identity, status, drop safety, mutex accounting, stacks, view equality, and composite properties. Tautological lemmas were consolidated per prior review feedback — good sign of iterative quality improvement.
- **Trust boundary documentation:** Excellent. The `thread_state_mut` trust boundary is thorough with explicit postcondition obligations for callers.

## Summary

The exec consistency fix is thorough and correct. All 6 original functions are faithfully represented in the Verus version with appropriate modeling abstractions. The 5 documented equivalences are sound — differences stem exclusively from Verus syntax requirements and the documented type modeling decisions (`Box<T>` → `T`, `ExitStatus` → `int`, stack types → `int`). `thread_state_mut` is correctly placed outside the verification boundary with comprehensive trust documentation. Verification passes cleanly (17/0). The only gaps are cosmetic: the consistency report omits mention of the `Debug` derive and could more explicitly document the field visibility change. Grade: **A** — production-quality consistency fix with minor documentation gaps.
