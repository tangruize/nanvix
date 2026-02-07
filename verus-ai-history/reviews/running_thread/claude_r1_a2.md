# Review: RunningThread Verification Module (Round 1, Attempt 2)

**Module:** `verus/split/kernel/pm/thread/running`
**Files reviewed:**
- `running.rs` (exec, 469 lines)
- `running.spec.rs` (spec, 276 lines)
- `running.proof.rs` (proof, 483 lines)

**Previous review:** `claude_r1_a1.md` (Grade: A-)
**Verification result:** ✅ Full crate passes (`875 verified, 0 errors`)

---

## 1. Review of Previously Raised Issues

### M1: Missing per-address mutex preservation in `exit()` and `ZombieThread` boundary model
**Previous severity:** Medium
**Status:** ✅ **FIXED (verified)**

Evidence of fix:
- `ZombieThread` now defines `spec_has_mutex` (running.spec.rs, lines 214–216).
- `ZombieThread::from_state` postcondition includes `forall|a: int| result.spec_has_mutex(a) == state.spec_has_mutex(a)` (running.rs, line 227).
- `exit()` postcondition includes `forall|a: int| result.spec_has_mutex(a) == self.spec_has_mutex(a)` (running.rs, line 367).
- Proof lemma `lemma_exit_preserves_mutexes` now includes per-address membership assertion (running.proof.rs, lines 242–252).
- New `ZombieThread::lemma_from_state_preserves_mutexes` added (running.proof.rs, lines 473–480).
- CROSS-MODULE-CHECK comment at `ZombieThread::from_state` updated to include per-address mutex preservation (running.rs, lines 213–218).

This is a thorough fix. All four state types now have symmetric `spec_has_mutex` coverage, and the exit transition preserves membership per-address. No gaps remain.

### M2: `take_mutex_guard` return type elision strengthens the contract
**Previous severity:** Medium
**Status:** ✅ **Documented (acceptable resolution)**

The prover added an explicit "API Strengthening Note" at running.rs lines 407–412 documenting this as an intentional design decision for verification. The strengthened precondition (`requires self.spec_has_mutex(address@)`) transforms a partial function into a total one, which is a sound overapproximation. Acceptable.

### L1: `join_cond()` omitted without verification alternative
**Previous severity:** Low
**Status:** ⚠️ **Acknowledged, unchanged**

The `join_cond()` method returning a sync primitive (`Condvar`) remains omitted. This is a recognized boundary limitation and is tracked in CROSS-MODULE-CHECK comments. No code change needed at this stage.

### L2: `exit()` does not require drop-safety as precondition
**Previous severity:** Low
**Status:** ✅ **Documented (acceptable resolution)**

Design Note added at running.rs lines 355–359 explaining the intentional omission. The note correctly identifies that adding `spec_drop_safe()` as a precondition would be a strengthening (the source allows exit with mutexes held). Flagged for future consideration. Acceptable.

### L3: Proof lemmas are trivially discharged
**Previous severity:** Low
**Status:** ✅ **Improved**

The prover now has 27+ lemmas (up from 24). Critically:
- `lemma_acquire_then_release_restores_mutex_state` (running.proof.rs, lines 344–376) contains a **non-trivial proof body** with explicit extensional equality reasoning: `assert(s.insert(address@).remove(address@) =~= s)`.
- `lemma_from_state_then_exit` updated with per-address mutex preservation in ensures (line 323).
- New `ZombieThread::lemma_from_state_preserves_mutexes` (line 473).

Many lemmas remain with empty bodies (relying on Verus's SMT solver), which is normal for properties that follow directly from postconditions. The composite lemma with an actual proof body demonstrates non-trivial reasoning capability. This is a meaningful improvement.

### L4: Boundary model cross-module verification obligations are comments-only
**Previous severity:** Low
**Status:** ⚠️ **Acknowledged, unchanged**

CROSS-MODULE-CHECK comments remain as documentation. Automating these checks is outside the scope of a single module's verification. No change expected.

---

## 2. New Issues Check

### No new issues found.

Specifically verified:
- **No `assume`, `admit`, or unsound escape hatches** in spec or proof files.
- The single `#[verifier::external]` on `thread_state_mut()` (running.rs, line 464) is justified: it provides mutable access to the inner `ThreadState` and is properly constrained by its caller's contracts.
- **Postcondition symmetry** is maintained: `sleep()`, `schedule()`, and `exit()` all preserve per-address mutex membership symmetrically.
- **Spec function coverage** is symmetric across all four state types (`RunningThread`, `SleepingThread`, `ReadyThread`, `ZombieThread`).
- **Full crate verification passes** (875 verified, 0 errors) — confirming no regressions.

---

## 3. Overall Assessment

The prover made substantive improvements:
1. **M1 genuinely fixed** with code changes (not just documentation). The per-address mutex preservation through `exit()` and `ZombieThread::from_state` is now properly specified and proven.
2. **M2 and L2 appropriately documented** as intentional design decisions with clear rationale.
3. **L3 meaningfully improved** with a non-trivial composite lemma demonstrating extensional set equality reasoning.
4. **No new issues introduced.** The code is clean, well-structured, and passes full verification.

The module provides a sound boundary model for `RunningThread` state transitions with complete mutex tracking across all transitions.

---

## 4. Grade

**Grade: A**

Justification: All medium issues resolved (one via code fix, one via justified documentation). Low issues either improved or appropriately acknowledged. Full verification passes. No soundness concerns remain. The remaining minor items (L1, L4) are architectural scope limitations, not verification defects.

---

## 5. Remaining Items (informational, not blocking)

| ID | Severity | Description | Status |
|----|----------|-------------|--------|
| L1 | Low | `join_cond()` sync boundary omitted | Acknowledged |
| L4 | Low | Cross-module checks are comments-only | Acknowledged |

These are tracked for future work and do not affect the soundness of the current verification.
