# Review: interrupted (claude-opus-4.6) — Round 2, Attempt 2

## Grade: A

## Previous Issues — Resolution Status

### HIGH: `thread_state_mut()` is `#[verifier::external]` (interrupted.rs:240)

**Status: RESOLVED**

The prover addressed this thoroughly. The function now includes:
- A `# Trust Boundary` section explaining the Verus limitation (`&mut T` return).
- **Intended postconditions** (identity and wf preservation) documented as comments (lines 226–228).
- **Known call sites** enumerated: `ThreadRefMut::Interrupted` in `pm/thread/mod.rs` and `pm/process/manager/mod.rs` accessing `fpu_state_mut()` (lines 230–235).
- Migration note for when Verus adds `&mut T` support (lines 239).

**Verification of claims:** I confirmed the call site documentation is accurate:
- `mod.rs:128` dispatches `ThreadRefMut::Interrupted(thread) => thread.thread_state_mut()` ✓
- `manager/mod.rs:1154,1169` access only `fpu_state_mut()` through the returned reference ✓
- FPU state is indeed not part of the verification model, so these mutations cannot affect `wf()` or `spec_id()` ✓

The suggested debug assertions were not added, which is acceptable since the actual mutation path (FPU state) doesn't interact with the verified invariants.

### MEDIUM: `join_cond()` omitted from verified code

**Status: RESOLVED (alternative approach)**

The prover chose not to add an `#[verifier::external_body]` stub (as originally suggested) but instead documented the omission comprehensively in the spec file's Trust Assumptions (spec.rs lines 26–30):

> "Since the `Condvar` type does not exist in the verification model, no boundary spec can be provided here."

**Assessment:** This rejection is justified. An `external_body` stub would require referencing `Condvar` in the return type, but `Condvar` is not modeled anywhere in the verification framework. A stub returning a non-existent type would be misleading rather than helpful. The documentation approach is the right trade-off: it makes the omission explicit without introducing a meaningless type declaration.

### MEDIUM: `ReadyThread` boundary model omits `admission_time`

**Status: RESOLVED**

The prover added:
- A `**Cross-module dependency**` comment (exec lines 83–85) requiring confirmation when ReadyThread is independently verified.
- An `**Out of scope**` comment (exec lines 87–91) explaining the omission is intentional (scheduling property, not safety/identity property).
- Corresponding Trust Assumptions entry in the spec file (spec.rs lines 41–45).

**Verification of claims:** The real `ReadyThread` does hold `admission_time: SystemTime` (ready.rs:51), set to `clock::now()` in `from_state` (ready.rs:113). The claim that this is purely a scheduling property is accurate — it doesn't affect identity, well-formedness, or interrupt reason correctness.

### MEDIUM: `set_interrupt_reason` accepts any `int`

**Status: RESOLVED**

Documented in Trust Assumptions (spec.rs lines 32–36): validity is enforced at the `InterruptedThread` level via `wf()`, and only `resume()` calls `set_interrupt_reason` with `self.reason` (guaranteed valid by `self.wf()`).

**Verification of claims:** The original `set_interrupt_reason` at `state.rs:204` takes `InterruptReason` (enum), not `int`. Only one call site exists in `interrupted.rs:114` (`self.state.set_interrupt_reason(self.reason)`). The constraint enforcement chain is sound: `wf()` ⇒ `spec_valid_reason(self.reason)` ⇒ reason is valid when passed to `set_interrupt_reason` in `resume()`.

### LOW: `InterruptedThread` struct fields are `pub`

**Status: RESOLVED**

Comprehensive comment added (exec lines 65–69) explaining the Verus limitation, warning against direct construction, and specifying the `wf()` establishment requirement for proof contexts.

### LOW: Proof resume lemmas manually reconstruct post-state

**Status: RESOLVED (no change needed)**

The NOTE comment at proof.rs lines 77–81 was already adequate per the previous review. No change was requested.

### LOW: `InterruptReason` modeled as `int` rather than Verus `enum`

**Status: RESOLVED (no change needed)**

Previous review acknowledged this was cosmetic and the `int` + validity predicate approach is sound. No change was requested or needed.

## New Observations

### Positive Additions

- **Enriched spec surface:** New spec functions added — `spec_is_killed`, `spec_is_timed_out`, `spec_state_interrupt_reason`, `spec_locked_mutex_count`, `spec_drop_safe`, `spec_has_mutex`, `spec_kernel_stack`, `spec_user_stack`. These provide a complete accessor layer for downstream modules.
- **Stronger frame conditions:** New `lemma_resume_preserves_stacks` (proof.rs:165–178) proves that resume preserves kernel and user stack ownership — strengthening the original resume frame conditions.
- **ReadyThread completeness:** New `lemma_from_state_preserves_interrupt_reason` (proof.rs:244–251) completes the ReadyThread boundary lemma set (identity + wf + interrupt reason).
- **Trust Assumptions section:** The spec file now has a comprehensive, well-organized Trust Assumptions block (lines 22–45) covering all four trust boundary items. This is excellent practice for verification projects.

### New Issues

(none)

No new bugs, unsoundness, or documentation gaps were introduced by the fixes. The code is cleaner and more complete than the previous version.

## Remaining Trust Boundary Items

These are inherent Verus/design limitations, not fixable issues:

| Item | Reason | Documentation |
|------|--------|---------------|
| `thread_state_mut` is `#[verifier::external]` | Verus does not support `&mut T` returns | exec.rs lines 215–243, spec.rs lines 37–40 |
| `join_cond()` omitted | `Condvar` not modeled in verification | exec.rs line 40, spec.rs lines 26–30 |
| `InterruptReason` as `int` | Design choice (sound with validity predicate) | spec.rs lines 10–14 |

All three are thoroughly documented and do not represent verification bugs.

## Summary

All seven issues from the previous review (1 high, 3 medium, 3 low) have been resolved — five through substantive code/documentation changes and two that required no action. The prover's one deviation from suggestion (documentation instead of `external_body` stub for `join_cond`) is well-justified.

The verification is sound: the core safety property (interrupt reason propagation through `resume()`) is fully machine-checked. Frame conditions are now stronger with the addition of stack preservation. The trust boundary is comprehensively documented with accurate call-site information (verified against source). No new issues were introduced.

Upgraded from A- to A. The remaining trust boundary items (external annotation, Condvar omission, int modeling) are inherent limitations documented to the highest practical standard.
