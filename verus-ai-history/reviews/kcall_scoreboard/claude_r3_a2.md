# Review: kcall_scoreboard (claude-opus-4.6)

## Grade: A

## Previous Issues Status

### High Issue 1: `dispatch()` inlines raw field mutations without proving equivalence to split API
- **Status: FIXED.**
- **Verification:** New lemma `lemma_split_api_equals_full_cycle` (proof.rs lines 801–827) was added. It proves that composing the four individual spec transitions (`spec_begin_dispatch` → `spec_handle` → `spec_handled` → `spec_complete_dispatch`) produces the same `ScoreBoardView` as `spec_full_cycle` and `spec_dispatch_success`. Since the monolithic `dispatch()` postcondition references `spec_dispatch_success`, this completes the equivalence chain: `dispatch()` postcondition ↔ `spec_dispatch_success` ↔ `spec_full_cycle` ↔ split API composition. The raw field mutations in `dispatch()` are now provably equivalent to the split API transitions. Verified by Verus (69/0 pass).

### High Issue 2: `handle()` takes `&mut self` vs original `&self` (concurrency gap)
- **Status: Acknowledged limitation (no code fix expected).**
- **Verification:** T4 documentation remains thorough and the refinement argument (mutex + semaphore enforce total order) is sound. This is a fundamental Verus limitation — it cannot reason about concurrent program logic. The sequential model remains a valid over-approximation of the concurrent protocol for the properties being verified. No change needed.

### Medium Issue 1: `KcallResult` pub fields allow construction of non-wf results
- **Status: Properly mitigated (no fix needed).**
- **Verification:** Re-examined. The `ret.wf()` precondition on `handled()` (line 677) and `dispatch()` (line 779) is enforced by Verus at all call sites. Any caller constructing a `KcallResult` with pub fields would fail verification if the result doesn't satisfy `wf()` when passed to these functions. The constructors (`ok()`, `success()`, `error()`) all guarantee `wf()` in their postconditions. This is the correct Verus pattern — preconditions are statically checked, making runtime assertions redundant.

### Medium Issue 2: `abandon_dispatch()` only demonstrated from Signaled phase in monolithic `dispatch()`
- **Status: FIXED.**
- **Verification:** Three new lemmas added (proof.rs lines 829–912):
  - `lemma_abandon_from_signaled` (line 841): Proves stuck state with `dispatched_value == 1`, `handled_value == 0`.
  - `lemma_abandon_from_dispatched` (line 868): Proves stuck state with `dispatched_value == 0`, `handled_value == 0`.
  - `lemma_abandon_from_handled` (line 895): Proves stuck state with `dispatched_value == 0`, `handled_value == 1`.
  Each lemma uses phase-specific semaphore preconditions matching the `wf()` invariant, producing a fully characterized stuck state per phase. All verified by Verus.

### Medium Issue 3: `ScoreBoardSlot::get_board()` returns `&ScoreBoard` instead of `&mut`
- **Status: Not fixed.**
- **Assessment:** This remains an API fidelity gap — the original `get_mut()` returns `&'static mut ScoreBoard`, but the verified slot only offers `&ScoreBoard` via `get_board()`. Callers must access `slot.board` directly for mutations. This is a Verus limitation (`&mut` return types in `verus!{}` are not well supported) documented in T1. It does not affect the correctness of the verified properties since the split API and `dispatch()` operate on `&mut self` of the `ScoreBoard` directly. Downgraded to Low.

### Low Issues (4): Out-of-scope items, ghost field, spec_n_identical_cycles
- **Status: Unchanged.** All properly documented as out of scope or low priority. No changes needed.

## New Changes Analysis

The update added exactly 4 new proof lemmas (65 → 69 verified conditions):

1. `lemma_split_api_equals_full_cycle` — Establishes equivalence between split API composition and monolithic `spec_full_cycle`/`spec_dispatch_success`. Sound and well-structured.
2. `lemma_abandon_from_signaled` — Phase-specific abandonment proof. Preconditions correctly reflect wf() in Signaled phase.
3. `lemma_abandon_from_dispatched` — Phase-specific abandonment proof. Preconditions correctly reflect wf() in Dispatched phase.
4. `lemma_abandon_from_handled` — Phase-specific abandonment proof. Preconditions correctly reflect wf() in Handled phase.

**No new issues introduced.** All new lemmas have appropriate preconditions, correct ensures clauses, and pass Verus verification. The spec and exec files are unchanged.

## Issues Found

### Critical

*None.*

### High

*None.*

### Medium

*None.*

### Low

- **Location:** `ScoreBoardSlot::get_board()` in scoreboard.rs (exec), line 1016
  - **Description:** Returns `&ScoreBoard` while original `get_mut()` returns `&'static mut ScoreBoard`. Callers must use `slot.board` directly for mutable access. This is a documented Verus limitation (T1) that does not affect correctness of the verified properties.
  - **Suggested Fix:** None needed. The limitation is properly documented and does not impact verification soundness.

- **Location:** `spec_n_identical_cycles` in scoreboard.spec.rs, lines 481–495
  - **Description:** Uses identical args/ret for all n cycles. The informal argument that the cycle counter property generalizes to varying inputs is correct (each `spec_full_cycle` increments by 1 regardless of args/ret), but a formal heterogeneous version would be strictly stronger.
  - **Suggested Fix:** Optional. Consider adding a sequence-based variant for completeness, but the existing `lemma_cycle_counter_monotonic` (single cycle) and `lemma_two_cycles` (two different cycles) already provide sufficient evidence.

## Positive Observations

- **No assume/external_body/trusted:** 69 verified conditions with zero escape hatches. Fully mechanized proofs throughout.

- **Clean fix for split API equivalence:** `lemma_split_api_equals_full_cycle` precisely addresses the concern about the monolithic `dispatch()` bypassing the split API. The proof establishes that the four-step split composition equals both `spec_full_cycle` and `spec_dispatch_success`, closing the equivalence loop.

- **Thorough per-phase abandonment coverage:** The three new abandonment lemmas (`lemma_abandon_from_signaled`, `lemma_abandon_from_dispatched`, `lemma_abandon_from_handled`) each use phase-appropriate semaphore preconditions (matching `wf()` invariant), fully characterizing the stuck state for every possible interruption point.

- **Comprehensive state machine verification:** The complete property set includes: initialization correctness, four phase transitions, data integrity (args + result), mutex invariant, semaphore signal/consume protocol, cycle counting, error path preservation, abandonment characterization, injectivity, and multi-cycle induction.

- **Exemplary documentation:** Trust boundaries T1–T5 remain clearly documented. The API Mapping table, API Divergence section, and Verification Scope section provide complete transparency. The new lemmas follow the same documentation conventions.

- **Well-structured proof organization:** The proof file is organized into clear sections: Initialization, State Machine Transitions, Semaphore Signal Protocol, Protocol Correctness, Invalid Transition Guards, Multi-Cycle Properties, KcallArgs/KcallResult auxiliary lemmas, Error Path Preservation, Abandon from Each Active Phase, and ScoreBoardSlot.

## Summary

The prover addressed the two actionable issues from the previous review:

1. **Split API equivalence** was proven via `lemma_split_api_equals_full_cycle`, establishing that the monolithic `dispatch()` postcondition (referencing `spec_dispatch_success`) is equivalent to the split API composition. This closes the gap between the two APIs.

2. **Per-phase abandonment** was proven via three new lemmas covering Signaled, Dispatched, and Handled phases, each with phase-specific semaphore preconditions. This provides complete coverage of all interruption points.

The remaining issues are Low priority: one Verus limitation (`get_board` returns `&` not `&mut`, properly documented) and one optional enhancement (heterogeneous multi-cycle spec). Neither affects the soundness or completeness of the verified properties.

The verification is now comprehensive for its stated scope (sequential state machine correctness). The primary inherent limitation — the `&self` → `&mut self` change for `handle()` and the sequential model's inability to verify concurrent aspects — remains a Verus platform limitation that is thoroughly documented in T4 with a convincing refinement argument.

Grade upgraded from A- to A based on the targeted fixes addressing the substantive review concerns.
