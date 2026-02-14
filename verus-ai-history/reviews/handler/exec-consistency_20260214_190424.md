# Review: handler Exec Consistency (claude-opus-4.6)

## Grade: A

## Verification Status

- **49 verified, 0 errors** — all functions pass Verus verification.
- No `assume`, `admit`, or unjustified `external_body` found.

## Review Criteria Assessment

### 1. MISMATCH Functions: Properly Restored or Equivalence Documented?

The consistency report states 0 mismatches were fixed. This is consistent with the code — the exec
model was already structurally aligned with the original source. No restoration was needed.

**Verdict: PASS.**

### 2. MISSING Functions: Added with Proper Verification?

One function was added: `kcall_handler()` as an `external_body` entry point (handler.rs:1065-1075).
This correctly models the original `kcall_handler(hal, mm, pm) -> ExitStatus` signature which
cannot be compiled by Verus due to OS types (`Hal`, `VirtMemoryManager`, `ProcessManager`).

- The `external_body` annotation is justified: the original parameters are Verus-incompatible.
- The postcondition is `ensures true` — intentionally weak since exit status originates from
  `harvest_zombies()` (T2 trust boundary). The verified properties (dispatch, yield, termination)
  are proved on the decomposed model in `kcall_handler_loop()`.
- The equivalence argument in the doc comment (lines 1044-1055) correctly maps each original
  API call to its shadow model counterpart.

**Verdict: PASS.**

### 3. Equivalence Justifications: Are They Sound?

The 22 extra functions and 8 extra structs use a shadow model decomposition pattern. The
justifications are sound:

**External bodies (T1-T5):**
- `signal_handled` (T1): Models `scoreboard.handled(ret)`. Postcondition `ensures true` matches
  the original's fire-and-forget semantics where signaling failure is logged but doesn't affect
  `kcall_handled`.
- `dispatch_to_subsystem` (T2): Models subsystem dispatch. Error postcondition
  (`is_error ==> error_code != 0`) is reasonable.
- `poll_messages_raw` (T4): Models IKC polling. No postcondition, correct since message
  availability depends on external I/O.
- `harvest_zombies` (T2): Models `pm.harvest_zombies(mm)`. The `is_initd <==> (found && pid == 1)`
  bidirectional postcondition correctly encodes the INITD identity invariant. The
  `error ==> !found` postcondition correctly models mutual exclusion.
- `notify_termination` (T2): Models `EventManager::notify_process_termination`. Returns bool
  matching the original `Ok(()) => true` / `Err(_) => false` pattern.
- `yield_cpu` (T3): Models `ProcessManager::giveup()`. No postcondition, correct.
- `event_init` (T5): Models `event::init(hal)`. Assumption of success matches the original's
  `panic!` on failure (fail-stop semantics).
- `poll_scoreboard_full` (T1): Models `ScoreBoard::get_mut()` + `handle()`. The `!has_error`
  postcondition assumes the `unreachable!` paths don't execute — consistent with original intent.
- `drain_remaining_zombies` (T2): Models post-loop `while let`. Correct.

**Verified helpers:**
- `classify_and_check_invalid`: Verified against `spec_returns_invalid_syscall`. The hardcoded
  number sets `{1, 2}` (invalid) and `{0, 4, 6, 7, 8, 10-19, 21, 28, 30, 31}` (valid) match
  the original `match KcallNumber::from(...)` arms exactly. Cross-verified against
  `src/libs/sys/src/sys/number.rs` NR_* constants (0-31). **Correct.**
- `new_work_state`, `should_yield`, `is_initd_terminated`, `make_invalid_syscall_error`: Simple
  verified helpers with tight postconditions. All correct.
- `handle_kcall_phase`: Models dispatch+signal protocol. Postcondition correctly ties
  `kcall_handled` to `poll.has_call` and `was_invalid_syscall` to classification.
- `handle_harvest_phase`: Thin wrapper over `harvest_zombies()` preserving postconditions.
- `poll_messages_gated`: Models `cfg_if!(feature = "stdio")`. The `!stdio_enabled ==> !result`
  postcondition correctly prevents phantom messages in non-stdio builds.
- `run_iteration`: Composes all three phases. Key correctness properties verified:
  - `should_terminate ==> !work_state.harvested_process` (INITD breaks before setting flag).
  - Yield-iff-idle equivalence.
  - Harvest outcome linkage via `spec_harvest_to_outcome`.
  - Non-INITD zombies: `notify_termination` result gates `harvested_flag`.
- `run_full_iteration`: Adds yield behavior with `!result.should_terminate` guard. The doc
  comment (lines 757-764) correctly explains why this guard is needed — in the original,
  INITD termination causes `break status` before reaching yield.
- Lifecycle model (`kcall_handler_init`, `kcall_handler_lifecycle_step`, `kcall_handler_loop`):
  Well-structured inductive proof. Init establishes base case (empty history), step preserves
  invariant using real harvest outcome, loop iterates with fuel-bounded termination.

**Verdict: PASS.** All justifications are sound and well-documented.

### 4. Exec Code Faithfulness to Original Source

I verified the exec model against the original `src/kernel/src/kcall/handler.rs` (200 lines):

| Original Control Flow | Exec Model | Match? |
|----------------------|------------|--------|
| `event::init(hal)` → panic on failure | `event_init()` → assumes success | ✓ (fail-stop) |
| `loop { ... }` infinite until `break` | `kcall_handler_loop(fuel)` bounded | ✓ (fuel artifact) |
| `ScoreBoard::get_mut()` → `handle()` | `poll_scoreboard_full()` | ✓ |
| `match KcallNumber::from(...)` 21 arms | `classify_and_check_invalid()` + `dispatch_to_subsystem()` | ✓ |
| `scoreboard.handled(ret)` | `signal_handled()` | ✓ |
| `kcall_handled = true` unconditional | `kcall_phase.kcall_handled = true` | ✓ |
| `cfg_if! { if stdio ... else false }` | `poll_messages_gated(stdio_enabled)` | ✓ |
| `pm.harvest_zombies(mm)` | `harvest_zombies()` | ✓ |
| `pid == ProcessIdentifier::INITD` → `break status` | `is_initd_terminated()` → `should_terminate` | ✓ |
| `EventManager::notify_process_termination(...)` → `Ok(()) => harvested_process = true` | `notify_termination()` → bool gates `harvested_flag` | ✓ |
| `!kcall_handled && !message_received && !harvested_process` → yield | `should_yield()` | ✓ |
| Post-loop `while let Ok(Some(...))` | `drain_remaining_zombies()` | ✓ |

**Kcall number mapping verified against source:**
All 21 handler match arms correspond to the correct NR_* constants in
`sys::number::KcallNumber`. The proof lemma `lemma_dispatch_coverage_matches_source()`
cross-checks handler classification against dispatcher constants (single source of truth).

**Constant verification:**
- `SPEC_INITD_PID() == 1`: Matches `ProcessIdentifier::INITD = ProcessIdentifier(1)` in
  `src/libs/sys/src/sys/pm/pid.rs:60`.
- `SPEC_ERROR_INVALID_SYSCALL() == 88`: Matches `ENOSYS = 88` in
  `src/libs/sysapi/src/errno.rs:173` and `InvalidSysCall = ENOSYS` in
  `src/libs/error/src/lib.rs:199`.

**Subtle correctness point (INITD yield suppression):**
The `run_full_iteration` function correctly guards yield with `!result.should_terminate`
(line 765). In the original, INITD termination causes `break status` during harvest,
exiting the loop before reaching the yield check at line 187. The model's sequential
structure requires this explicit guard to match the original's control flow. This is
correctly documented in the code comment.

**Verdict: PASS.** The exec model faithfully represents the original source.

### 5. Verification Status

```
49 verified, 0 errors
Duration: 10s
Status: PASSED
```

No `assume` or `admit` statements. All `external_body` functions correspond to documented
OS/HAL dependency boundaries (T1-T5). The verification is clean.

**Verdict: PASS.**

## Issues Found

### Critical
- None.

### Minor
- None.

### Observations (Non-Issues)

1. **Fuel parameter semantic gap**: The `fuel` parameter in `kcall_handler_loop` means the model
   cannot prove the handler *always* terminates. This is explicitly documented (lines 957-962)
   and is an inherent limitation of bounded verification of an unbounded loop. The
   `lemma_oracle_connected_liveness` provides the conditional termination proof under a
   liveness assumption.

2. **Exit status unconstrained**: The exit status value is intentionally unconstrained
   because it originates from `harvest_zombies()` (T2). This is correct — the handler
   cannot verify ProcessManager state.

3. **Proof coverage is comprehensive**: The 24 proof lemmas cover dispatch totality,
   classification correctness, invalid detection, yield/idle correctness, work flag
   monotonicity, termination conditions, loop invariant (base + inductive), exec-to-spec
   bridging, constant regression, dispatch coverage cross-check, and oracle-connected
   conditional liveness. This is thorough for a shadow model verification.

## Summary

The handler exec consistency fix is well-executed. The single addition (`kcall_handler` as
`external_body`) is properly justified by the Verus-incompatible parameter types. All 22
extra functions and 8 extra structs serve clear roles in the shadow model decomposition,
with each `external_body` mapped to a specific trust boundary (T1-T5). The verified functions
have tight postconditions that faithfully encode the original's control flow semantics —
notably the INITD yield suppression and the `harvested_process` flag gating by
`notify_termination` success. Kcall number mappings, INITD PID, and ENOSYS error code all
match their source definitions. Verification passes cleanly with 49 verified, 0 errors.
