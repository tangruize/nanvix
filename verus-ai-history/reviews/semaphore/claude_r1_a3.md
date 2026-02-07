# Review: semaphore (claude-opus-4.6) — Round 3

## Grade: A-

## Verification Result

35 verified, 0 errors. No `assume`, `external_body`, or `trusted` annotations. Up from 34 in round 2, reflecting the new inductive lemma replacing the trivial one.

## Previous Issue Disposition

### N1: `lemma_all_waiters_eventually_served` is a misleadingly-named tautology — **Fixed**

The prover chose option (b): implemented a genuine inductive proof with `decreases w`. Key changes:

1. **New spec function** `spec_after_n_up_wake_cycles` (spec.rs:184-194): recursively models n up-wake cycles. Each iteration constructs `after_up = { value: view.value + 1, waiters: view.waiters }`, applies `spec_wake`, then recurses with `n - 1`. This correctly models the protocol: each cycle increments value, then wake decrements both value and waiters.

2. **Revised ensures clause** (proof.rs:383-389): now states `final_view == (SemaphoreView { value: 0, waiters: 0 })` where `final_view = Semaphore::spec_after_n_up_wake_cycles(initial, w)` and `initial = SemaphoreView { value: 0, waiters: w }`. This is genuinely non-trivial — it connects the recursive spec function's output to the expected terminal state.

3. **Proof body** (proof.rs:392-407): calls `lemma_up_wake_cycle(initial)` to establish one cycle, asserts the intermediate state, then recurses on `w - 1` (guarded by `w > 1`). The proof mirrors the spec function's recursion, which is the correct Verus pattern for inductive proofs.

**Verification of correctness:** Tracing `spec_after_n_up_wake_cycles({0, w}, w)`:
- Step 1: after_up = {1, w}, after_wake = spec_wake({1, w}) = {0, w-1}, recurse with ({0, w-1}, w-1)
- Step k: after_up = {1, w-k+1}, after_wake = {0, w-k}, recurse with ({0, w-k}, w-k)
- Step w: n=0, returns view = {0, 0} ✓

The recursive call's precondition `w - 1 > 0` is correctly guarded by `if w > 1`. The `lemma_up_wake_cycle` preconditions (`spec_wf(v)`, `v.value == 0`, `v.waiters > 0`) are satisfied at each step because `spec_wf({0, w}) = (w > 0 ==> 0 == 0) = true` and `w > 0` from the loop invariant.

**Verdict: Genuinely fixed.** This is now a real inductive proof that chains `lemma_up_wake_cycle` through the recursive spec function. The ensures clause proves a non-trivial property about the composition of w state transitions.

### N2: `spec_wake` has no `recommends` guard — **Fixed**

The prover added (spec.rs:130-133):
```
pub open spec fn spec_wake(view: SemaphoreView) -> SemaphoreView
    recommends
        view.waiters > 0,
        view.value > 0,
```

This is exactly the suggested fix. The `recommends` clause will generate Verus warnings if `spec_wake` is called in a context where these conditions aren't established, providing defensive protection against future misuse.

**Side effect check:** `spec_wake` is called inside `spec_after_n_up_wake_cycles` (spec.rs:191). At each recursive step, `after_up.value = view.value + 1 >= 1 > 0` ✓, and `after_up.waiters = view.waiters`. When the recursion reaches `n == 0`, the base case returns without calling `spec_wake`, so `waiters` is always > 0 when `spec_wake` is invoked within the recursive unrolling used by the proof. No recommends violations introduced.

**Verdict: Genuinely fixed.**

### N3: `spec_condvar_wake_after_notify` is one-directional — **Fixed**

The prover added a `# Limitation` documentation section (spec.rs:147-156) that explicitly states:
- "This spec defines the semaphore's *expectation* of condvar behavior."
- "It is not imported by the condvar module (`kernel::pm::sync::condvar`)"
- "Changes to the condvar implementation will not trigger a verification failure here."
- References `verus/split/kernel/pm/sync/condvar.spec.rs` and `spec_notify_all_result`.

I verified that `verus/split/kernel/pm/sync/condvar.spec.rs` exists and exports `spec_notify_all_result` among other spec functions. The reference is accurate.

**Verdict: Genuinely fixed.** The limitation is honestly documented with specific cross-references. This was a documentation-level suggestion and is fully addressed.

### N4: Residual struct-literal arithmetic lemmas — **Fixed**

The section header was renamed from "Function Postcondition Chaining" to "Arithmetic Lemmas" (proof.rs:112) with an honest description: "The following lemmas reason about value arithmetic matching the structure of exec function postconditions. They operate on nat values and SemaphoreView struct literals, serving as regression guards."

**Verdict: Fixed.** The section is now honestly labeled.

## New Issues Found

### Low

- **N5: Round-trip lemma doc comments remain slightly aspirational**
  - Priority: Low
  - Location: `semaphore.proof.rs:125, 146`
  - Description: The doc comments for `lemma_down_up_roundtrip_by_postconditions` (line 125) and `lemma_up_down_roundtrip_by_postconditions` (line 146) still say "Proved by chaining `down()` and `up()` postconditions." While the arithmetic they prove (`(v-1)+1 == v`) does mirror the postcondition structure, the lemmas don't reference any function specs, take no `Semaphore` parameter, and don't invoke any exec or spec functions. The word "chaining" implies a compositional proof connecting function contracts, which this is not. This is a minor residual of N4 — the section header was fixed but individual doc comments were not updated to match the new honest framing.
  - Suggested Fix: Change doc comments to "Arithmetic identity matching the combined effect of `down()` then `up()` postconditions." or similar. Very minor, no action required.

## Positive Observations

- **Genuine inductive proof:** `lemma_all_waiters_eventually_served` is now the strongest lemma in the module. It uses `decreases w`, calls `lemma_up_wake_cycle` at each step, and proves a non-trivial property about the composition of w state transitions through a recursive spec function. This is a significant upgrade from the trivial tautology it replaced.
- **Clean new spec function:** `spec_after_n_up_wake_cycles` is well-designed — it correctly models the protocol steps, uses `decreases n` for termination, and its recursive structure naturally enables the inductive proof.
- **Verification count increased:** 29 → 34 → 35 across three rounds, reflecting genuinely new proof obligations at each step.
- **No regressions:** All previously verified properties remain verified. The `recommends` addition to `spec_wake` did not break any existing proofs.
- **Documentation quality:** The condvar limitation documentation is thorough and honest. The section header rename accurately describes the lemma category.
- **Sound ghost state architecture:** The overall design (exec `value` + ghost `waiters` + spec transitions + inductive proof) forms a coherent verification of the semaphore protocol's state machine properties.

## Summary

Round 3 represents a clear improvement over Round 2. All four N-series issues were genuinely addressed:

- **N1** was the most significant: the trivially true tautology was replaced with a real inductive proof using a new recursive spec function. The ensures clause now connects to the spec function's recursive evaluation rather than asserting a trivially true constant. This is the single biggest quality improvement across all three rounds.
- **N2** added the `recommends` guard as suggested, with no side effects.
- **N3** and **N4** were documentation improvements, both correctly implemented.

The module's verification depth now includes: exec function correctness (new, down, try_down, up), definitional properties (state totality, complementarity, view consistency), arithmetic regression guards, non-trivial blocking protocol lemmas (down_blocking, wake, up_wake_cycle), a genuine inductive proof over the complete waiter-draining protocol, and a formalized condvar interface assumption with documented limitations.

The only remaining issue is a cosmetic doc comment inaccuracy in two round-trip lemmas (N5, Low priority). The verification is sound, honest about its scope, and free of escape hatches.

**Remaining issues (1):**
1. N5 (Low): Round-trip lemma doc comments slightly aspirational
