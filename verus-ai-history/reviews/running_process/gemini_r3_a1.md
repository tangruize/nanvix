# Review: running_process (gemini-3-pro-preview)

## Grade: A

## Issues Found

### Low
- **Dependency on External Body**: `interrupted_resume` is declared as `external_body`. While standard for modular verification, the soundness of `RunningProcess` properties depends entirely on the correctness of `InterruptedProcess::resume` and its spec. This is a cross-module dependency that must be tracked.
- **Model vs Implementation**: The verified code is a model that uses `u64` counters and ghost sequences to represent the original `NonEmptyVecDeque` collections. While this accurately captures the state machine logic, it relies on the correctness of the manual translation between the original Rust code and this verification model (e.g., assuming `ready_count > 0` correctly maps to `ready.is_some()`).

## Positive Observations
- **Comprehensive Coverage**: All methods in the original source are modeled and verified, including complex state transitions like `sleep` and `exit`.
- **Precise Specifications**: The specs track the exact content of all thread lists (ready, interrupted, sleeping, zombie), proving that threads are never lost or duplicated during transitions.
- **Bug Fix Verification**: The verification effort successfully identified a logic bug in `exit_thread` (losing zombie threads) which has been fixed in the original source.
- **PID Immutability**: The verification explicitly proves that the Process ID (PID) is preserved across all operations.
- **Oracle Usage**: The use of oracle parameters for `wakeup` and `try_join_thread` is well-designed, allowing verification of search-dependent logic by pushing the search correctness burden to the caller (checked via preconditions).
- **Documentation**: The verified file contains excellent documentation explaining the model, assumptions, and the mapping to the original code.

## Summary
The verification of `running_process` is of high quality. It uses a faithful model to verify the complex state transitions of a running process, ensuring that thread lists are managed correctly and process identity is preserved. The specifications are strong, covering not just safety (no crashes) but functional correctness (exact list content). The discovery and fix of a bug in the original source demonstrates the value of this verification. The use of a model (counters + ghost sequences) is appropriate for this level of abstraction.
