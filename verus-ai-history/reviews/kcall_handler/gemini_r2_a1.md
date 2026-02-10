# Review: kcall_handler (gemini-3-pro-preview)

## Grade: A-

## Issues Found

### Medium
- **Verification of Model vs Implementation**: The verification target `verus/split/kernel/kcall/handler.rs` contains a *model* of the handler (`run_full_iteration`, `run_iteration`), not the actual `kcall_handler` function from `src/kernel/src/kcall/handler.rs`. While the model logic closely mirrors the source, there is no mechanical guarantee that the implementation matches the verified model. If the original source changes (e.g., yield logic), the model could drift.
  - *Fix*: Ideally, verify the actual code by importing it or using `verus!` macro on the source. If that's not feasible due to complex dependencies, add a strict regression test or checksum mechanism to detect drift in the original file.

### Low
- **Manual Kcall Classification**: The `spec_classify_handler_kcall` function (and `lemma_dispatch_coverage_matches_source`) manually lists all 21 kcall numbers to match the source's `match` statement. This is a duplication of logic.
  - *Fix*: This is acceptable for a model, but requires manual updates when new syscalls are added.
- **Missing Liveness Proof**: The verification proves *conditional* liveness (if INITD terminates, loop exits), but does not prove that INITD *will* terminate (which depends on external factors). This is noted in the docs but represents a limit of the verification scope.

## Positive Observations
- **Clear Loop Invariant**: The use of a ghost history (`Seq<HarvestOutcome>`) to track the non-termination of INITD is a robust way to prove the loop invariant.
- **Faithful Modeling**: The model `run_full_iteration` accurately captures the subtle control flow of the original handler, including the "yield iff no work" logic and the "break on INITD termination" behavior (modeled by suppressing yield when `should_terminate` is true).
- **Strong Spec Constants**: The proof includes regression lemmas (`lemma_spec_constants_match_source`) to ensure hardcoded constants like `INITD_PID=1` and `ENOSYS=88` match the source definitions.
- **Clean Split**: The separation into `exec` (model), `spec`, and `proof` files is clean and follows the project structure well.

## Summary
The verification of `kcall_handler` is high quality, providing strong guarantees about the correctness of the kernel's main event loop logic (yield conditions, work tracking, termination). The primary limitation is that it verifies a *model* of the code rather than the code itself, introducing a maintenance burden to keep the model synchronized with the implementation. The use of external bodies for subsystems is appropriate for this level of abstraction.
