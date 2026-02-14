# Review: semaphore Exec Consistency (claude-opus-4.6)

## Grade: A

## Issues Found

### Critical

- None.

### Minor

1. **`down()` and `down_available()` are identical.** Both functions have the same signature, preconditions, postconditions, and body (`self.value = self.value - 1`). The fix report lists `down()` as "Added to Verus" (MISSING → added) and `down_available()` as "Justified extra," but they are code duplicates. This is not a correctness issue — both are verified — but the redundancy is unnecessary. `down()` could delegate to `down_available()` or vice versa.

2. **`view()` always returns `waiters: 0`.** The `View` implementation (spec.rs:347) hardcodes `waiters: 0`, meaning exec-constructed views never reflect waiter state. This is documented in the "Ghost State Architecture" section and is intentional (waiters are pure ghost state), but it means `wf()` can never observe `waiters > 0` through the `@` operator on an exec `Semaphore`. The blocking protocol lemmas operate on manually constructed `SemaphoreView` values, which is correctly documented.

3. **`up()` return type divergence not fully justified.** The original `up()` returns `Result<(), Error>` because `notify_first()` can fail. The fix documents this under T5 ("notify success assumed"), but the Verus model silently drops error propagation. If `notify_first()` fails in practice, the semaphore value is incremented but no waiter is woken — a thread could sleep indefinitely. The trust assumption is documented but the practical consequence (potential liveness failure) could be more prominently flagged.

4. **`value` field is `pub`.** The exec struct declares `pub value: usize` (line 204), violating Nanvix coding standards requiring private fields with getter/setter access. This is documented as a Verus tooling constraint (line 193–194), which is a reasonable justification, but worth noting.

### Observations (Non-Issues)

- No `assume`, `admit`, or `external_body` directives are present in any of the three files.
- All 45 verification conditions pass.
- The consistency report accurately describes 3 mismatches documented as equivalent, 1 missing function added, and 4 justified extras.

## Criteria Assessment

### 1. Were all MISMATCH functions properly restored or equivalence documented?

**Yes.** Three mismatches are documented:

| Function | Mismatch | Equivalence Justification |
|----------|----------|--------------------------|
| `new` | `AtomicUsize::new(value)` → `value`, `Condvar::new()` omitted | Follows from struct divergence; both initialize count identically. |
| `try_down` | `&self` → `&mut self`, `Result<(), Error>` → `bool` | Decision logic identical: `if value > 0 { value -= 1; success } else { fail }`. Formal mapping via `spec_try_down_result_maps_ok()`. |
| `up` | `&self` → `&mut self`, `fetch_add(1)` → `self.value += 1`, `notify_first()` modeled by `spec_wake()` | Core increment logic identical. Return type difference documented under T5. |

Each equivalence justification is sound and connects back to the sequential modeling decision.

### 2. Were MISSING functions added with proper verification?

**Yes.** The `down()` function was added with:
- Preconditions: `wf()`, `spec_is_available()`, `ctx@.safe_for_down()`.
- Postconditions: value decremented by 1, waiters unchanged, `wf()` preserved.
- Body: `self.value = self.value - 1`.
- Verification: passes as part of the 45 verified conditions.

The function models the instant-success path of the original `down()` loop. The blocking path is covered by `down_or_block()` and spec-level transitions.

### 3. Are equivalence justifications sound?

**Yes.** The justifications follow a consistent pattern:
- `AtomicUsize` → plain `usize` is forced by Verus limitations and documented as sequential model.
- `&self` → `&mut self` follows from replacing atomics with sequential mutation.
- `Condvar` omission is compensated by spec-level `spec_down_blocking()` and `spec_wake()`.
- `Result` → simpler return types (`bool`, `()`) are formally mapped via spec predicates.

The refinement argument (lines 149–163) provides an informal linearizability connection between the sequential model and the concurrent original, which is appropriate given Verus's limitations.

### 4. Does the exec code now faithfully represent the original source?

**Yes, within the documented modeling constraints.** The exec code captures all three original functions (`new`, `down`, `try_down`, `up`) with correct decision logic. The additional helper functions (`down_available`, `down_or_block`, `get_value`, `is_available`) are properly justified as verification decompositions. The API mapping table (lines 66–74) and divergence documentation (lines 77–91) clearly establish the correspondence.

### 5. Does verification still pass?

**Yes.** 45 verified, 0 errors. No `assume`, `admit`, or `external_body` directives. The fix report claims +1 from the new `down` function (44 → 45), consistent with the current result.

## Summary

The exec consistency fix is well-executed. All original functions are represented in the Verus model with correct decision logic, and divergences are thoroughly documented with sound justifications. The sequential modeling approach (replacing `AtomicUsize`/`Condvar` with plain `usize`/`&mut self`) is the correct design choice given Verus's limitations, and the trust boundaries and assumptions are explicitly enumerated. The blocking protocol is modeled at the spec level with proven lemmas for state machine correctness. The only notable redundancy is between `down()` and `down_available()`, which could be consolidated but does not affect correctness. Verification passes cleanly with 45 conditions and no proof shortcuts.
