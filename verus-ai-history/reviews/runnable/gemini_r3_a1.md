# Review: runnable (gemini-3-pro-preview)

## Grade: C

## Issues Found

### Critical
- **Implementation vs Model Discrepancy (Ghost Code)**
  - **Location**: `RunnableProcess` struct and all functions (exec).
  - **Description**: The verified code is an abstract model, not the actual implementation. The original `RunnableProcess` stores threads in `NonEmptyVecDeque`, but the verified version replaces these with `Ghost<Seq<int>>`. The `RunningProcess` return type of `run()` is entirely ghost, meaning execution data is lost.
  - **Impact**: The verification proves protocol correctness (state machine transitions) but does not verify memory safety, container integrity, or actual thread data management. The verified code cannot replace the original code in the kernel.
  - **Suggested Fix**: This requires a significant refactor to verify the actual data structures (`VecDeque`) and thread types, or explicit acknowledgement that this is a model-only verification.

### High
- **Missing Exec Functions**
  - **Location**: `find_thread`, `find_thread_mut`, `earliest_admission_time`.
  - **Description**: These public functions exist in the original code but are omitted from the verified exec module (only spec versions exist).
  - **Impact**: Verified callers cannot query the process state (e.g., to find a thread or check admission time).
  - **Suggested Fix**: Implement these functions in the verified module, potentially requiring verified views for the returned references or `SystemTime`.

### Medium
- **Oracle Parameter in `wakeup`**
  - **Location**: `wakeup` function.
  - **Description**: The function requires a `found: bool` oracle parameter because it cannot search the ghost sleeping list at runtime.
  - **Impact**: The correctness of the search is delegated to the caller via precondition, rather than being verified within the function. This differs from the original behavior where `wakeup` performs the search.
  - **Suggested Fix**: If the implementation is made concrete (non-ghost), the search can be performed and verified internally.

### Low
- **Implicit Return Types**
  - **Location**: `run` function.
  - **Description**: Original `run` returns a tuple including `InterruptReason` and `ContextInformation`. Verified `run` returns a `RunningProcess` struct where `interrupt_reason` is a ghost field and context info is omitted.
  - **Suggested Fix**: Document clearly that HAL types are abstracted away at this boundary.

## Positive Observations
- **Strong State Machine Verification**: The verification successfully proves that PID is preserved and that thread counts/transitions adhere to the protocol across all operations.
- **Oracle-Free Logic**: `run` and `terminate` correctly use proofs (lemmas) and exec-level counters to derive decisions (min index, branching) without needing oracle parameters, which is an improvement over typical initial verification attempts.
- **Clean Separation**: Specs and proofs are well-organized and separated from the (model) code.

## Summary
The verification of `runnable` is a high-quality **abstract model** of the component, proving that the state machine logic is correct. However, it fails to verify the **actual implementation** as it abstracts away the concrete data structures (vectors of threads) into ghost sequences. Consequently, it does not guarantee memory safety or correctness of the running kernel code. To achieve "Level A" verification, the concrete data structures must be restored and verified.
