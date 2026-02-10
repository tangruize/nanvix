# Review: kcall_scoreboard (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

- None.

### High

- **Location:** `ScoreBoard` struct (exec: `scoreboard.rs:224`), all fields `pub`
  - **Description:** All fields on `KcallArgs`, `KcallResult`, `ScoreBoard`, and `ScoreBoardSlot` are `pub`. This means any caller can directly mutate internal state (e.g., `sb.phase = ScoreBoardPhase::Handled`), completely bypassing the verified state-machine transitions. Verus's ensures clauses on methods are sound, but they only guarantee correctness *if callers use those methods*. With public fields, a caller can silently break invariants.
  - **Suggested Fix:** Make all struct fields private (or `pub(crate)` at most) and expose only the verified methods as the API. Add getter methods where needed. This is also a Nanvix coding standard: "Member fields in `struct`s must be private and accessed via getter/setter methods."

- **Location:** `ScoreBoard::handle()` (exec: `scoreboard.rs:451`)
  - **Description:** The original `handle()` takes `&self` and uses an atomic `try_down()` on the semaphore, meaning it can be called concurrently without exclusive access. The verified model requires `&mut self`, which is a significant semantic divergence. This is documented in the API Divergence section, but it means the verification doesn't actually prove anything about the concurrent `&self` usage pattern. The `get_args()` separation is a consequence of this modeling choice.
  - **Suggested Fix:** This is an inherent limitation of Verus's sequential model. The documentation correctly identifies this as a trust boundary (T4). No code fix needed, but the trust boundary documentation should note that the `&self → &mut self` change means the mutual exclusion property between `handle()` and `dispatch()` is *assumed*, not proven.

### Medium

- **Location:** `ScoreBoard::abandon_dispatch()` precondition (exec: `scoreboard.rs:574-577`)
  - **Description:** The `abandon_dispatch()` function requires `old(self).spec_is_handled()` as a precondition. However, in the original code, `handled.down()` can be interrupted *before* the handler completes (i.e., while still in `Signaled` or `Dispatched` phase). The model only captures the case where the handler has already completed and signaled back. The `Signaled` phase abandon (handler hasn't started) and `Dispatched` phase abandon (handler is mid-processing) are not modeled.
  - **Suggested Fix:** Add `abandon_dispatch_signaled()` and `abandon_dispatch_dispatched()` variants that model interruption in earlier phases, or generalize the precondition to accept any non-Idle phase.

- **Location:** `ScoreBoardSlot` — missing `get_board_mut()` (exec: `scoreboard.rs:851`)
  - **Description:** The API mapping table mentions `get_board[_mut]()` but only `get_board(&self)` is implemented, returning an immutable reference. Since all state-machine transitions require `&mut self`, callers cannot actually perform `begin_dispatch()`, `handle()`, `handled()`, or `complete_dispatch()` through the slot API. The `get_board_mut()` method is missing.
  - **Suggested Fix:** Add a `get_board_mut(&mut self) -> &mut ScoreBoard` method with appropriate ensures clauses.

- **Location:** Proof file — `lemma_try_handle_fail_preserves_state` (proof: `scoreboard.proof.rs:570-578`)
  - **Description:** The ensures clause `sb@ == sb@` is a tautology — it's always true regardless of any state. This lemma claims to prove state preservation on failed try_handle, but the ensures clause doesn't actually assert anything meaningful about state preservation. It should compare against some prior state.
  - **Suggested Fix:** Since this is on `&ScoreBoard` (immutable reference), state preservation is trivially guaranteed by Rust's type system. The lemma could be removed or its documentation clarified to note it's just confirming wf() is preserved. A more useful version would operate on a pre/post view comparison.

- **Location:** Proof file — `lemma_try_handle_idle_noop` (proof: `scoreboard.proof.rs:586-594`)
  - **Description:** Same issue: `sb@ == sb@` is a tautology. Does not prove meaningful state preservation.
  - **Suggested Fix:** Same as above.

- **Location:** `KcallResult` model (exec: `scoreboard.rs:205-210`)
  - **Description:** The original `KcallResult` is a Rust enum with `Success(KcallSuccess)` and `Error(KcallError)` variants. The verified model uses `is_success: bool` + `value: i64`. This means the model allows states that the original enum cannot represent (e.g., the model cannot distinguish between different success payload types if `KcallSuccess` were to evolve). More importantly, the `Copy` semantics of the original `KcallResult` (line 173: `Ok(self.ret)`) are implicitly matched but not verified — the model doesn't prove that reading `self.ret` after the handshake returns a *copy* rather than a moved value.
  - **Suggested Fix:** Add a comment noting that `KcallResult: Copy` in the original ensures `Ok(self.ret)` doesn't move the value out. The flat struct model inherently has copy semantics in Verus, so this is sound, but should be documented.

### Low

- **Location:** `ScoreBoard::complete_dispatch()` (exec: `scoreboard.rs:657`)
  - **Description:** Returns `Ghost<KcallResultView>` but the original `dispatch()` returns `Ok(self.ret)` — a concrete `KcallResult`, not a ghost. The ghost return type means callers can only use the result at the spec level, not at the exec level. This is a minor semantic gap.
  - **Suggested Fix:** Return the concrete `KcallResult` (or a copy of it) in addition to or instead of the ghost value.

- **Location:** Module-level `pub fn init()` (original: `mod.rs:192-195`)
  - **Description:** The module-level `init()` function is listed as out of scope but it is the public API entry point. Its absence means the verified model doesn't cover the full public API surface.
  - **Suggested Fix:** This is a thin wrapper (`info!` + `ScoreBoard::init()`), so omission is reasonable. No action needed, but noting for completeness.

- **Location:** `impl Debug for KcallArgs` (original: `mod.rs:66-74`)
  - **Description:** Not modeled. Correctly identified as out of scope (no safety implications).
  - **Suggested Fix:** None needed.

- **Location:** Spec file — `spec_n_identical_cycles` (spec: `scoreboard.spec.rs:443`)
  - **Description:** The `spec_n_identical_cycles` function only models identical args/ret across all cycles. While the comment correctly notes the cycle counter property generalizes, the spec itself cannot be used to reason about cycles with varying arguments.
  - **Suggested Fix:** Consider adding a `spec_n_varying_cycles` that takes a sequence of (args, ret) pairs, or document more prominently that this is intentionally limited to the identical case.

## Positive Observations

- **Excellent documentation.** The module-level doc comment is exceptionally thorough, with a clear API mapping table, trust boundary analysis, refinement argument for the sequential model, and explicit scope boundaries. This is among the best-documented verification efforts I've seen.
- **Sound state machine model.** The four-phase protocol (Idle → Signaled → Dispatched → Handled → Idle) accurately captures the rendezvous handshake semantics. The semaphore values are explicitly tracked rather than abstracted away.
- **Comprehensive error path modeling.** Both success and failure paths are modeled for `try_handle()`, `try_begin_dispatch()`, and `abandon_dispatch()`. The characterization of the stuck state after `abandon_dispatch()` as a wf() violation is particularly insightful.
- **Well-formedness invariant.** The `wf()` spec ties together phase, mutex, and semaphore state in a crisp invariant that is preserved by all happy-path transitions and intentionally violated by `abandon_dispatch()`.
- **Clean spec/proof/exec separation.** Specifications, proofs, and executable code are cleanly separated into three files with appropriate include directives. Proof lemmas are well-organized by category.
- **Verification passes.** All 62 verification conditions pass, with no `assume` or unjustified `external_body` in the module.
- **Ghost state for induction.** The `completed_cycles: Ghost<nat>` field enables inductive reasoning (n-cycle lemma) without runtime overhead.
- **Semaphore infallibility proofs.** `lemma_semaphore_up_dispatched_cannot_fail` and `lemma_semaphore_up_handled_cannot_fail` justify why `up()` errors are not modeled, closing a potential soundness gap.

## Summary

This is a high-quality verification of the scoreboard dispatch protocol. The state machine model is faithful to the original's four-phase handshake, with semaphore signal/consume transitions explicitly tracked. Error paths are thoroughly modeled, including the nuanced `abandon_dispatch()` stuck state. The documentation is exemplary, with clear trust boundaries and a well-reasoned refinement argument for the sequential abstraction.

The main areas for improvement are: (1) making struct fields private to prevent invariant violations by callers, (2) adding `get_board_mut()` to complete the slot API, (3) modeling `abandon_dispatch()` for earlier phases (Signaled/Dispatched), and (4) fixing the tautological ensures clauses in two proof lemmas. The `&self → &mut self` change for `handle()` is an inherent modeling limitation that is well-documented. Overall, this verification provides strong confidence in the sequential correctness of the scoreboard protocol.
