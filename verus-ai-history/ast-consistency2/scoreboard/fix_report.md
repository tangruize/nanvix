# Exec Consistency Fix: scoreboard

## Summary
- Mismatches fixed: 0
- Missing functions added: 0
- Documented equivalences: 8 (6 functions + 2 structs)

## Root Cause

The AST diff compared `src/kernel/src/kcall/mod.rs` against
`verus/split/kernel/kcall/mod.rs`, but the verus version intentionally split
the scoreboard code into a separate `scoreboard.rs` file for verification
modularity. All 6 functions and 2 structs reported as MISSING_IN_VERUS exist
in `verus/split/kernel/kcall/scoreboard.rs` with full verification (71
verified, 0 errors). No exec code changes are needed.

## Changes

| Function/Struct | Action | Justification |
|-----------------|--------|---------------|
| `KcallArgs` (struct) | Documented equivalence | Present in `scoreboard.rs` lines 220–235. Fields use `i32` instead of `ProcessIdentifier`/`ThreadIdentifier` (which are `#[repr(C)]` wrappers around `i32` via `From<i32>`). Semantically identical. |
| `ScoreBoard` (struct) | Documented equivalence | Present in `scoreboard.rs` lines 294–310. Mutex/Semaphore modeled as `locked: bool`, `dispatched_value: u8`, `handled_value: u8`. Phase tracking added as explicit `ScoreBoardPhase` enum. `completed_cycles: u64` is verification-only state. Global singleton modeled via `ScoreBoardSlot`. |
| `fmt` (Debug for KcallArgs) | Out of scope | Formatting impl with no safety implications. Documented in scoreboard.rs API Mapping table (line 78). |
| `get_mut` [get_mut_source.rs](get_mut_source.rs) | Documented equivalence | Modeled by `ScoreBoardSlot::try_get_board()` (lines 1066–1077) and `ScoreBoardSlot::get_board()` (lines 1091–1100). `try_get_board()` returns `false` when uninitialized, modeling `Err(ErrorCode::TryAgain)`. `get_board()` returns `&ScoreBoard` when initialized, modeling the `Ok` path. Note: original returns `&'static mut`; Verus cannot model `&mut` returns from global state (documented in Trust Boundary T1). |
| `dispatch` [dispatch_source.rs](dispatch_source.rs) | Documented equivalence | Present in `scoreboard.rs` lines 787–916. The original acquires a mutex, sets args, signals `dispatched.up()`, waits on `handled.down()`, and returns the result. The verus version models this as a full cycle with environment parameters (`lock_acquired`, `up_failed`, `down_interrupted`, `handler_progress`) to verify all error paths. The split API (`begin_dispatch`, `complete_dispatch`, `abandon_dispatch`) is also available for fine-grained phase reasoning. Core exec logic (set args → signal → wait → read result) is faithfully modeled. |
| `handle` [handle_source.rs](handle_source.rs) | Documented equivalence | Present in `scoreboard.rs` lines 513–528. Original: `self.dispatched.try_down()?; Ok(&self.args)`. Verus: sets `dispatched_value = 0`, transitions phase to `Dispatched`, returns `Ghost(self.args@)`. Takes `&mut self` instead of `&self` (documented divergence: required to model signal consumption; original uses atomic `try_down`). Error path modeled by `try_handle()` (lines 546–567). |
| `handled` [handled_source.rs](handled_source.rs) | Documented equivalence | Present in `scoreboard.rs` lines 680–696. Original: `self.ret = ret; self.handled.up()`. Verus: `self.result = ret; self.handled_value = 1; self.phase = ScoreBoardPhase::Handled`. Direct semantic mapping: stores result and signals completion. |
| `init` (standalone) | Documented equivalence | Original: `info!("..."); ScoreBoard::init()`. Modeled by `ScoreBoardSlot::init()` (lines 1025–1035) which calls `ScoreBoard::new()` and sets `initialized = true`. The `info!()` logging is out of scope for verification (line 79 in scoreboard.rs). `ScoreBoard::init()` in the original sets `SCOREBOARD = Some(ScoreBoard { ... })`; the verus model faithfully reproduces the initial field values: `pid: i32::MAX`, `tid: i32::MAX`, all args `0`, semaphores at `0`, result `ok()`. |

## File Organization

The original `src/kernel/src/kcall/mod.rs` contains both module declarations
and the `ScoreBoard` implementation inline. The verus version splits this into:

- `mod.rs` — Module declarations (`pub mod dispatcher; pub mod handler; pub mod scoreboard;`)
- `scoreboard.rs` — ScoreBoard verification model with exec, spec, and proof code
- `scoreboard.spec.rs` — View types, spec functions, state transition specifications
- `scoreboard.proof.rs` — Proof lemmas for protocol correctness

This split is a standard verification practice for separating concerns and does
not change exec semantics. All original exec logic is accounted for in
`scoreboard.rs`.

## Verification: PASS

```
verification results:: 71 verified, 0 errors
Duration: 10s
Module: kernel::kcall::scoreboard
```
