# Review: kcall_handler (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `poll_messages_raw` / `poll_messages_gated` (exec: verus/split/kernel/kcall/handler.rs)
  **Description:** Message polling is now feature-gated, but the model still ignores the batching and buffer-limit guards (`IKC_POLL_BATCH_SIZE`, `MAX_IKC_MESSAGES`). As a result, `poll_messages_raw()` may return true in cases where the original loop would not read any messages, which can suppress yields and break semantic equivalence in those states.
  **Suggested Fix:** Add a spec/assumption that captures the buffer-limit/batch behavior (e.g., a precondition on when `poll_messages_raw()` may return true), or model a bounded polling loop that can return false when the kernel buffer is full.

### Low
- **Location:** Scoreboard poll outcomes (spec vs exec)
  **Description:** The spec defines `ScoreBoardPollOutcome` (including `AccessError`), but the exec model uses `ScoreBoardPollResult` with no conversion or ensures linking it to the spec enum. This leaves the spec’s error outcome unconnected to executable behavior.
  **Suggested Fix:** Add an exec-to-spec conversion similar to `spec_harvest_to_outcome`, or remove the unused spec enum/lemmas if AccessError is intentionally out of scope.

## Positive Observations
- The lifecycle history now uses the real harvest outcome (via `spec_harvest_to_outcome`), fixing the prior synthetic-history issue.
- A full loop model (`kcall_handler_loop`) is now present, and the stdio feature gate for IKC polling is explicitly modeled.
- Harvest notification semantics and termination handling remain aligned with the original code.

## Summary
Most prior issues are genuinely fixed, including lifecycle history linkage, loop coverage, and stdio gating. The remaining gaps are limited to message polling limits and a minor spec/exec mismatch around scoreboard outcomes. Addressing those would bring the verification close to complete and sound.
