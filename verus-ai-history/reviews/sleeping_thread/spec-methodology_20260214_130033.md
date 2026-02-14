# Review: sleeping_thread Spec Methodology (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- **Several public methods lack `wf()` precondition (Step 3 violation):** The methods `id()` (line 328), `thread_state()` (line 341), `alarm()` (line 354), and `get_thread_data_area()` (line 400) do not require `self.wf()` as a precondition. Per the guidelines (Step 3), every public method with a `Self` input parameter should require `inv()`/`wf()`. While these are read-only accessors that happen to work without `wf()`, omitting it weakens the contract and makes it possible for callers to invoke them on ill-formed instances, breaking the methodology's encapsulation discipline.

- **Public method spec uses direct field access `self.state@` (Step 3 violation):** In `thread_state()` (line 344), the postcondition `result@ == self.state@` directly accesses the implementation field `self.state` rather than going through the view (`self@.state`). The guidelines state: "never talk directly about the fields of a `Self` parameter... instead say things like `self@.y`."

### Medium
- **Spec functions are `pub open` instead of private (Step 3 tension):** The guidelines say "Don't write any further `pub` specification functions in `impl MyType` beyond `inv` and `view`." However, `SleepingThread` exposes 10+ `pub open spec fn` accessors (e.g., `spec_id`, `spec_alarm`, `spec_locked_mutex_count`, etc.) in the `impl SleepingThread` block. These would more properly be placed on `SleepingThreadView` or kept private. This is a common pragmatic pattern in this codebase and the functions do delegate to `ThreadState` specs cleanly, so the impact is moderate.

- **`clock_now()` boundary function duplicated across modules:** As noted in the code's own TODO (line 87), this `external_body` function is duplicated between `sleeping.rs` and `ready.rs`. While individually sound, this creates a maintenance risk where postconditions could diverge.

### Low
- **Boundary model structural divergence acknowledged but unresolved:** The `ReadyThread` boundary model here includes `admission_time` while the one in `interrupted.rs` does not (documented at line 57–60). This is tracked with a TODO and does not affect local soundness, but could cause issues in future cross-module composition.

- **`SleepingThreadView` is defined but not used in public method specs:** The `View` implementation exists and is `closed`, but public method specs exclusively use `self.spec_*()` accessors rather than `self@.field`. This means the view type is essentially dead code from a specification perspective — callers cannot reason through it. This contradicts the intent of Step 1 (write an abstraction) and Step 3 (use `self@.y` in public specs).

## Positive Observations

- **`wf()` is correctly `pub closed spec fn`** for all three types (`SleepingThread`, `ReadyThread`, `InterruptedThread`).
- **`view()` is correctly `closed spec fn`** in all three `View` implementations.
- **View types use fully abstract types:** `int`, `Option<int>`, `nat`, `ThreadStateView` — no concrete types like `usize`, `Vec`, `SystemTime`.
- **No `assume` or `admit` statements** anywhere in the codebase.
- **`external_body` usage is justified and well-documented:** `clock_now()` models an opaque system clock; `thread_state_mut()` is `#[verifier::external]` due to Verus's inability to express `&mut T` returns, with thorough trust boundary documentation.
- **Verification passes cleanly:** 33 verified, 0 errors.
- **Comprehensive proof lemmas** covering construction, identity preservation, state transitions, mutex accounting, drop safety, TDA round-trip, and view equality.
- **Excellent documentation** of trust boundaries, cross-module obligations, and verification model decisions.

## Summary

The sleeping_thread verification is well-executed with clean verification (33/0), no assumes/admits, properly closed `wf()` and `view()` functions, and fully abstract view types. The main methodology gaps are: (1) four read-only public methods missing `wf()` preconditions, (2) one postcondition using direct field access `self.state@` instead of `self@.state`, and (3) spec accessors placed on `impl SleepingThread` rather than on `SleepingThreadView` as the guidelines prefer. These are moderate deviations from the methodology but do not compromise soundness. The trust boundary documentation is exemplary.
