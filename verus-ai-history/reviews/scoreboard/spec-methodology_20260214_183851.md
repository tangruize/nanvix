# Review: scoreboard Spec Methodology (claude-opus-4.6)

## Grade: B+

## Issues Found

### Critical

- None.

### High

- **view() functions are `open spec fn` instead of `pub closed spec fn`.** The guidelines (Step 1) explicitly require `pub closed spec fn view(&self) -> MyTypeView` so that users cannot see implementation internals through the view mapping. All four `View` trait implementations (`KcallArgs`, `KcallResult`, `ScoreBoard`, `ScoreBoardSlot`) at lines 121, 137, 145, 161 of `scoreboard.spec.rs` use `open spec fn view`. This exposes the mapping between concrete fields and abstract View fields, defeating the abstraction barrier. **Mitigation:** Verus's `View` trait requires `open spec fn view` — this is a Verus framework constraint. The implementations cannot be `closed` when implementing the standard `View` trait. This is a known limitation, not an authoring error. However, this should be documented as a deviation.

- **wf()/inv() functions are `pub open spec fn` instead of `pub closed spec fn`.** The guidelines (Step 2) require `pub closed spec fn inv(&self) -> bool` so users cannot see implementation invariant details. `ScoreBoard::wf()` (line 230), `KcallResult::wf()` (line 205), and `ScoreBoardSlot::wf()` (line 524) are all `pub open spec fn`. This exposes internal invariant structure (semaphore counts, mutex state, phase relationships) to consumers. **Partial justification:** The module documentation notes all fields are `pub` because "Verus's `verus!{}` macro requires field access for spec functions and View trait implementations." Since the struct fields are already public, making `wf()` closed would provide little additional encapsulation. Nevertheless, this deviates from the guidelines.

- **Public method specs reference `self.field` directly instead of `self@.field`.** The guidelines (Step 3) state: "never talk directly about the fields of a `Self` parameter. For instance, don't say `self.x`; instead say things like `self@.y`." Multiple public methods violate this:
  - `begin_dispatch` ensures: `self.locked`, `self.dispatched_value == 1`, `self.args@ == args@` (lines 488-490).
  - `handle` ensures: `self.dispatched_value == 0`, `self.args@ == old(self).args@` (lines 527-528, 530).
  - `try_handle` ensures: `self.dispatched_value == 0`, `self.args@ == old(self).args@` (lines 561-562).
  - `try_begin_dispatch` ensures: `self.locked`, `self.dispatched_value == 1` (lines 606-607).
  - `abandon_dispatch` ensures: `self.locked`, `self.phase == old(self).phase`, `self.dispatched_value`, `self.handled_value` (lines 641-646).
  - `get_args` ensures: `self.args@` (line 669).
  - `handled` ensures: `self.handled_value == 1`, `self.result@`, `self.args@` (lines 695-697).
  - `complete_dispatch` requires: `old(self).completed_cycles < u64::MAX` (line 724); ensures: `self.locked`, `self.dispatched_value`, `self.handled_value`, `self.result@`, `self.completed_cycles` (lines 728-735).
  - `dispatch` requires: `old(self).completed_cycles < u64::MAX` (line 808); ensures extensively reference `self.locked`, `self.phase`, `self.args@`, `self.result@`, `self.dispatched_value`, `self.handled_value`, `self.completed_cycles` (lines 811-868).
  - `is_idle`, `is_signaled`, `is_dispatched`, `is_handled` all require `self.wf()` directly (valid) but these are simple inspectors.
  - `ScoreBoardSlot` methods: `try_get_board` ensures `self.board.wf()` (line 1079). `get_board` ensures `(*result).wf()`, `(*result)@ == self.board@` (lines 1101-1102).

  **Partial justification:** Since struct fields are `pub` and `wf()`/`view()` are `open`, the abstraction barrier is already open. Using `self.field` directly is pragmatically equivalent to `self@.field` in this model. However, this makes the specs brittle to refactoring — if the struct representation changes, all specs must be updated rather than just the view mapping.

### Medium

- **All spec functions in `impl ScoreBoard` are `pub open` — none are private.** The guidelines (Step 4) suggest private spec functions for internal helpers. All ~20 spec functions on `ScoreBoard` (e.g., `spec_dispatched_count`, `spec_handled_count`, `spec_is_idle`, `spec_begin_dispatch`, `spec_full_cycle`, etc.) are `pub open spec fn`. Many of these are internal state transition specifications that should arguably be private or at least `pub closed` per the guidelines. The state transition specs (`spec_begin_dispatch`, `spec_handle`, `spec_handled`, `spec_complete_dispatch`, `spec_abandon_dispatch`) expose protocol internals to consumers.

- **No `inv()` function — uses `wf()` instead.** The guidelines use `inv()` as the canonical name. The code uses `wf()` (well-formedness). This is a naming deviation only; the function serves the same purpose. The name `wf()` is arguably more descriptive for this model, but inconsistent with the methodology document.

### Low

- **`KcallArgsView` uses both `int` and `nat` types.** `pid` and `tid` are `int` (correct, since `ProcessIdentifier`/`ThreadIdentifier` wrap `i32` which can be negative), while `number`, `arg0`–`arg3` are `nat` (correct, since they wrap `u32`). This is appropriate abstract typing — noted for completeness.

- **`ScoreBoardView` exposes internal state in abstract view.** Fields like `locked`, `dispatched_value`, `handled_value`, and `completed_cycles` are implementation details (mutex state, semaphore counters, verification-only counter). The guidelines (Step 1) say "It should hide internal fields that aren't important to users of `MyType`." A client of ScoreBoard only needs `phase`, `args`, and `result`. **Mitigation:** For this verification model, these fields are needed to express and verify the protocol invariants. This is a reasonable trade-off for a verification-only model.

## No assume/admit/unjustified external_body

No `assume`, `admit`, or `#[verifier::external_body]` annotations were found in the scoreboard files (`scoreboard.rs`, `scoreboard.spec.rs`, `scoreboard.proof.rs`). All 71 verification obligations pass without shortcuts. The `external_body` instances found in the `kcall` directory are in `dispatcher.rs` and `handler.rs` (separate modules with documented trust boundaries), not in the scoreboard.

## Verification Result

```
verification results:: 71 verified, 0 errors
Duration: 9s
Status: PASSED
```

## Summary

The scoreboard verification model is thorough and well-structured, with comprehensive coverage of the four-phase dispatch protocol, error paths, multi-cycle properties, and semaphore signaling. All 71 verification obligations pass with zero errors and no assume/admit/external_body shortcuts — a clean verification.

The primary methodology deviation is that the abstraction barrier prescribed by the guidelines is not enforced: `view()` and `wf()` are `open` rather than `closed`, struct fields are `pub`, and public method specs reference `self.field` directly rather than `self@.field`. This is partly justified by Verus framework constraints (the `View` trait requires `open spec fn`) and partly by the design choice to expose all model fields as `pub` for spec-level reasoning. However, it means the specs are coupled to the struct representation — a refactoring of `ScoreBoard`'s fields would require updating all method specs, not just the view mapping.

The model quality is high: View types use appropriate abstract types (`int`, `nat`, `bool`), state transitions are precisely modeled, error paths are characterized, and inductive properties (n-cycle counter) are proven. The documentation is excellent, with detailed trust boundary analysis and API mapping tables. Grade B+ reflects the solid verification and documentation quality offset by the systematic deviation from the encapsulation guidelines.
