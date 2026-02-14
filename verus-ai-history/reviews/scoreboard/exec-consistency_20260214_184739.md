# Review: scoreboard Exec Consistency (claude-opus-4.6)

## Grade: A

## Verification Status

**PASSED**: 71 verified, 0 errors. Duration: 10s.

## Issues Found

### Critical

- None.

### Minor

1. **`handle()` receiver divergence (`&self` → `&mut self`) is well-documented but inherent.**
   The original `handle(&self)` uses atomic `try_down()` requiring no exclusive access. The Verus model uses `&mut self` to model signal consumption. This is a fundamental modeling limitation (Verus cannot reason about atomics) and is correctly documented in Trust Boundary T4 and the API Divergence section (scoreboard.rs lines 151–156, 90–91). No action needed, but it means the sequential model *assumes* mutual exclusion between `handle()` and `dispatch()` rather than *proving* it.

2. **`KcallResult` flattening (`enum` → `struct { is_success, value }`) loses type-level variant distinction.**
   The original uses `KcallResult::Success(KcallSuccess(i64))` / `KcallResult::Error(KcallError(i32))`. The Verus model uses a flat struct with `is_success: bool` + `value: i64`. The `wf()` invariant (scoreboard.spec.rs lines 217–219) correctly constrains error values to i32 range, and `lemma_error_wf_constrains_range` proves this. This is a sound abstraction.

3. **`completed_cycles: u64` is verification-only state with no original counterpart.**
   This field tracks protocol progress for inductive proofs. The overflow guard (`completed_cycles < u64::MAX`) is required as a precondition on `complete_dispatch()` and `dispatch()`. This is a reasonable verification artifact that does not affect exec faithfulness. Properly documented in the API Divergence section (scoreboard.rs lines 104–106).

## Criterion-by-Criterion Assessment

### 1. Were all MISMATCH functions properly restored or equivalence documented?

**Yes.** The fix report identifies 0 mismatches requiring code changes. The AST diff flagged items as MISSING_IN_VERUS because it compared against `verus/split/kernel/kcall/mod.rs` (which contains only module declarations) rather than `scoreboard.rs` (which contains the actual implementation). This is a false positive from the diff tool, not a real consistency issue.

### 2. Were MISSING functions added with proper verification?

**Yes — all 6 functions and 2 structs are present in `scoreboard.rs` with full verification.**

| Original | Verus Location | Verified |
|----------|---------------|----------|
| `KcallArgs` struct | scoreboard.rs:220–235 | ✓ (View impl, new(), lemmas) |
| `ScoreBoard` struct | scoreboard.rs:294–310 | ✓ (View impl, wf(), all methods) |
| `fmt` (Debug) | Not modeled | Correctly out of scope (formatting) |
| `ScoreBoard::init()` | `ScoreBoard::new()` (line 423) + `ScoreBoardSlot::init()` (line 1025) | ✓ |
| `ScoreBoard::get_mut()` | `try_get_board()` (line 1066) + `get_board()` (line 1091) | ✓ |
| `ScoreBoard::dispatch()` | `dispatch()` (line 787) + split API (`begin_dispatch`, `complete_dispatch`, `abandon_dispatch`) | ✓ |
| `ScoreBoard::handle()` | `handle()` (line 513) + `try_handle()` (line 546) | ✓ |
| `ScoreBoard::handled()` | `handled()` (line 680) | ✓ |
| `pub fn init()` | Not modeled | Correctly out of scope (logging wrapper) |

### 3. Are equivalence justifications sound?

**Yes.** Each justification in the fix report accurately describes the relationship between original and Verus code:

- **`KcallArgs`**: Using `i32` instead of `ProcessIdentifier`/`ThreadIdentifier` is correct because these are `#[repr(C)]` wrappers around `i32` with `From<i32>` accepting any value. The View type maps to `int`/`nat` for spec-level reasoning.

- **`ScoreBoard`**: The structural remodeling (Mutex → `locked: bool`, Semaphore → `dispatched_value: u8`/`handled_value: u8`, phase enum, cycle counter) faithfully captures the protocol state machine. The `wf()` invariant correctly encodes: (a) result well-formedness, (b) semaphore values match phase expectations, (c) mutex held iff non-Idle.

- **`dispatch()`**: The monolithic `dispatch()` at line 787 correctly models all four outcomes of the original: lock failure, up failure, down interruption (with 3 handler-progress sub-cases), and success. The `handler_progress` parameter (0–2) elegantly models nondeterministic handler advancement during concurrent execution. The inline handler steps (lines 883–893, 899–904) match the sequential composition of `handle()` + `handled()`.

- **`handle()`**: Returns `Ghost<KcallArgsView>` instead of `Result<&KcallArgs, Error>`. The `try_handle()` variant covers the error path. The `get_args()` method provides reference access post-transition. This decomposition is necessitated by Verus's `&mut self` requirement.

- **`handled()`**: Direct mapping: `self.result = ret; self.handled_value = 1; self.phase = Handled` corresponds to `self.ret = ret; self.handled.up()`.

- **`init()` / `get_mut()`**: `ScoreBoardSlot` correctly models the `Option<ScoreBoard>` global pattern with `initialized: bool` + `board: ScoreBoard`. `init()` is idempotent (matching the original's unconditional overwrite). `try_get_board()` models the `ErrorCode::TryAgain` failure path.

### 4. Does the exec code now faithfully represent the original source?

**Yes, with documented and justified divergences.** The exec code in `scoreboard.rs` captures all observable behaviors of the original:

- **Initialization**: `ScoreBoard::new()` produces the same initial field values as the original `ScoreBoard::init()` (pid=i32::MAX, tid=i32::MAX, args=0, semaphores=0, result=ok()). Verified by `lemma_init_is_wf`.

- **Dispatch protocol**: The four-phase state machine (Idle → Signaled → Dispatched → Handled → Idle) correctly models the original's mutex-lock → set-args → dispatched.up() → handled.down() → read-result → unlock flow.

- **Error paths**: All three error paths from the original `dispatch()` are modeled:
  - `lock()` failure → `DispatchOutcome::LockFailed` (state preserved)
  - `dispatched.up()` failure → `DispatchOutcome::UpFailed` (args written, no signal, lock released)
  - `handled.down()` interruption → `DispatchOutcome::DownInterrupted` (stuck state characterized per handler progress)

- **Handle try_down**: `try_handle()` returns false when not Signaled, modeling `ErrorCode::TryAgain`.

- **Semaphore value preconditions**: `lemma_semaphore_up_dispatched_cannot_fail` and `lemma_semaphore_up_handled_cannot_fail` prove that `up()` is always called with value 0, preventing overflow.

### 5. Does verification still pass?

**Yes.** 71 verified, 0 errors. All exec functions, spec functions, and proof lemmas verify successfully.

## Trust Boundaries Assessment

The five trust boundaries (T1–T5) are well-identified and honestly documented:

- **T1** (global singleton): `static mut` memory safety is unverifiable in Verus; the init/access pattern is modeled.
- **T2** (mutex correctness): Assumed, separately verified.
- **T3** (semaphore correctness): Assumed, separately verified. Value preconditions *are* proven here.
- **T4** (sequential ordering): Sound refinement argument provided (mutex + semaphore enforce total order within cycles). The `handle(&self)` → `handle(&mut self)` change is honestly flagged.
- **T5** (error handling): All error paths are modeled with state preservation proofs.

## Proof Coverage Assessment

The proof file provides comprehensive coverage:

- **Initialization**: 3 lemmas (init wf, default args, default result).
- **State transitions**: 4 lemmas (one per transition).
- **Semaphore protocol**: 2 lemmas (dispatched signal/consume, handled signal/consume).
- **Protocol correctness**: 5 lemmas (full cycle, result integrity, args integrity, mutex during active, cycle counter monotonicity).
- **Multi-cycle**: 3 lemmas (n-cycle induction, two-cycle composition, injectivity).
- **Invalid transitions**: 3 lemmas (no dispatch when active, idle state clean, signaled has pending signal).
- **Error paths**: 10 lemmas (try_handle fail/idle/success, lock failure, semaphore up cannot fail ×2, abandon not-wf, abandon preserves data, dispatch success/interrupted/up-failed equivalence).
- **Abandon per-phase**: 4 lemmas (signaled, dispatched, handled, unified).
- **Split API**: 1 lemma (composition equals full cycle).
- **ScoreBoardSlot**: 5 lemmas (new uninitialized, init valid, try_get iff init, uninitialized fails, initialized ready, reinit fresh).
- **KcallArgs/KcallResult**: 5 lemmas (equal fields, ok valid, success always wf, error constrains range, equal fields).

## Summary

The scoreboard exec consistency fix is correct and thorough. No actual code changes were needed — the AST diff tool produced false positives because it compared against `mod.rs` (module declarations only) rather than `scoreboard.rs` (where the implementation lives). All 6 original functions and 2 structs are faithfully modeled in the Verus verification with well-documented divergences where Verus limitations require different modeling choices (atomic operations, `&mut` returns, `static mut` globals). The verification passes with 71 verified properties and 0 errors. The proof coverage is comprehensive, covering initialization, all state transitions, semaphore protocol correctness, error path preservation, multi-cycle properties, and the global singleton pattern. The trust boundaries are honestly identified with a sound refinement argument for the sequential model.
