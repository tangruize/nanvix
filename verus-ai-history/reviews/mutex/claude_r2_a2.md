# Review: mutex — Round 2 Attempt 2 (claude-opus-4.6)

## Grade: A-

## Previous Review Disposition

### Issue 1 (High): `lock()` precondition requires `spec_is_unlocked()`

**Previous request:** Strengthen `lock()` with a fairness-conditioned termination argument or mark it `external_body` with a weaker postcondition.

**Status: Implicitly rejected — rejection justified.**

The prover made no changes to `lock()`. This is the correct decision. The `lock()` precondition is an inherent consequence of the `&mut self` sequential model. The two suggested alternatives would not improve soundness:
- A fairness-conditioned ghost predicate would add spec complexity without verifiable content (fairness is a liveness property requiring a concurrent model).
- Marking `lock()` as `external_body` would *reduce* verification coverage — the current version verifies the full body without escape hatches.

The existing documentation (lines 228–237 of `mutex.rs`, and the Verification Scope section) is explicit and honest about this limitation. The prover was right to leave this as-is.

### Issue 2 (Medium): Token global uniqueness not documented as trust assumption

**Status: Fixed.** T3 trust assumption added at lines 110–115 of `mutex.rs`. The documentation is well-written, accurately identifies the gap (`pub ghost view` allows external construction), and proposes a concrete future mitigation (singleton token pattern or resource algebra).

**Verified:** Lines 110–115 of `mutex.rs` contain T3. The corresponding spec comment at lines 72–76 of `mutex.spec.rs` also acknowledges the per-instance limitation. These are consistent.

### Issue 3 (Medium): `unlock()` condvar notification error path undocumented

**Status: Fixed.** Comment added at lines 265–267 of `mutex.rs`: "The original `notify_first()` error path (handled with `warn!()` in `Drop`) is not modeled; condvar notification is an external dependency verified separately."

**Verified:** The comment accurately describes the original behavior (line 203 of `src/kernel/src/pm/sync/mutex.rs`: `warn!("failed to unlock mutex ...")`) and the scoping decision.

### Issue 4 (Medium): `try_lock()` return type diverges from original's `Result` pattern

**Status: Not changed — acceptable.**

The prover kept the `(bool, Tracked<Option<MutexToken>>)` return type. On re-examination, this is a reasonable Verus idiom:
- The postconditions (`result.0 ==> result.1@.is_some()` and `!result.0 ==> result.1@.is_none()`) fully specify the correlation between the bool and the Option.
- Any caller that misuses the tuple (e.g., ignoring the bool) would fail Verus verification at the call site.
- The `(bool, Tracked<Option<...>>)` pattern avoids complications with `Result` and `Tracked` type interactions in Verus.

The original concern about preventing misuse is addressed by the postconditions — misuse is caught at verification time, not prevented at the type level. This is adequate for a verified codebase where all callers are also verified.

Downgraded from Medium to **non-issue**. Retained as a suggestion below.

### Issue 5 (Low): `is_locked()` not in API mapping table

**Status: Fixed.** Added at line 67 of `mutex.rs`: `| (none in original) | is_locked(&self) | Verification-only helper. |`

### Issue 6 (Low): Trivial proof lemma names overstate contribution

**Status: Fixed.** Renamed:
- `lemma_try_lock_unlocked_succeeds` → `lemma_unlocked_implies_not_locked`
- `lemma_try_lock_locked_fails` → `lemma_locked_implies_locked`

**Verified:** The ensures clauses are unchanged (lines 62–67 and 70–76 of `mutex.proof.rs`). Only the function names and doc comments were updated. The new names accurately describe what the lemmas prove: definitional unfolding of the `spec_is_unlocked`/`spec_is_locked` predicates.

## New Issues Introduced

(none)

No regressions detected. The changes are purely additive (documentation, renames) with no modifications to spec functions, exec code, or postconditions.

## Remaining Suggestions (Non-blocking)

- **`try_lock()` return type:** Consider returning `Result<Tracked<MutexToken>, ()>` in a future iteration to match the original API ergonomics. This is a stylistic preference, not a correctness concern — the current postconditions prevent misuse at verification time.

## Verification Soundness Assessment

- **Verification conditions:** 27 verified, 0 errors.
- **Escape hatches:** None. No `assume`, `external_body`, `trusted`, or `admit` usage.
- **Spec/exec separation:** Clean three-file split (`mutex.rs`, `mutex.spec.rs`, `mutex.proof.rs`).
- **Well-formedness preservation:** `wf()` is established by `new()` and preserved by `try_lock()`, `lock()`, and `unlock()` — confirmed in all postconditions.
- **Token-ownership protocol:** `MutexToken` is created only on successful lock acquisition, bound to the mutex instance via view identity (`token.view == self@`), and consumed by `unlock()`. Double-unlock is precondition-blocked (`lemma_no_double_unlock`). Cross-instance token use is precondition-blocked (`lemma_token_instance_isolation`).
- **Proof coverage:** 20 proof lemmas covering definitional properties (9), protocol properties (6), and safety properties (5). All are automatically discharged.

## Positive Observations

- **Responsive to review feedback.** Four of six issues were directly addressed with precise, minimal changes. The two unaddressed items were correctly identified as either inherent limitations or acceptable design choices.
- **Zero escape hatches** remains the strongest quality signal. All 27 VCs are fully solver-discharged.
- **Documentation quality is exemplary.** The module header (116 lines) provides API mapping, divergence documentation, trust boundaries, trust assumptions (now T1–T3), and verification scope. This is production-grade verification documentation.
- **Honest about limitations.** The documentation does not overclaim. The Verification Scope section explicitly lists six categories of out-of-scope behavior.

## Summary

The prover addressed all actionable items from the previous review. The T3 trust assumption, condvar comment, lemma renames, and API mapping update were all implemented correctly. The two items not changed (`lock()` precondition, `try_lock()` return type) were appropriately left as-is — the former is an inherent model limitation and the latter is a reasonable Verus idiom with verification-time safety.

The mutex verification is sound within its stated scope: it proves sequential state machine correctness of the lock/unlock protocol with no escape hatches, comprehensive proof coverage, and production-grade documentation. No blocking issues remain.
