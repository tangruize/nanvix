# Review: ready Exec Consistency (claude-opus-4.6)

## Grade: A

## Summary

The exec consistency fix report correctly identifies all 8 functions in the original
`ReadyThread` and accounts for each one. No code changes were made — the report
documents that all AST mismatches are due to well-justified verification modeling
abstractions. The exec code faithfully represents the original source logic for all
modeled properties. Verification passes cleanly: 34 verified, 0 errors.

## Issues Found

### Critical

- None.

### Minor

1. **`thread_state_mut()` trust boundary is adequate but inherently risky.**
   The `#[verifier::external]` escape hatch is necessary (Verus cannot express
   `&mut T` returns), and the documentation (lines 530–553) clearly states the
   trust obligations. The three verified forwarding methods (`set_interrupt_reason`,
   `store_mutex_guard`, `take_mutex_guard`) partially mitigate the risk. However,
   any caller using `thread_state_mut()` for HAL operations operates entirely
   outside the verification boundary. The AUDIT note referencing specific call
   sites (line 553) is good practice.

2. **`EXIT_STATUS_INTERRUPTED()` hardcodes `4`.**
   The spec constant (ready.spec.rs:76) correctly documents the derivation chain
   (`ErrorCode::Interrupted` → `EINTR` → `4`), and I independently confirmed
   `EINTR = 4` in `src/libs/sysapi/src/errno.rs:21`. The CROSS-MODULE-CHECK
   comment is appropriate. This is not a bug but a maintenance risk if `EINTR`
   ever changes — flagging for awareness.

3. **Boundary models (`RunningThread`, `ZombieThread`) are unverified against their
   real implementations.**
   The cross-module verification obligations are well-documented (lines 173–181
   and 216–224), with explicit CROSS-MODULE-CHECK comments listing the postconditions
   that must be confirmed when those modules are independently verified. This is
   the correct approach for incremental verification but creates a trust gap until
   `running.rs` and `zombie.rs` are verified.

## Detailed Assessment

### 1. MISMATCH Functions: Properly Restored or Equivalence Documented?

All 7 original functions are accounted for:

| Function | Status | Assessment |
|----------|--------|------------|
| `new` | Equivalent | Omits `ContextInformation` and `FpuState` params (HAL boundary). Core logic identical: `ThreadState::new(...)` + `clock_now()`. ✅ |
| `from_state` | Equivalent | Uses `ThreadState` instead of `Box<ThreadState>` (transparent wrapper). Logic identical. ✅ |
| `id` | Equivalent | Body identical: `self.state.id()`. Named-return syntax is Verus-required. ✅ |
| `thread_state` | Equivalent | Body identical: `&self.state`. Named-return syntax is Verus-required. ✅ |
| `thread_state_mut` | Equivalent | Body identical: `&mut self.state`. Correctly placed outside `verus!` block with `#[verifier::external]`. ✅ |
| `admission_time` | Equivalent | Body identical: `self.admission_time`. Return type `int` vs `SystemTime` is the modeling abstraction. ✅ |
| `run` | Equivalent | Core logic preserved: `take_interrupt_reason` + `get_thread_data_area` + `RunningThread::from_state`. Omits `context_mut()` (HAL raw pointer). Returns `RunResult` struct instead of 4-tuple. ✅ |
| `terminate` | Equivalent | Uses `exit_status_interrupted_value()` helper instead of `ErrorCode::Interrupted.into()`. Semantically identical (both produce value 4). ✅ |

### 2. MISSING Functions: Added with Proper Verification?

| Function | Status | Assessment |
|----------|--------|------------|
| `join_cond` | Intentionally omitted | Returns `Condvar` (opaque sync primitive). ThreadState model excludes `join_cond` field. Properly documented in file header (line 31) and spec file (line 25). ✅ |

### 3. Equivalence Justifications Sound?

- **`Box<ThreadState>` → `ThreadState`**: Sound. `Box` is a transparent heap wrapper with no semantic impact on thread state logic.
- **`SystemTime` → `int`**: Sound. Only non-negativity is needed; scheduling ordering is out of scope.
- **`clock::now()` → `clock_now()`**: Sound. External body with minimal postcondition `result >= 0`.
- **`ErrorCode::Interrupted.into()` → constant `4`**: Sound. Verified against source: `EINTR = 4` in `errno.rs:21`.
- **4-tuple → `RunResult` struct**: Sound. The omitted `*mut ContextInformation` is a raw HAL pointer used only for assembly context switching; it cannot be meaningfully specified.
- **`Condvar` omission**: Sound. Sync primitives are outside verification scope, and the ThreadState model consistently excludes this field.

### 4. Does Exec Code Faithfully Represent the Original?

Yes. Every executable code path in the original is preserved in the Verus model:
- Construction paths (`new`, `from_state`) capture identity and timestamp.
- Accessor paths (`id`, `thread_state`, `thread_state_mut`, `admission_time`) are direct pass-throughs.
- State transitions (`run`, `terminate`) preserve the core logic and produce the correct target types.
- The three verification-only forwarding methods (`set_interrupt_reason`, `store_mutex_guard`, `take_mutex_guard`) are additive — they provide verified alternatives to `thread_state_mut()` but do not alter existing behavior.

### 5. Does Verification Pass?

Yes. `34 verified, 0 errors` confirmed by running `./verus-ai/scripts/verify.sh ready`.

## Verification-Quality Notes

- **Spec coverage is comprehensive**: well-formedness, identity preservation, interrupt reason extraction, mutex accounting, drop safety, and admission time are all specified and verified.
- **Proof coverage is thorough**: 19 lemmas covering construction, run(), terminate(), composition, and view equality.
- **Closed `wf()` predicates** follow methodology correctly — users depend on postconditions, not internal structure.
- **Documentation quality is high**: file header, trust boundary docs, cross-module obligations, and inline comments are clear and accurate.

## Recommendation

Accept. The exec code is a faithful verification model of the original with well-documented, sound modeling abstractions. The three minor items noted above are awareness points, not blockers.
