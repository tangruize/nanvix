# Review: kcall_scoreboard (gemini-3-pro-preview)

## Grade: A

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- None.

### Low
- **API Divergence in `dispatch` Signature**: The verified `dispatch` function takes `args: KcallArgs` and `ret: KcallResult` as inputs, whereas the original takes individual scalar arguments and returns the result produced by the handler. This is well-documented as a modeling choice for sequential verification (where `ret` represents the non-deterministic handler output), but it represents a structural difference from the source.
- **`ScoreBoard::get_mut` return type**: The original returns `Result<&'static mut ScoreBoard, Error>`, while the verification model uses `ScoreBoardSlot` with `try_get_board` (returning bool) and `get_board` (returning `&ScoreBoard`). Mutable access is achieved via the `board` field directly in the verified model. This is a necessary adaptation for Verus but constitutes an API divergence.

## Positive Observations
- **Soundness**: No `assume` or `external_body` directives were found in the verified code (except in documentation comments). All properties are proven.
- **Comprehensive Documentation**: The "Trust Boundaries", "API Divergence", and "Verification Model" sections in `scoreboard.rs` are exemplary. They clearly state what is verified and what is assumed, providing a strong refinement argument for the sequential model.
- **Robust State Machine**: The four-phase protocol (`Idle` -> `Signaled` -> `Dispatched` -> `Handled` -> `Idle`) is strictly enforced by the `ScoreBoardPhase` enum and transition lemmas.
- **Error Path Modeling**: The verification correctly models failure paths for lock acquisition (`lock_acquired: bool` parameter) and semaphore wait interruption (`down_interrupted: bool` parameter), proving that state is preserved or fails safely to a known state (`abandon_dispatch`).
- **Liveness/Progress**: The `completed_cycles` ghost counter and associated lemmas prove that the protocol makes progress (cycles are monotonically increasing), addressing liveness properties.
- **Data Integrity**: Lemmas explicitly prove that arguments passed by the dispatcher are received by the handler (`lemma_args_integrity`) and the result set by the handler is received by the dispatcher (`lemma_result_integrity`).

## Summary
The verification of `kcall_scoreboard` is of high quality. It successfully abstracts the concurrent rendezvous protocol into a sequential state machine that Verus can verify, while accurately modeling the control flow, error handling, and data integrity of the original implementation. The use of ghost state (`completed_cycles`) to prove inductive properties about protocol progress is a strong feature. The separation of spec, proof, and exec code is clean and follows best practices. The trust boundaries (mutex, semaphore, sequential execution) are clearly articulated and reasonable for this component.
