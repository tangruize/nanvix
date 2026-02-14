# Review: runnable Exec Consistency (claude-opus-4.6)

## Grade: A

## Verification Result

```
verification results:: 70 verified, 0 errors
Duration: 10s
```

Verification passes with 70 verified items, 0 errors. Consistent with the fix report's claim of improvement from 67 to 70 verified items.

## Issues Found

### Critical

- None.

### Major

- None.

### Minor

1. **`state()` return type divergence**: The exec model returns `ProcessIdentifier` (by value) while the original returns `&ProcessState`. The fix report documents this as a Verus limitation. The justification is sound — only the PID is tracked in this verification model — but callers of `state()` in the original code may access fields beyond PID (e.g., `vmem`). This is acceptable at the current abstraction level but should be noted as a **cross-module verification obligation** for when `ProcessState` is independently verified. The documentation in the exec file (lines 867–886) correctly flags this.

2. **`earliest_admission_time()` omits `unwrap_or(clock::now())` fallback**: The fix report correctly identifies this as dead code given the `NonEmptyVecDeque` invariant. The exec model's `wf()` precondition (`ready_thread_ids@.len() >= 1`) makes this sound. However, the original code structurally contains the fallback — if the `NonEmptyVecDeque` invariant were ever relaxed upstream, this model would silently diverge. The documentation (lines 888–895) correctly notes this.

3. **`wakeup()` uses `Result<RunnableProcess, RunnableProcess>` vs original `Result<Self, Self>`**: These are semantically identical since `Self = RunnableProcess`. No issue.

4. **`interrupt_reason` hardcoded to `0i64` in `run()`** (line 519): The original `run()` returns an `Option<InterruptReason>` from `next_thread.run()`, which carries thread-specific state. Setting this to `0i64` is acceptable since the spec documents this field as "unconstrained at this abstraction level" (spec line 152, exec line 627–641). The `spec_run()` view function also uses `0int` (spec line 640). This is consistent and documented.

### Informational

1. **Struct fields are `pub`**: The exec model uses `pub` fields for Verus proof ergonomics (exec line 109–110), while the original has private fields with getter/setter methods. This is a necessary and well-documented deviation for verification.

2. **`TerminateResult` enum vs `Result<InterruptedProcess, ZombieProcess>`**: The exec uses a custom enum while the original uses `Result`. These are isomorphic — `TerminateResult::Interrupted` maps to `Ok(InterruptedProcess)` and `TerminateResult::Zombie` maps to `Err(ZombieProcess)`. The postconditions correctly constrain both branches, and bridging lemmas (`lemma_terminate_interrupted_view_eq`, `lemma_terminate_zombie_view_eq`) prove equivalence to the view-level spec transitions.

3. **`sleeping_count` decrement in `wakeup()`** (exec line 848): `sleeping_count: self.sleeping_count - 1` — this is safe because the proof block (line 717–718) asserts `self.sleeping_count >= 1u64` when `found` is true, preventing underflow. Correct.

## Criterion Assessment

### 1. Were all MISMATCH functions properly restored or equivalence documented?

**Yes.** All four reported mismatches (`new`, `from_state`, `run`, `terminate`) are documented with detailed equivalence justifications in the fix report. The type abstractions are sound:
- `ReadyThread` → `(i64, i64)` for (tid, admission_time): captures the two properties relevant to scheduling.
- `Box<ProcessState>` → `ProcessIdentifier`: captures the identity-tracking property.
- `NonEmptyVecDeque<T>` → `Vec<i64>` with `wf()` enforcing `len() >= 1`: sound isomorphism.
- `Option<NonEmptyVecDeque<T>>` → `Vec<i64>` with empty representing `None`: sound isomorphism.

The algorithmic logic of each function is verified to match:
- **`new()`**: Creates process with one ready thread, empty other lists. ✓
- **`from_state()`**: Assigns all fields from parameters. ✓
- **`run()`**: Linear scan for earliest admission time, remove selected, create RunningProcess. The loop invariant ties `min_idx` to `spec_min_index_rec` at each iteration, and the proof block confirms `min_idx == spec_earliest_ready_index()` post-loop. ✓
- **`terminate()`**: Ready→zombie, sleeping→interrupted, branch on existence. The exec-level counters replace `Option::take()` pattern matching but `wf()` ties them to vector lengths. ✓

### 2. Were MISSING functions added with proper verification?

**Yes.** Two missing functions were added:
- **`earliest_admission_time()`** (exec lines 899–946): Implements a linear min-scan loop with proper invariants. The proof block invokes `lemma_min_index_rec_bounds` and `lemma_earliest_ready_index_bounds` to tie the loop result to `spec_earliest_admission_time()`. The postcondition ensures `result == self.spec_earliest_admission_time()` and `result >= 0i64`. Verified.
- **`state()`** (exec lines 881–886): Returns `ProcessIdentifier` copy. Simple, correct postcondition ties result to `self@.pid`. Verified.

### 3. Are equivalence justifications sound?

**Yes.** Each justification follows the established type abstraction pattern:
- HAL boundary types (`ContextInformation`, `VirtualAddress`, `Vmem`, `InterruptReason`) are correctly omitted — they carry no protocol-level invariants relevant to process state management.
- The `unreachable!()` branch in original `run()` is correctly absent because `wf()` guarantees `ready_thread_ids@.len() >= 1`, making the branch provably unreachable.
- The `exec-level counters` pattern (`interrupted_count`, `sleeping_count`) for `terminate()` is sound because `wf()` enforces `counter as nat == vec@.len()`, making the branch decision equivalent to the original's `Option::take()` + `match`.
- The `vec_search()` approach for `wakeup()` replaces the original `remove_if()` closure — both perform a linear search for a matching thread ID. The exec implementation correctly handles both found and not-found cases.

### 4. Does the exec code now faithfully represent the original source?

**Yes**, with documented and justified abstractions. Function-by-function comparison:

| Original Function | Exec Model | Faithful? |
|---|---|---|
| `new(pid, ready_thread, vmem)` | `new(pid, ready_tid, ready_time)` | ✓ (Vmem omitted, thread decomposed) |
| `from_state(state, ready, interrupted, sleeping, zombie)` | `from_state(pid, ready_ids, ready_times, interrupted_ids, sleeping_ids, zombie_ids, ...)` | ✓ (types abstracted) |
| `state()` → `&ProcessState` | `state()` → `ProcessIdentifier` | ✓ (returns tracked property) |
| `state_mut()` → `&mut ProcessState` | Omitted | ✓ (Verus limitation, documented) |
| `run()` | `run()` | ✓ (same algorithm, HAL types omitted) |
| `terminate()` | `terminate()` | ✓ (same logic, different result type encoding) |
| `wakeup(tid)` | `wakeup(tid)` | ✓ (same search+move logic) |
| `add_thread(ready_thread)` | `add_thread(ready_tid, ready_time)` | ✓ (thread decomposed) |
| `earliest_admission_time()` | `earliest_admission_time()` | ✓ (dead fallback omitted) |
| `find_thread(tid)` | Spec-only | ✓ (Verus limitation, documented) |
| `find_thread_mut(tid)` | Omitted | ✓ (Verus limitation, documented) |

### 5. Does verification still pass?

**Yes.** `70 verified, 0 errors` confirmed by running `./verus-ai/scripts/verify.sh runnable`.

## Additional Observations

### Proof Quality

The proof file is comprehensive with 70 verified lemmas/functions covering:
- Construction well-formedness (`lemma_new_is_wf`, `lemma_new_has_one_ready`).
- PID immutability across all operations.
- Thread count preservation for `run()`, `wakeup()`.
- Correct branching in `terminate()` for all three cases (no interrupted/sleeping, interrupted only, sleeping only).
- Min-index correctness via inductive proof (`lemma_min_index_rec_bounds`).
- Content-level postconditions specifying exact list contents after each operation.
- View-level bridging lemmas connecting exec results to spec-level abstract state transitions.

### Trust Boundary Documentation

The trust boundary is clearly documented in both exec (lines 44–68) and spec (lines 52–106) files. Key trust assumptions:
- Thread ID ownership/disjointness from Rust's type system.
- HAL boundary types are opaque.
- `clock_now()` returns non-negative values.
- Cross-module obligations for `state_mut()` callers.

### Spec-Level Abstract State Transitions

The spec file provides a complete set of abstract state transition functions (`spec_new`, `spec_from_state`, `spec_run`, `spec_terminate_to_interrupted`, `spec_terminate_to_zombie`, `spec_wakeup`, `spec_add_thread`) that downstream modules can use for compositional verification. Each has a corresponding bridging lemma in the proof file.

## Summary

The exec consistency fix is thorough and well-executed. All original functions are either faithfully modeled with sound type abstractions, added as missing exec implementations with proper verification, or documented as intentionally omitted with clear Verus-limitation justifications. The verification passes cleanly with 70 verified items. The documentation quality is high, with trust boundaries, cross-module obligations, and equivalence rationales clearly stated. The only minor concern is the inherent risk of model drift if upstream types change (e.g., `NonEmptyVecDeque` invariant relaxation), but this is a general verification modeling concern rather than a deficiency in this specific fix.
