# Review: running_thread (claude-opus-4.6) — Round 3, Attempt 3

## Grade: A-

## Change Delta

The three module files (running.rs, running.spec.rs, running.proof.rs) are **identical** to the versions reviewed in claude_r3_a2.md. No code, specification, proof, or documentation changes were made. Since the previous review returned PASSED: YES with 0 remaining issues, an unchanged submission is the expected and correct response.

## Re-verification of Previous Assessment

To maintain rigor, I re-verified the key claims from the previous review rather than rubber-stamping:

### API Coverage: Confirmed

Original `RunningThread` has 10 public functions. The verified model accounts for all 10:
- **7 verified:** `from_state`, `sleep`, `schedule`, `id`, `thread_state`, `put_mutex_guard`, `take_mutex_guard`
- **1 external:** `thread_state_mut` (Verus `&mut T` limitation, documented trust obligations)
- **1 omitted:** `join_cond` (Condvar sync boundary, documented)

### Specification Soundness: Confirmed

- All spec functions (`spec_id`, `spec_locked_mutex_count`, `spec_has_mutex`, `spec_drop_safe`, `spec_is_interrupted`, `spec_interrupt_reason`, `spec_user_tda`, `spec_kernel_stack`, `spec_user_stack`, `wf`) delegate directly to `ThreadState` spec functions. No semantic mismatches.
- `wf()` is `self.state.wf()`, which requires `locked_mutex_set@.finite() && locked_mutex_set@.len() == locked_mutex_count as nat`. This correctly ties the ghost model to the runtime counter.

### Postcondition Completeness: Confirmed

Each state transition (`sleep`, `schedule`, `exit`) ensures all of: state view preservation, identity preservation, mutex count preservation, per-address mutex non-interference, drop safety preservation, and target well-formedness. The `exit()` function additionally ensures status capture; `sleep()` additionally ensures alarm capture.

### Trust Assumptions: Confirmed Sound

- **T1 (no double-lock):** `!old(self).spec_has_mutex(address@)` on `put_mutex_guard`. Justified: double-locking a non-recursive mutex deadlocks, so this condition always holds in correct executions.
- **T2 (release-what-you-hold):** `old(self).spec_has_mutex(address@)` on `take_mutex_guard`. Justified: releasing an unheld mutex is a bug; the original's `None` return is defensive, not a valid code path.
- **HAL boundary:** `*mut ContextInformation` returns omitted from `sleep`/`schedule`/`exit`. Justified: raw pointers cannot be modeled in Verus.
- **Sync boundary:** `join_cond()` omitted. Justified: `Condvar` is opaque.
- **External escape:** `thread_state_mut()` is `#[verifier::external]`. Justified: Verus cannot express `&mut T` return types.

### Proof Lemmas: Confirmed

The proof file contains 28 lemmas across 4 impl blocks. All are structurally sound:
- Construction lemmas (4): wf, id, drop safety, mutex preservation.
- Identity correctness (1).
- Per-transition lemmas (6 each × 3 transitions = 18): id, wf, mutexes, drop safety, state view, plus alarm/status capture where applicable.
- Composite lemmas (3): `lemma_new_is_drop_safe`, `lemma_from_state_then_schedule`, `lemma_from_state_then_exit`.
- View equality (1) and acquire-then-release roundtrip (1).

The roundtrip lemma (`lemma_acquire_then_release_restores_mutex_state`) is the only non-trivial proof, using set extensionality (`s.insert(x).remove(x) =~= s` for `x ∉ s`). This is correct.

## Known Limitations (unchanged, not blocking)

1. `take_mutex_guard` eliminates `None` path (trust assumption T2).
2. Cross-module boundary model consistency is comment-based, not mechanically checked.
3. `thread_state_mut` is `#[verifier::external]` (Verus tool limitation).
4. `join_cond()` omitted (Condvar cannot be modeled).
5. `*mut ContextInformation` returns omitted (raw pointer, HAL boundary).

These are all documented design decisions or Verus tool limitations. None are defects.

## Summary

No changes since the previous review, which is appropriate given the previous PASSED verdict with 0 remaining issues. Independent re-verification confirms the verification model is sound, API coverage is complete, specifications are faithful to the original semantics, and all trust assumptions are well-justified and documented. The grade remains A- due to the inherent limitations of the verification model (external escape hatch, boundary models without mechanical cross-module checking), which are tool constraints rather than proof quality issues.
