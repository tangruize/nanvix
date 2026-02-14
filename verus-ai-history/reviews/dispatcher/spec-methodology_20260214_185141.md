# Review: dispatcher Spec Methodology (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- **Public struct fields expose implementation details.** `DispatchResult`, `DispatchArgs`, `SleepError`, `SleepableOutcome`, `FallibleOutcome`, and `ScoreboardDispatchOutcome` all have `pub` fields (e.g., `pub is_success: bool`, `pub value: i64`). The guidelines state "Member fields in `struct`s must be private and accessed via getter/setter methods." The comment on line 148 explains "Fields are `pub` because Verus requires public fields for spec-level access" — this is a known Verus limitation and is documented, but it still allows callers to bypass the View abstraction and access concrete fields directly. Public method `ensures` clauses (lines 261, 280, 299, 325, 502, 842, 891, 931–954, 1048–1060) reference `result.is_success`, `result.value`, `args.number` etc. directly instead of using `result@.is_success`, `result@.value`, `args@.number`. Per the guidelines (Step 3): "never talk directly about the fields of a Self parameter. For instance, don't say `self.x`; instead say things like `self@.y`." While Verus may require `pub` fields, the specs should still route through the View types for consistency.

- **`SleepError::spec_kind()` and `spec_error_code()` are `pub open spec fn` — should be `pub closed`.** (dispatcher.spec.rs:447–453) These functions directly expose `self.kind` and `self.error_code as int`. Per the guidelines (Step 1): view() and related spec accessors should be `pub closed spec fn` to hide implementation internals. Making these `open` allows callers to see that `spec_kind()` returns `self.kind` directly, leaking the representation. The `wf()` function at line 457 correctly uses `pub closed spec fn`.

### Medium
- **View types use `u32` / `bool` instead of abstract types in some places.** `DispatchArgsView` uses `nat` (good) but the constants like `KCALL_GET_PID()` return `u32` (concrete type), and `DispatchResultView.is_success` uses `bool` rather than an abstract success/error enum. `DispatchResultView.value` uses `int` (good). The use of `u32` for constants is pragmatic since the underlying KcallNumber is `#[repr(u32)]`, but it slightly departs from the "use abstract types" guideline. This is a minor deviation that is justified by the nature of the domain (hardware ABI constants).

- **No `inv()` on View types — only `wf()` on exec types.** The guidelines (Step 2) call for `pub closed spec fn inv(&self) -> bool` on the implementation type. The code uses `wf()` instead of `inv()` for the same purpose. This is a naming deviation — the semantics are correct (well-formedness predicates are closed, on the exec types, and used in pre/postconditions). The inconsistency is cosmetic but worth noting for cross-module consistency.

- **`SleepError` has no View type.** While `DispatchResult` and `DispatchArgs` properly have `DispatchResultView` and `DispatchArgsView` respectively, `SleepError`, `SleepableOutcome`, `FallibleOutcome`, and `ScoreboardDispatchOutcome` lack View types. These are primarily internal/boundary types, so this is less critical, but `SleepError` in particular is used in public-facing specs (`handle_sleep_error` is `pub fn`). The `spec_kind()` and `spec_error_code()` accessors serve as a partial substitute but don't follow the formal View pattern.

### Low
- **`open spec fn view()` in View trait implementations.** (dispatcher.spec.rs:405, 440) The View trait requires `open spec fn view()`, and the implementation correctly delegates to `closed spec fn spec_view()` to hide internals. This is well-documented with comments explaining the pattern. No issue, just noting the correctly applied workaround.

- **Standalone spec functions are all `pub open`.** Functions like `spec_classify_kcall`, `spec_is_locally_handled`, `spec_result_wf`, `spec_handle_sleep_error`, etc. are all `pub open spec fn`. For standalone functions (not methods on a type), `open` is appropriate since they define the specification vocabulary that callers need to reason about. This is consistent with the guidelines' distinction between type-level specs (closed) and shared specification vocabulary (open).

- **16 `external_body` functions.** Lines 556–747 contain 16 external body functions modeling subsystem calls. Each is documented with a trust boundary identifier (T1–T4b) and has meaningful postconditions (wf, success constraints). This is the expected pattern for dependency boundary types. The trust boundaries are well-documented in the module-level documentation (lines 88–116). No unjustified external bodies.

## Summary

The dispatcher verification model is well-structured and comprehensive, with 66 verified lemmas/functions and 0 errors. The spec methodology largely follows the guidelines with proper View types (`DispatchResultView`, `DispatchArgsView`), `pub closed spec fn` for `spec_view()` and `wf()`, and well-documented trust boundaries for all 16 `external_body` functions. No `assume` or `admit` statements exist anywhere in the spec or proof files.

The main deviations are: (1) public struct fields and public method specs referencing concrete fields (`result.is_success`) instead of view fields (`result@.is_success`), which is partly a Verus limitation but the specs should still prefer the view path; (2) two `pub open` spec accessors on `SleepError` that should be `pub closed`; (3) using `wf()` naming instead of `inv()` as prescribed by the guidelines. These are moderate methodology gaps that don't compromise soundness — the verification is complete and the trust boundaries are explicit and narrow. The ABI-level verification (`do_kcall_abi`) closing trust boundary T5 is particularly well done.
