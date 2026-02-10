# Review: kcall_handler (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** kcall_handler coverage (exec: verus/split/kernel/kcall/handler.rs)
  **Description:** The verified code does not model the full `kcall_handler` loop: there is no function that initializes events, iterates until INITD terminates, or performs the post-loop zombie drain. `run_iteration`/`run_full_iteration` only model a single iteration, so the main function is effectively uncovered, and the loop invariant in the spec is never tied to executable behavior.
  **Suggested Fix:** Add a verified wrapper that models the full loop (including init, iteration history, termination, and post-loop drain) and connect it to the invariant/proofs.
- **Location:** `run_iteration` harvest work flag (exec: verus/split/kernel/kcall/handler.rs)
  **Description:** The model sets `harvested_process = harvest.found` even if `notify_termination` would fail. In the original code, `harvested_process` is only set to true when `notify_process_termination` succeeds; failures leave it false, which can change the yield decision.
  **Suggested Fix:** Model `notify_termination` returning success/failure and update `harvested_process` accordingly, or constrain the external body with a postcondition that aligns with the original success-only behavior.

### Medium
- **Location:** IKC polling (exec: verus/split/kernel/kcall/handler.rs)
  **Description:** `poll_messages()` is always executed and unconstrained, but the original code only polls under the `stdio` feature flag and respects `IKC_POLL_BATCH_SIZE` and `MAX_IKC_MESSAGES`. This allows message work in builds that should never observe it and ignores batching/limit behavior.
  **Suggested Fix:** Gate message polling in the model to match the feature flag and add a simple spec capturing batching/limit effects (e.g., at-most-one or bounded message receipt per iteration).
- **Location:** Error paths for scoreboard/harvest (spec vs exec)
  **Description:** The spec defines `HarvestOutcome::HarvestError`, but the executable model uses `ZombieHarvestResult` with only a `found` flag; scoreboard access/handle errors and harvest errors are not represented. This removes error-and-continue behaviors present in the original loop.
  **Suggested Fix:** Extend the exec model to include explicit error outcomes (or model them in `poll_scoreboard_full`/`harvest_zombies`) and prove that they do not affect termination and yield correctness.

### Low
- **Location:** Post-loop cleanup (exec: verus/split/kernel/kcall/handler.rs)
  **Description:** `drain_remaining_zombies` is an external body with `ensures true` and is not invoked by any modeled loop. The review text claims post-loop cleanup but no property is actually proved.
  **Suggested Fix:** Either model a verified drain loop with a minimal progress spec or explicitly mark the cleanup as out-of-scope in the verified claims.

## Positive Observations
- Dispatch classification totality/correctness and invalid syscall handling are explicitly specified and proved.
- Yield-iff-idle behavior and work-flag monotonicity are captured clearly.
- Trust boundaries are documented, and the spec/proof separation is clean and organized.

## Summary
The verification captures important control-flow properties for a single iteration but misses the full handler loop, including initialization, termination sequencing, and post-loop cleanup. Several behaviors (harvest notification failures, feature-gated message polling, and error paths) are abstracted away in ways that can change yield and equivalence. Strengthening the loop model and aligning work-flag semantics with the original will materially improve coverage and correctness.
