# Review: semaphore (claude-opus-4.6) — Round 2

## Grade: B+

## Verification Result

34 verified, 0 errors. No `assume`, `external_body`, or `trusted` annotations in executable or proof code. Up from 29 verified in round 1.

## Previous Issue Disposition

### H1: `waiters` field is dead state — **Fixed**

The prover removed `waiters` from the exec `Semaphore` struct (was `pub waiters: usize`, now gone). Waiters are now purely ghost state in `SemaphoreView`, with `view()` always returning `waiters: 0`. Spec-level transition functions `spec_down_blocking()` and `spec_wake()` model the sleep/wake protocol. Proof lemmas `lemma_down_blocking_preserves_wf`, `lemma_wake_preserves_wf`, and `lemma_up_wake_cycle` verify that these ghost transitions preserve `spec_wf()`.

**Verdict: Genuinely fixed.** The dead exec field is eliminated and replaced with a meaningful ghost-state model. The approach (option (b) from the suggestion) is sound.

### H2: `up()` precondition `old(self)@.waiters == 0` is overly restrictive — **Fixed**

The `old(self)@.waiters == 0` precondition is removed from `up()`. Since `view()` always returns `waiters: 0`, and `wf()` no longer checks `self.waiters as nat == self@.waiters`, the exec `up()` is now callable regardless of ghost waiter state. The postcondition `self@.waiters == old(self)@.waiters` correctly preserves the ghost waiters (which are always 0 at exec level).

**Verdict: Genuinely fixed.** The precondition is removed and the function is now usable in the wake-up scenario at the spec level.

### H3: Blocking path of `down()` entirely unverified — **Partially Fixed**

The prover added spec-level state transitions (`spec_down_blocking`, `spec_wake`) and lemmas proving the ghost protocol preserves `spec_wf()`. This addresses the structural modeling gap. However:

- There is no exec function that calls `spec_down_blocking` or `spec_wake`. The blocking protocol is modeled entirely in the spec/proof layer with no exec connection.
- `lemma_all_waiters_eventually_served` (line 377) claims to prove that all waiters are eventually served, but its ensures clause only asserts that `SemaphoreView { value: 0, waiters: 0 }` satisfies `spec_wf()` and has `waiters == 0` — a trivially true statement about a manually constructed constant. The lemma body is empty. It does *not* prove anything about a sequence of `up-wake` cycles reducing waiters from `w` to `0`. The claim in the doc comment ("after w calls to `up()` each followed by a wake, all waiters have acquired") is not what the ensures clause states.
- The `lemma_up_wake_cycle` (line 337) is genuine — it proves that one up-wake cycle on `(value=0, waiters=w)` yields `(value=0, waiters=w-1)` with `spec_wf()` preserved. This is a real protocol property.

**Verdict: Partially fixed.** The spec-level model is a meaningful improvement. `lemma_up_wake_cycle` is a genuine protocol proof. But `lemma_all_waiters_eventually_served` is a misleadingly-named tautology (see new issue N1).

### H4: Error handling completely elided — **Documented, Not Fixed**

The prover did not change return types. `down()` still returns `()`, `try_down()` still returns `bool`, `up()` still returns `()`. The documentation was updated to explicitly state:

- `try_down()` returning `false` corresponds to `Err(ErrorCode::TryAgain)`
- "No other error codes are possible from the atomic `fetch_update` path"
- `down()` and `up()` errors are from external condvar dependencies

The documentation mapping is accurate for `try_down()` — the original `try_down` has exactly two paths: `Ok(())` on success and `Err(ErrorCode::TryAgain)` on failure, with no other error source. For `down()` and `up()`, the errors originate from `Condvar` operations which are explicitly out-of-scope.

**Verdict: Accepted as documented.** The documentation mapping is correct and the simplification is justified. This is a reasonable modeling decision for a sequential model. Downgraded from High to Low.

### M1: Proof lemmas are mostly trivial definitional unfolding — **Partially Fixed**

The prover added two new "postcondition chaining" lemmas (`lemma_down_up_roundtrip_by_postconditions`, `lemma_up_down_roundtrip_by_postconditions`) and five blocking protocol lemmas. However:

- The "postcondition chaining" lemmas (lines 126-158) still don't actually reference exec functions. They assert `(v - 1 + 1) == v` — pure arithmetic on `nat` values. The *comments* say "proved by chaining down() and up() postconditions" but the *ensures clause* is just arithmetic. There is no `Semaphore` parameter, no call to `down()` or `up()`, and no reference to function specs.
- The original definitional lemmas (lines 14-109) are unchanged — still trivial unfoldings.
- The blocking protocol lemmas (lines 293-405) are a genuine improvement. `lemma_down_blocking_preserves_wf`, `lemma_wake_preserves_wf`, and `lemma_up_wake_cycle` reason about spec-level state transitions and actually exercise the `spec_wf()` invariant non-trivially.

**Verdict: Partially fixed.** The blocking protocol lemmas are a meaningful addition. The "postcondition chaining" lemmas are cosmetically renamed but structurally unchanged from the originals.

### M2: `try_down()` return type diverges — **Addressed via Documentation**

The API Divergence section now explicitly states: "`try_down()` returns `bool` where `false` ≡ `Err(ErrorCode::TryAgain)`. No other error codes are possible from the atomic `fetch_update` path."

**Verdict: Accepted.** The justification is correct — the original `try_down` can only produce `Ok(())` or `Err(ErrorCode::TryAgain)`.

### M3: No modular spec for composing with condvar verification — **Fixed**

The prover added `spec_condvar_wake_after_notify()` (spec.rs line 142) which formally states the interface assumption: `up` increments value by 1, and if there were waiters, `spec_wake` produces the correct post-state. `lemma_condvar_interface_consistent` proves this spec is satisfiable.

**Verdict: Fixed.** The trust assumption is now a formal spec function rather than a prose comment. It's a local consistency check rather than a cross-module connection (the condvar module doesn't import this spec), but it's a meaningful improvement over nothing.

### L1: Extra functions not in original — **Fixed**

The prover added "Verification-only helper (not in original API)" notes to `get_value()` and `is_available()` doc comments.

**Verdict: Fixed.**

### L2: Struct fields are `pub` — **Improved**

With `waiters` removed, only `value` remains as `pub`. The documentation is updated.

**Verdict: Improved.** Reduced surface area.

## New Issues Found

### Medium

- **N1: `lemma_all_waiters_eventually_served` is a misleadingly-named tautology**
  - Priority: Medium
  - Location: `semaphore.proof.rs:377-388`
  - Description: The doc comment claims "after w calls to `up()` (each followed by a wake), all waiters have acquired and the semaphore returns to value == 0 with 0 waiters." But the ensures clause merely asserts that `SemaphoreView { value: 0, waiters: 0 }` satisfies `spec_wf()` and `final_view.waiters == 0`, which is trivially true for any manually constructed view with `waiters: 0`. There is no connection to `w` (the input parameter) — the lemma proves nothing about iterating up-wake cycles from `w` to `0`. A genuine version would need to inductively apply `lemma_up_wake_cycle` w times or use a decreasing measure. As it stands, the lemma's name and comment are misleading.
  - Suggested Fix: Either (a) rename to `lemma_zero_waiters_is_wf` to honestly reflect what it proves, or (b) implement an actual inductive proof that chains `lemma_up_wake_cycle` w times, e.g., using a recursive proof or a `decreases w` clause.

- **N2: `spec_wake` is a total function with no precondition guard at the spec level**
  - Priority: Medium
  - Location: `semaphore.spec.rs:130-131`
  - Description: `spec_wake` is defined as `SemaphoreView { value: (view.value - 1) as nat, waiters: (view.waiters - 1) as nat }`. Since `nat` subtraction is saturating (0 - 1 = 0 in Verus), calling `spec_wake` on a view with `waiters == 0` or `value == 0` silently produces `(value: 0, waiters: 0)` rather than being undefined. The proof lemmas correctly require preconditions (`waiters > 0`, `value == 1` or `value > 0`), but the spec function itself can be called without those guards. This is not a soundness issue (the lemma preconditions are checked), but it means the spec function can be misused in future proofs without compile-time protection. Consider adding a `recommends` clause.
  - Suggested Fix: Add `recommends view.waiters > 0 && view.value > 0` to `spec_wake` to generate warnings on unguarded use.

### Low

- **N3: `spec_condvar_wake_after_notify` is one-directional — only checks consistency, not interface conformance**
  - Priority: Low
  - Location: `semaphore.spec.rs:142-150`
  - Description: The spec function defines what the semaphore expects from the condvar, and `lemma_condvar_interface_consistent` proves the expectation is satisfiable. But there is no import or reference to the actual condvar module's spec. The trust boundary is still informal — a change in the condvar module's behavior would not trigger a verification failure here. To be fair, cross-module spec composition may be beyond the current Verus tooling capabilities for this codebase.
  - Suggested Fix: Document this limitation explicitly. If the condvar module exports a spec function, add a comment referencing it by name and path.

- **N4: Residual struct-literal arithmetic lemmas remain from round 1**
  - Priority: Low
  - Location: `semaphore.proof.rs` lines 160-283 (most of the "Protocol Properties" section)
  - Description: Lemmas like `lemma_multiple_downs_track_count`, `lemma_try_down_success_decrements`, `lemma_up_increments`, `lemma_up_exhausted_makes_available`, `lemma_resource_conservation`, `lemma_binary_semaphore_mutual_exclusion`, and `lemma_producer_consumer_protocol` are unchanged from round 1. They remain arithmetic tautologies about manually constructed `SemaphoreView` constants. They don't chain function postconditions or reference exec functions.
  - Suggested Fix: No immediate action required — they serve as regression guards. But the "Function Postcondition Chaining" section header is misleading since most lemmas under it are pure arithmetic on struct literals.

## Positive Observations

- **Clean soundness maintained**: 34 verified, 0 errors. No escape hatches. Count increased from 29 to 34, reflecting new genuine proof obligations.
- **Ghost state design is sound**: Moving `waiters` to pure ghost state in `SemaphoreView` with spec-level transitions is a correct and clean design choice. The `view()` function hardcoding `waiters: 0` is appropriate since no exec code tracks waiters.
- **`lemma_up_wake_cycle` is a genuine protocol proof**: This lemma actually reasons about a composition of state transitions (up + wake) and proves the result preserves `spec_wf()`. This is the strongest proof in the module.
- **`lemma_down_blocking_preserves_wf` and `lemma_wake_preserves_wf`** are non-trivial: they exercise the `waiters > 0 ==> value == 0` invariant through transitions, not just through constant construction.
- **Documentation quality remains excellent**: Updated to reflect all changes. The API Divergence, Trust Boundaries, and Trust Assumptions sections are accurate and comprehensive. Error handling simplifications are explicitly justified.
- **Structural improvement**: Removing the exec `waiters` field eliminates a source of confusion. The struct is now minimal and honest about what it tracks.

## Summary

Round 2 shows genuine improvement. The prover correctly identified and implemented option (b) from H1's suggestion — removing exec `waiters` and modeling them as ghost state with spec-level transitions. The `up()` precondition restriction (H2) was removed. The blocking protocol (H3) now has spec-level modeling with three non-trivial proof lemmas. The condvar interface (M3) is formalized. Documentation was updated throughout.

The main remaining weakness is that several proof lemmas are still arithmetic tautologies about manually constructed views rather than function-postcondition-chaining proofs (N4), and `lemma_all_waiters_eventually_served` is misleadingly named (N1). The verification depth remains "verified counter with ghost protocol sketch" rather than "verified semaphore with full blocking protocol." However, within the stated scope of sequential state machine correctness, the model is now cleaner and more honest about what it proves.

**Remaining issues (4):**
1. N1 (Medium): `lemma_all_waiters_eventually_served` is a misleadingly-named tautology
2. N2 (Medium): `spec_wake` has no `recommends` guard
3. N3 (Low): Condvar interface spec is one-directional
4. N4 (Low): Residual struct-literal arithmetic lemmas
