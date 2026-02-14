# Review: sleeping_thread Exec Consistency (claude-opus-4.6)

## Grade: A

## Function Coverage

| Original Function | Verus Status | Verdict |
|---|---|---|
| `from_state` | Verified (full postconditions) | ✅ Equivalent — body `Self { state, alarm }` identical; type changes (`Box<ThreadState>`→`ThreadState`, `Option<SystemTime>`→`Option<int>`) are documented modeling decisions |
| `wakeup` | Verified (full postconditions) | ✅ Equivalent — body `ReadyThread::from_state(self.state)` identical |
| `interrupt` | Verified (full postconditions) | ✅ Equivalent — body `InterruptedThread::from_state(self.state, reason)` identical; `InterruptReason`→`int` modeled with `spec_valid_reason` |
| `id` | Verified | ✅ Equivalent — body `self.state.id()` identical |
| `thread_state` | Verified | ✅ Equivalent — body `&self.state` identical |
| `thread_state_mut` | `#[verifier::external]` | ✅ Properly handled — Verus cannot express `&mut T` returns; trust obligations documented |
| `join_cond` | `#[verifier::external]` | ✅ Added (was MISSING) — stub returns dummy `Condvar`; consistent with `interrupted.rs` pattern |
| `alarm` | Verified | ✅ Equivalent — body `self.alarm` identical |
| `set_thread_data_area` | Verified (full postconditions) | ✅ Equivalent — body `self.state.store_thread_data_area(user_tda)` identical; interleaved `proof {}` blocks are ghost-only |
| `get_thread_data_area` | Verified | ✅ Equivalent — body `self.state.get_thread_data_area()` identical |

**All 10 original functions accounted for: 8 verified, 2 `#[verifier::external]` with documented justification.**

## Issues Found

### Critical

- None.

### Minor

1. **`join_cond` stub returns dummy `Condvar` instead of delegating to `self.state.join_cond()`.** This is intentional since the Verus `ThreadState` model elides the condvar field entirely, but the stub body is technically non-faithful to the original. The `#[verifier::external]` annotation correctly excludes it from verification, and the trust boundary documentation (lines 43–50 of the exec file) explicitly acknowledges this. **No action needed** — this is a sound architectural decision consistent with the project-wide `Condvar` elision.

2. **`ReadyThread` boundary model includes `admission_time` field, diverging from `interrupted.rs` boundary model.** The exec file documents this at line 59 with a TODO for cross-module standardization. Both are individually sound. **Low risk** — noted for future cross-module work.

3. **`clock_now()` is `#[verifier::external_body]` with postcondition `result >= 0`.** This is a trust assumption. The exec file documents (lines 79–87) that an identical declaration exists in `ready.rs` and flags it for extraction into a shared module. The postcondition is minimal and sound. **No action needed.**

## Verification

- **33 verified, 0 errors** — confirmed by running `./verus-ai/scripts/verify.sh sleeping_thread`.
- **No `assume`, `admit`, or unjustified `external_body`** found in exec, spec, or proof files.
- The sole `external_body` (`clock_now`) has a minimal, sound postcondition (`result >= 0`).
- The two `#[verifier::external]` functions (`thread_state_mut`, `join_cond`) have documented trust obligations.

## Equivalence Justification Assessment

All 8 documented equivalences in the consistency report are **sound**:

- **Type modeling** (`Box`→direct, `SystemTime`→`int`, `InterruptReason`→`int`, `VirtualAddress`→`int`, `Condvar`→elided) is consistent across the verification model and documented in the module header.
- **Visibility changes** (`pub(super)`→`pub`, private fields→`pub`) are documented as Verus proof ergonomics requirements.
- **Exec bodies** are structurally identical after stripping ghost/proof annotations, which compile away.
- **`spec_valid_reason` predicate** correctly constrains the `int` domain to `{0, 1}`, matching `InterruptReason::Killed` and `InterruptReason::TimedOut`.

## Summary

The exec consistency fixes are thorough and well-documented. All 10 original functions are present in the verified code — 8 with full Verus verification and 2 with justified `#[verifier::external]` annotations. The missing `join_cond` function was properly added following the established `interrupted.rs` pattern. Type modeling decisions are consistent, documented, and sound. The proof file provides comprehensive lemma coverage for construction, state transitions, and property preservation. No `assume`/`admit` or unjustified trust assumptions were found. The only items to track are cross-module TODOs (boundary model standardization, clock utility extraction) which are correctly flagged in the source.
