# AI-Augmented Formal Verification of an OS Kernel: Memory Management and Process Scheduling

**Abstract**

Formal verification of systems software has traditionally required person-years of expert effort, limiting its practical adoption. We present an AI-augmented verification methodology that combines frontier LLMs as specification synthesizers and proof constructors (the *Prover*) with multiple LLM reviewers that iteratively challenge and improve verification completeness. Our approach is grounded in three practical principles: (1) direct synthesis—leveraging frontier AI's ability to generate complete initial proofs without extensive scaffolding, (2) iterative refinement—using multi-model review to systematically identify gaps until convergence, and (3) trust decomposition—minimizing human involvement to specification semantic review while mechanical proof checking remains fully automated. We apply this methodology to verify the memory management and process scheduling subsystems of Nanvix, a microkernel targeting serverless computing. Through iterative verification across 43 reviewed modules, we verified ~14,300 lines of original Rust source code, producing 76,694 lines of Verus verification code (a 5.3× expansion) with 1,857 mechanically proven properties, 1 assume, and 134 external_body annotations. The verification uncovered 11 previously unknown bugs—including critical arithmetic overflows, state machine logic errors, and a scheduler fairness flaw. Human effort was limited to approximately two person-weeks of specification review and intervention. Our experience demonstrates that AI-driven verification can achieve substantial productivity improvements over traditional methods and can scale from library-level components to complex kernel subsystems including concurrent scheduling.

---

## 1. Introduction

### 1.1 The Verification Cost Crisis

Operating system kernels are among the most critical software components, yet formal verification of kernel code remains prohibitively expensive. Landmark projects such as seL4 [1] required over 11 person-years to verify 10,000 lines of C code. This high barrier has limited formal verification to well-funded, safety-critical projects, leaving the vast majority of systems software without mathematical correctness guarantees.

The bottleneck is not proof checking—modern SMT solvers verify proofs in seconds—but rather the *intellectual labor* of:
1. **Specification authoring**: Formalizing what "correct" means.
2. **Invariant discovery**: Finding properties that hold across all states.
3. **Proof construction**: Guiding the solver through complex reasoning.

Recent advances in large language models (LLMs) suggest these cognitive tasks may be automatable. Frontier models achieve 93.9% on VeruSage [3], demonstrating genuine proof construction ability. However, naively prompting an LLM to "verify this code" is insufficient—the model may produce incomplete specifications, skip difficult functions, or subtly alter semantics.

### 1.2 Key Insight: Prover-Reviewer Iteration

We observe that effective AI-driven verification requires structured interaction between complementary roles:

- **Prover**: An AI agent that synthesizes specifications and constructs proofs.
- **Reviewers**: Multiple AI agents that critically examine the prover's output for gaps, unsoundness, and semantic drift.

This structure addresses the core challenge: a single AI may produce plausible but incomplete verifications. By introducing independent reviewers—especially from different model families—we create constructive tension that drives toward completeness.

The methodology is simple but effective:
1. The **Prover** generates an initial verified implementation.
2. Multiple **Reviewers** independently identify issues and assign quality grades.
3. The Prover addresses all issues or justifies rejection.
4. Iteration continues until all reviewers assign their highest grade (A+).

### 1.3 Key Finding: Direct Synthesis Works

An important empirical finding shaped our methodology. We initially attempted a *scaffolded approach*:

```
┌──────────────────────────────────────────────────────────────────────────────┐
│              Initial Scaffolded Approach (Abandoned)                         │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Step 1        Step 2        Step 3         Step 4         Step 5            │
│  ┌──────┐      ┌─────┐      ┌─────┐       ┌─────┐       ┌─────┐            │
│  │Code  │ ──▶  │Docs │ ──▶  │Spec │ ──▶   │Proof│ ──▶   │Debug│            │
│  │Under-│      │Write│      │Write│       │Skel-│       │Until│            │
│  │stand │      │     │      │     │       │eton │       │Pass │            │
│  └──────┘      └─────┘      └─────┘       └─────┘       └─────┘            │
│                                                                              │
│  ~2 hours per module, high prompt complexity                                 │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

However, we discovered that frontier models can *directly synthesize* a complete initial verification in a single prompt, then refine through iterative review:

```
┌──────────────────────────────────────────────────────────────────────────────┐
│              Final Direct Synthesis Approach (Adopted)                        │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│  Step 1                              Step 2                                   │
│  ┌────────────────────┐            ┌────────────────────┐                    │
│  │ Direct Synthesis   │ ──────▶   │ Multi-Model Review │                    │
│  │ (Spec + Proof +    │            │ (Iterative         │                    │
│  │  Debug in one shot)│            │  Refinement)       │                    │
│  └────────────────────┘            └────────────────────┘                    │
│                                                                               │
│  ~30 min initial + ~2 hours review = faster & simpler                         │
│                                                                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Insight**: The scaffolded approach assumes AI needs step-by-step guidance—a reasonable assumption for weaker models. But frontier models (achieving 93.9% on VeruSage) have sufficient reasoning capacity to synthesize specifications and proofs directly. The *initial completeness* may be lower, but the review process efficiently fills gaps. This represents a qualitative shift: **AI capability has crossed a threshold where complex verification becomes tractable**.

This finding motivates our central research question: *Given that frontier AI can solve benchmark verification problems, can it generalize to novel, real-world systems like OS kernels—and can it scale beyond memory management to complex concurrent subsystems such as process scheduling?*

### 1.4 Core Contributions

Our work makes the following contributions:

1. **Methodology**: We present an automated prover-reviewer iteration workflow—with multi-model review, trust decomposition, and anti-cheating guardrails—that systematically drives AI-generated verification toward completeness (Section 3).

2. **Empirical Validation at Scale**: We verify ~14,300 lines of original kernel source code across two major subsystems (memory management and process scheduling), producing 76,694 lines of Verus verification code with 1,857 mechanically proven properties in ~2 person-weeks of human effort (Section 4–5).

3. **Bug Discovery**: The verification uncovered 11 previously unknown bugs—including critical arithmetic overflows in allocators and a scheduler fairness flaw causing 1000:1 thread starvation—demonstrating that AI-augmented formal verification is an effective bug-finding tool for production kernel code (Section 5.5).

### 1.5 Target System: Nanvix Microkernel

Nanvix is a microkernel-based research operating system targeting serverless computing [4]. In serverless environments:
- **Cold-start latency** is critical—memory allocation is on the critical path.
- **Short-lived processes** demand efficient allocation/deallocation.
- **Multi-tenancy** requires strong isolation guarantees.
- **Fair scheduling** prevents starvation among co-located workloads.

Both the memory management and process scheduling subsystems are foundational to kernel correctness. Bugs in allocators can cause double allocation, use-after-free, and memory leaks; bugs in scheduling can cause deadlocks, starvation, and state corruption. These are precisely the **functional correctness** properties that formal verification can prove.

---

## 2. Background

### 2.1 Verus: Rust Formal Verification

Verus [5] is a verification tool for Rust that uses SMT (Satisfiability Modulo Theories) solvers to prove correctness properties. Key Verus concepts include:

- **Specification Functions (`spec fn`)**: Pure mathematical functions for expressing properties.
- **Proof Functions (`proof fn`)**: Lemmas that establish logical relationships.
- **Executable Functions (`fn`)**: Implementation code with `requires`/`ensures` contracts.
- **Ghost Code**: Specification-only code that doesn't affect runtime execution.
- **View Types**: Abstract representations of concrete data structures for specification.

Verus is particularly well-suited for Rust verification because it leverages Rust's ownership system to simplify reasoning about memory safety.

### 2.2 Nanvix Kernel Architecture

The verified subsystems span two major kernel components:

```
┌───────────────────────────────────────────────────────────────────────────┐
│                    Nanvix Kernel Architecture                             │
├───────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌────────────────────────────────┐   ┌────────────────────────────────┐  │
│  │ Memory Management (MM)         │   │ Process Management (PM)        │  │
│  │                                │   │                                │  │
│  │ Virtual Memory (VMem)          │   │ Process Manager                │  │
│  │ KPage, VirtInit                │   │ Process States (FSM)           │  │
│  ├────────────────────────────────┤   │ Thread Manager                 │  │
│  │ Physical Memory                │   │ Thread States (FSM)            │  │
│  │ Frame, UPool, KPool            │   │                                │  │
│  ├────────────────────────────────┤   │                                │  │
│  │ Kernel Memory Mgmt             │   │ Synchronization                │  │
│  │ KHeap, KStack, UStack          │   │ Mutex, CondVar, Semaphore,     │  │
│  │ KRedZone                       │   │ SpinLock, Fence                │  │
│  ├────────────────────────────────┤   │                                │  │
│  │ Library Components             │   │ Clock (Scheduler)              │  │
│  │ Bitmap, Slab, RawArray         │   │                                │  │
│  │ Error                          │   ├────────────────────────────────┤  │
│  └────────────────────────────────┘   │ System Types                   │  │
│                                       │ PID, TID, Capability           │  │
│  ┌────────────────────────────────┐   ├────────────────────────────────┤  │
│  │ HAL (Hardware Abstraction)     │   │ System Calls (KCalls)          │  │
│  │ FrameAddress types             │   │ create/join/terminate          │  │
│  ├────────────────────────────────┤   │ lock/unlock, signal/wait       │  │
│  │ Kernel Call Dispatch           │   │ sleep                          │  │
│  │ Dispatcher, Handler,           │   └────────────────────────────────┘  │
│  │ Scoreboard                     │                                       │
│  └────────────────────────────────┘                                       │
│                                                                           │
└───────────────────────────────────────────────────────────────────────────┘
```

### 2.3 Verification Properties

We target the following categories of safety properties:

**Memory Safety Properties (P1-P3)**:
- **P1 (No Double Allocation)**: An allocated frame cannot be allocated again.
- **P2 (No Use After Free)**: A freed frame is removed from the allocated set.
- **P3 (No Double Free)**: Only allocated frames can be freed.

**Liveness Properties (P4-P5)**:
- **P4 (Allocator Liveness)**: Allocation succeeds when free resources exist.
- **P5 (Failure Safety)**: State remains unchanged on failure.

**Memory Aliasing Property**:
- **P6 (No Memory Aliasing)**: All allocated frames have disjoint memory regions.

**Provenance Property** (for kernel pool):
- **P7 (Provenance Tracking)**: Frames can only be freed to their originating pool.

**Process/Thread State Machine Properties (P8-P12)**:
- **P8 (Valid State Transitions)**: State changes follow the defined FSM (e.g., Ready→Running→Zombie).
- **P9 (No Orphaned Threads)**: Thread lifecycle is properly tracked across process states.
- **P10 (Synchronization Safety)**: Mutex lock/unlock operations maintain consistency; no double-lock or unlock-without-lock.
- **P11 (Scheduler Fairness)**: Time quantum management ensures bounded scheduling.
- **P12 (Capability Integrity)**: Process capabilities can only be modified through authorized operations.

---

## 3. Methodology: Prover-Reviewer Iteration

### 3.1 Overview

Our methodology consists of three phases: initial synthesis by the Prover, iterative review by multiple Reviewers, and final human validation. The key insight is that iteration, not initial perfection, drives verification quality.

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                       Prover-Reviewer Iteration Workflow                        │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  Phase 1                      Phase 2                       Phase 3             │
│  ┌────────────────┐         ┌─────────────────────┐       ┌────────────┐        │
│  │ Prover Agent   │         │ Review Loop         │       │ Human      │        │
│  │ Direct Synth.  │────────▶│ (until all A+)      │──────▶│ Validation │        │
│  │ (Claude Opus)  │         │                     │       │            │        │
│  └────────────────┘         └─────────────────────┘       └────────────┘        │
│         │                          │                            │               │
│         ▼                          ▼                            ▼               │
│   Initial Verus code       Reviewers identify issues      Spec semantic         │
│   with specs + proofs      Prover fixes until A+          review only           │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

**Figure 1**: Prover-Reviewer Iteration Workflow

### 3.2 Reviewer Issue Categories

Based on our experience across 827 review artifacts spanning both memory management and process scheduling, we identify seven categories of issues that reviewers typically find:

| Category | Description | MM Example | PM Example |
|----------|-------------|------------|------------|
| **Coverage** | Missing function verification | "`alloc_range` is not verified" | "~15 functions from `ProcessManagerInner` are unverified" |
| **Postcondition** | Insufficient `ensures` clauses | "Missing liveness guarantee in `alloc`" | "Missing `number_buffered_messages` frame postcondition in `exit_thread`" |
| **Equivalence** | Semantic drift from original | "`alloc_many` returns non-contiguous in verified but contiguous in original" | "`terminate_ready_to_interrupted` puts process in interrupted queue, but original keeps it in ready" |
| **Invariant** | Missing state invariants | "No `no_memory_aliasing` in invariant" | "No global thread ID uniqueness lemma" |
| **Trust Boundary** | Unjustified external_body | "`from_raw_parts` should not be external_body" | "Outer `ProcessManager` wrapper (~30 public methods) is entirely unverified" |
| **Concurrency Gap** | Sequential model cannot express concurrent semantics | — | "`lock()` requires `spec_is_unlocked()`, so contended locking—the primary use case—is outside the model" |
| **Abstraction Fidelity** | Verified model loses structural information | — | "Using `Set<int>` for ready queue loses FIFO ordering; scheduling fairness cannot be proven" |

The first five categories appeared consistently in both MM and PM verification. The last two—**Concurrency Gap** and **Abstraction Fidelity**—emerged specifically during PM verification, where the prover must model inherently concurrent primitives (mutexes, condition variables) and stateful data structures (scheduling queues) within Verus's sequential reasoning framework.

### 3.3 Grading System

Each reviewer assigns a grade after review:

| Grade | Meaning | Action |
|-------|---------|--------|
| **A+** | No issues, verification complete | Proceed to next module |
| **A** | Minor issues, acceptable | Fix recommended but not required |
| **B** | Moderate issues | Must address before proceeding |
| **C/D/F** | Significant gaps | Major revision required |

Iteration continues until **all reviewers assign A+**. This prevents premature convergence.

### 3.4 Trust Boundary Decomposition

A critical insight is that not all verification components require the same level of trust. We decompose the verification into three trust levels:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                       Trust Boundary Decomposition                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Trust Level 0: ZERO TRUST (Mechanically Verified)                      │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │  • Proof validity: Checked by Verus SMT solver                     │ │
│  │  • Logical consistency: Guaranteed by type system                  │ │
│  │  • No AI trust required for proof correctness                      │ │
│  │  • 1,857 properties mechanically verified                          │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  Trust Level 1: SEMANTIC TRUST (Human-Reviewed Specs)                   │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │  • Specification adequacy: Do specs match intent?                  │ │
│  │  • ~1,098 spec functions across all modules to review              │ │
│  │  • Domain expertise required, but tractable                        │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  Trust Level 2: IMPLEMENTATION TRUST (Explicit Boundary)                │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │  • external_body functions: Trusted without proof                  │ │
│  │  • 134 functions across all modules                                │ │
│  │  • Concentrated in raw memory ops and unsafe process operations    │ │
│  │  • Explicitly documented and auditable                             │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

**Figure 2**: Trust Boundary Decomposition

This decomposition enables a key efficiency: humans review only Level 1 (specifications), while Level 0 is mechanically guaranteed and Level 2 is explicitly bounded.

### 3.5 Why Multiple Reviewers?

We use multiple AI reviewers from different model families (Claude, GPT, Gemini) because:

1. **Different models catch different issues**: Opus produces detailed reviews with many issues, while GPT and Gemini tend to focus on fewer but sometimes critical points.
2. **Redundancy provides confidence**: When multiple reviewers agree on no issues, we have higher confidence in completeness.
3. **Reduces single-model blind spots**: Each model family may have systematic weaknesses due to training data differences.

### 3.6 Workflow Phases

#### Phase 1: Setup

Before invoking AI agents, we prepare:

1. **Source Code**: The original Rust implementation to be verified.
2. **High-Level Goal**: A brief description of what to verify.
3. **Codebase Context**: Related modules and dependencies.

Note: We do *not* provide detailed property specifications or Verus tutorials. Frontier models already understand Verus from training data and can infer appropriate specifications from code structure.

#### Phase 2: Initial Proof Generation (AI Prover)

The AI prover receives the source code and generates a three-file split for each module:

1. **`{name}.spec.rs`**: View types, invariants, spec functions (abstract specification).
2. **`{name}.rs`**: Executable code with `requires`/`ensures` contracts.
3. **`{name}.proof.rs`**: Proof lemmas and proof methods.

This three-file organization separates concerns: specifications can be reviewed independently of proofs, and proofs can be regenerated without touching specifications.

The prover operates as a code agent with access to tools (file viewing, editing, Verus execution). It iteratively refines its output until Verus verification succeeds.

#### Phase 3: Iterative Review (AI Reviewers)

Once the prover generates an initial verified implementation, multiple AI reviewers independently analyze the code. Each reviewer is prompted to identify issues in several categories:

**Review Categories**:
1. **Coverage**: Are all functions from the original source verified?
2. **Specification Adequacy**: Do the specs capture the intended behavior?
3. **Proof Completeness**: Are there unjustified assumptions or gaps?
4. **Semantic Equivalence**: Does the verified code behave identically to the original?
5. **Trusted Code Boundary**: Are external_body annotations minimized and justified?

**Iterative Convergence**:
The prover receives all reviewer issues and must either:
1. **Fix** the issue and explain the change, or
2. **Justify** why the issue is not applicable.

This process repeats until all reviewers report no remaining issues **and assign grade A+**. In our experience, this typically requires 2-4 iterations per module.

#### Phase 4: Post-Review Polish

After convergence, we apply three automated refinement passes:

1. **Consistency Check**: Verify semantic equivalence between original and verified code, flagging divergences.
2. **Simplification**: Remove redundant lemmas and tautological proofs to reduce verification time.
3. **Strengthening**: Add error-case specifications and enhance postconditions for failure paths.

These polish passes were applied to the initial memory management modules and generated detailed reports documenting each change.

#### Phase 5: Human Validation

Human involvement is minimized but essential:

1. **Specification Review**: Verify that specs match intended behavior.
2. **Verus Verification**: Confirm `verus --crate-type lib lib.rs` passes.
3. **Bug Triage**: Evaluate discovered bugs and determine if they are genuine.
4. **Intervention**: Occasionally unstick the AI when it reaches an impasse.

### 3.7 Guardrails and Anti-Cheating Mechanisms

AI agents may attempt to "cheat" by taking shortcuts that undermine verification quality. We implement several guardrails:

**Guardrail 1: Assume Statement Detection**
```bash
grep -c "assume" verus/*.rs  # Must be 0
```
The `assume` keyword in Verus introduces unverified axioms. Our CI script flags any code containing assumes, forcing the prover to construct genuine proofs.

**Guardrail 2: External Body Auditing**
```bash
grep -n "external_body" verus/*.rs > external_bodies.txt
```
Each `external_body` must be explicitly justified. Reviewers challenge any new external_body annotations.

**Guardrail 3: Verification Timeout Control**
```bash
timeout 60s verus --crate-type lib lib.rs
```
We enforce a 60-second timeout per module to prevent resource explosion and force efficient proof strategies.

**Guardrail 4: Coverage Verification**
```bash
diff <(grep "pub fn" src/original.rs | sort) \
     <(grep "pub fn" verus/verified.rs | sort)
```
Ensures the prover doesn't silently skip difficult functions.

**Guardrail 5: Automated Git Tracking**
Every prover iteration is automatically committed to git, enabling rollback and audit trails. The verification script (`verify.sh`) logs Verus results and detects patterns like `assume`, `external_body`, `admit`, and `trusted`.

### 3.8 Multi-Model Strategy

We employ different AI models for different roles:

| Role | Model | Rationale |
|------|-------|-----------|
| Prover | Claude Opus 4.6 (fast) | Strongest reasoning, 30min timeout |
| Reviewer 1 | Claude Opus 4.6 | Independent high-quality review |
| Reviewer 2 | GPT-5.2-Codex | Different perspective, catches blind spots |
| Reviewer 3 | Gemini 3 Pro (Preview) | Additional diversity |

Using multiple model families reduces the risk of systematic blind spots. The automated workflow (`workflow.py`) orchestrates up to 3 outer rounds with 3 inner iterations per reviewer.

### 3.9 Methodology Evolution

Our methodology evolved significantly during the project:

| Aspect | Phase 1 (MM Libraries) | Phase 2 (MM Kernel) | Phase 3 (PM) |
|--------|----------------------|---------------------|--------------|
| **Prover Model** | Claude Opus 4.5 | Claude Opus 4.5 | Claude Opus 4.6 |
| **Code Organization** | Single file per module | Single file | Three-file split |
| **Review Process** | Manual | Semi-automated | Fully automated pipeline |
| **Spec Style** | Bit-level mirroring | Mixed | Abstract (sets, sequences) |
| **Consistency Check** | Manual | Automated reports | Integrated |
| **Modules Per Campaign** | Individual | Individual | Dependency-ordered batches |

The shift from Phase 1 to Phase 3 represents a significant maturation: specifications became more abstract, the workflow became fully automated, and verification campaigns were organized by dependency order to ensure compositional correctness.

---

## 4. Implementation

### 4.1 Verified Modules

We verified 43 modules spanning the complete memory management and process scheduling subsystems, organized into five major components:

#### Libraries (4 modules)

| Module | Original LoC | Verified LoC | spec fn | proof fn | exec fn | external_body | assume |
|--------|-------------|-------------|---------|----------|---------|---------------|--------|
| `error` | ~80 | 80 | 0 | 0 | 4 | 2 | 0 |
| `raw_array` | 351 | 628 | 10 | 21 | 4 | 8 | 0 |
| `bitmap` | 1,182 | 2,418 | 17 | 41 | 11 | 0 | 0 |
| `slab` | 227 | 3,005 | 24 | 52 | 5 | 0 | 0 |
| **Subtotal** | **1,840** | **6,131** | **51** | **114** | **24** | **10** | **0** |

#### Memory Management — Physical Memory (3 modules)

| Module | Original LoC | Verified LoC | spec fn | proof fn | exec fn | external_body | assume |
|--------|-------------|-------------|---------|----------|---------|---------------|--------|
| `frame` | 202 | — | — | — | — | — | — |
| `upool` | 239 | — | — | — | — | — | — |
| `kpool` | 249 | — | — | — | — | — | — |
| `manager` | 99 | — | — | — | — | — | — |
| **Subtotal (phys)** | **789** | **4,683** | **75** | **40** | **43** | **2** | **0** |

#### Memory Management — Virtual Memory & Kernel Allocators (7 modules)

| Module | Original LoC | Verified LoC | spec fn | proof fn | exec fn | external_body | assume |
|--------|-------------|-------------|---------|----------|---------|---------------|--------|
| `vmem` | 1,112 | — | — | — | — | — | — |
| `kpage` | 79 | — | — | — | — | — | — |
| `virt_init` | 225 | — | — | — | — | — | — |
| `kheap` | 237 | 2,013 | 17 | 29 | 2 | 0 | 0 |
| `kredzone` | 96 | 1,077 | 10 | 28 | 6 | 3 | 1 |
| `kstack` | 141 | 717 | 24 | 5 | 9 | 0 | 0 |
| `ustack` | 93 | 932 | 24 | 7 | 14 | 0 | 0 |
| **Subtotal (virt+alloc)** | **1,983** | **9,160** | **128** | **95** | **74** | **14** | **1** |

#### HAL & Kernel Call Dispatch (4 modules)

| Module | Original LoC | Verified LoC | spec fn | proof fn | exec fn | external_body | assume |
|--------|-------------|-------------|---------|----------|---------|---------------|--------|
| `frame_address` | 82 | 369 | 11 | 1 | 11 | 0 | 0 |
| `dispatcher` | — | — | — | — | — | — | — |
| `handler` | — | — | — | — | — | — | — |
| `scoreboard` | — | — | — | — | — | — | — |
| **Subtotal (kcall)** | **810** | **8,076** | **127** | **128** | **75** | **26** | **0** |

#### Process Management (25 modules)

| Component | Modules | Original LoC | Verified LoC | spec fn | proof fn | exec fn | external_body | assume |
|-----------|---------|-------------|-------------|---------|----------|---------|---------------|--------|
| **System Types** | pid, tid, capability | ~500 | 2,535 | 27 | 50 | 45 | 14 | 0 |
| **Synchronization** | mutex, condvar, semaphore, spinlock, fence | 879 | 5,088 | 64 | 134 | 36 | 0 | 0 |
| **Thread States** | ready, running, sleeping, interrupted, zombie, state, thread_manager | 1,390 | 6,788 | 147 | 160 | 64 | 3 | 0 |
| **Process States** | runnable, running, sleeping, interrupted, zombie, process_state, capability, process_manager, process_manager_unsafe | 4,620 | 15,259 | 226 | 172 | 195 | 30 | 0 |
| **Clock/Scheduler** | clock | 168 | 1,826 | 25 | 37 | 16 | 2 | 0 |
| **System Calls** | create_thread, join_thread, terminate, lock_mutex, unlock_mutex, signal_cond, wait_cond, sleep | 1,362 | 12,297 | 200 | 155 | 48 | 33 | 0 |
| **Subtotal (PM)** | **25** | **8,919** | **43,793** | **689** | **708** | **404** | **82** | **0** |

#### Summary

| Component | Modules | Original LoC | Verus LoC | Expansion |
|-----------|---------|-------------|-----------|-----------|
| Libraries | 4 | 1,840 | 6,131 | 3.3× |
| MM Physical | 4 | 789 | 4,683 | 5.9× |
| MM Virtual + Alloc | 7 | 1,983 | 9,160 | 4.6× |
| HAL + KCall | 4 | 810 | 8,076 | 10.0× |
| Process Mgmt | 25 | 8,919 | 43,793 | 4.9× |
| **Grand Total** | **44** | **~14,341** | **76,694** | **5.3×** |

We verified **~14,341 lines of original Rust source code**, producing **76,694 lines of Verus verification code** (specifications, proofs, and annotated implementations) with **1,857 mechanically proven properties**. The 5.3× expansion factor is primarily due to specification functions, proof lemmas, and `requires`/`ensures` contracts.

### 4.2 Verification Results

```
$ cd verus/split && verus --crate-type lib lib.rs --time
verification results:: 1857 verified, 0 errors

total-time:           10987 ms    (estimated total cpu time 25872 ms)
    rust-time:                4526 ms
        init-and-types:           2456 ms
    verification-time:        5763 ms
        vir-time:                 2325 ms
        verify-crate-time:        3438 ms
            total smt-time:       7784 ms   (57 threads)
            total smt-run:        6565 ms, 56776544 rlimit (57 threads)
```

All 1,857 properties verify in under 11 seconds with 57-thread parallel SMT solving.

### 4.3 Verified Property Taxonomy

The 1,857 verified properties fall into five categories, spanning qualitatively different correctness concerns across MM and PM:

| Property Category | MM Example | PM Example | Count (est.) |
|-------------------|------------|------------|-------------|
| **State Integrity** | Allocator invariant preserved across alloc/free | Process FSM well-formedness preserved across all transitions | ~600 |
| **Safety** | No double allocation; frames disjoint in memory | No double-lock; zombie thread state not silently lost | ~400 |
| **Liveness** | Allocation succeeds when free frames exist | Thread creation succeeds when IDs available | ~200 |
| **Frame Conditions** | Only the target bit changes in bitmap | Message count unchanged by thread exit | ~450 |
| **Structural** | Usage count tracks set bits exactly | PID monotonicity ensures freshness; queue disjointness | ~200 |

Two examples illustrate the depth of verified properties:

**Memory No-Aliasing (MM)** — a global invariant over all allocated frames:
```rust
pub open spec fn frames_disjoint(&self) -> bool {
    forall|i: int, j: int|
        #![trigger self.is_allocated(i), self.is_allocated(j)]
        (self.is_allocated(i) && self.is_allocated(j) && i != j)
        ==> self.frame_addr(i) + FRAME_SIZE <= self.frame_addr(j)
            || self.frame_addr(j) + FRAME_SIZE <= self.frame_addr(i)
}
```

**Process Queue Disjointness (PM)** — no process exists in two scheduling queues simultaneously:
```rust
pub open spec fn queues_disjoint(&self) -> bool {
       self.ghost_ready@.disjoint(self.ghost_suspended@)
    && self.ghost_ready@.disjoint(self.ghost_zombie@)
    && self.ghost_suspended@.disjoint(self.ghost_zombie@)
    && (self.ghost_running.is_some() ==>
           !self.ghost_ready@.contains(self.ghost_running.unwrap())
        && !self.ghost_suspended@.contains(self.ghost_running.unwrap()))
}
```

These two invariants exemplify the shift from arithmetic reasoning (MM) to set-theoretic state machine reasoning (PM) as verification scaled from data structures to kernel subsystems.

### 4.4 Trusted Code Boundary

The 134 `external_body` annotations are distributed across modules:

| Component | external_body Count | Primary Justification |
|-----------|--------------------|-----------------------|
| Libraries | 10 | Raw memory operations (alloc, dealloc, pointer access) |
| MM Kernel | 14 | Hardware page table operations, MMU interactions |
| HAL + KCall | 26 | System call dispatch, hardware interaction |
| PM (Process) | 30 | Unsafe process table access, raw pointer manipulation |
| PM (Syscalls) | 33 | Process/thread creation involving unsafe operations |
| PM (Sys Types) | 14 | Type conversion, capability bit manipulation |
| PM (Thread) | 3 | Thread context switch |
| PM (Clock) | 2 | Timer interrupt handling |
| PM (Sync) | 0 | **Fully verified with zero external_body** |

Notable: The synchronization module (mutex, condvar, semaphore, spinlock, fence) is **fully verified** with zero external_body annotations—all 5,088 lines of verified code are mechanically checked.

### 4.5 Assume Statement Analysis

The verified codebase contains exactly **1 assume statement**, located in `kredzone.rs`:
```rust
assume(res.unwrap() == spec_load_result(ghost.view, index as int));
```
This assume bridges a gap between the concrete `RawArray::get()` return value and its abstract specification, arising from the `external_body` boundary of `RawArray`. Eliminating this would require inlining the raw array specification into the redzone module.

### 4.6 Abstraction Decisions

Several intentional abstractions differ from the original implementation:

| Aspect | Original | Verified | Rationale |
|--------|----------|----------|-----------|
| Ownership | `Rc<RefCell<>>` | Single-owner | Simpler specification |
| Deallocation | RAII via Drop | Explicit `free()` | Clearer proof obligations |
| Memory clearing | `clear` parameter | Omitted | Orthogonal to safety |
| Byte access | `Deref`/`DerefMut` | Not modeled | Beyond allocation safety |
| Capabilities | Bit-field operations | Set-based abstraction | Cleaner specification |
| Process state | Complex enum | Explicit state types | Enables FSM verification |

---

## 5. Evaluation

### 5.1 Bugs Discovered Through Formal Verification

A primary outcome of our verification effort is the discovery of **11 previously unknown bugs** spanning both memory management and process scheduling:

| # | Bug | Module | Severity | Type | Description |
|---|-----|--------|----------|------|-------------|
| 1 | **Zombie Thread Loss** | `kernel/pm/process` | **Critical** | State logic | Double `Option::take()` silently discards zombie thread state |
| 2 | Quantum Inheritance | `kernel/pm/process` | Medium | Design flaw | Same-process context switch inherits residual quantum, causing 1000:1 starvation |
| 3 | Mutex Capacity Check | `kernel/pm/process` | Medium | Logic ordering | Capacity check precedes presence check, rejecting existing entries when map full |
| 4 | Slab Underflow | `libs/slab` | Medium-High | Arithmetic | Unchecked subtraction can underflow, corrupting slab metadata |
| 5 | Bitmap Index Overflow | `libs/bitmap` | Medium | Arithmetic | Bounds check recomputation overflows, bypassing the check |
| 6 | Bitmap Multiplication Overflow | `libs/bitmap` | Medium | Arithmetic | `array.len() * 8` overflows on large arrays |
| 7 | Slab Deallocate Alignment | `libs/slab` | Medium-High | Missing check | Unaligned pointers silently free wrong blocks |
| 8 | Slab Deallocate Bounds Overflow | `libs/slab` | Low-Medium | Arithmetic | Multiplication overflow disables upper bounds check |
| 9 | Slab Address Overflow | `libs/slab` | Medium | Arithmetic | `addr + offset` overflows in `from_raw_parts()` |
| 10 | Slab Multiplication Overflow | `libs/slab` | Medium | Arithmetic | `num_index_blocks * block_size` overflows |
| 11 | Address Overflow | `libs/slab` | Medium | Arithmetic | Base address computation overflow |

These bugs divide into two qualitatively different classes. Bugs 4–11 are **arithmetic overflow/underflow** issues in memory management libraries—precisely the kind of bugs that Verus catches structurally by requiring overflow-free arithmetic proofs. Bugs 1–3 are **logic and design errors** in the process scheduler—deeper bugs that require state machine modeling to surface.

#### Case Study: Zombie Thread Loss (Bug #1)

This bug best illustrates why formal verification catches errors that testing, code review, and Rust's type system all miss.

**Context.** When a running thread exits, `RunningProcess::exit_thread()` must (1) convert the running thread into a zombie record, (2) collect it with any previously accumulated zombies, and (3) pass the collection to the next process state. The function has four branches depending on which threads remain (ready, interrupted, sleeping, or zombie-only).

**The Bug.** In the interrupted branch, the function calls `self.zombie.take()` on a field already consumed earlier in the same function:

```rust
pub fn exit_thread(mut self, status: ExitStatus) -> ... {
    let (zombie_thread, ctx) = self.running.exit(status);

    // Line 261: self.zombie consumed here — first take()
    let zombie_threads = match self.zombie.take() {
        Some(mut zs) => { zs.push_back(zombie_thread); zs },
        None => NonEmptyVecDeque::new(zombie_thread),
    };

    if let Some(ready_threads) = self.ready.take() {
        // Ready branch: uses local zombie_threads ✓
        Ok((..., Some(zombie_threads), ...))
    } else if let Some(interrupted_threads) = self.interrupted_threads.take() {
        // Interrupted branch: calls self.zombie.take() AGAIN ✗
        InterruptedProcess::from_sleeping(
            ...,
            self.zombie.take(),  // ← BUG: always None (already consumed)
        );
    } else { ... }  // sleeping/zombie branches: use zombie_threads ✓
}
```

Three of four branches correctly use the local `zombie_threads`. The interrupted branch accidentally calls `self.zombie.take()` a second time—which always returns `None` because the `Option` was already consumed. All zombie thread records are **silently discarded**.

**Why existing tools miss it.** Rust's `Option::take()` is *designed* to be called multiple times safely—it returns `None` on subsequent calls without panicking. The code is memory-safe, type-correct, and compiles without warnings. The bug is a pure logic error invisible to the borrow checker. Testing is also unlikely to catch it: the interrupted branch requires a specific thread configuration (no ready threads, at least one interrupted thread), and the effect (zombie state loss) manifests as a gradual resource leak rather than an immediate crash.

**How verification found it.** The AI prover, when constructing the Verus model for `exit_thread()`, was forced to explicitly track ownership of the zombie collection through each branch—verification requires proving where every value goes. The prover naturally modeled the correct behavior: the local `zombie_threads` must be passed to *every* branch. When an AI reviewer compared this model against the original source code, it identified the divergence: the source passes `self.zombie.take()` (always `None`) instead of `zombie_threads` in the interrupted branch. The bug was a refactoring regression introduced 7 months earlier when the author restructured the process state machine; a nearly identical bug in a sibling function (`exit()`) had been noticed and fixed the same day, but this instance was overlooked.

**Significance.** This bug demonstrates the core value proposition of our approach: formal verification requires *exhaustive* ownership tracking across all control flow paths. Even a single forgotten variable in one branch is caught—not by AI intelligence, but by the mathematical obligation that every value must be accounted for. The AI's role was to construct the model and identify the source-model divergence; the proof obligation itself is what made the bug visible.

### 5.2 Verification Effort

The project was conducted in three major phases:

| Phase | Scope | Modules | Original LoC | Verus LoC | Properties |
|-------|-------|---------|-------------|-----------|------------|
| Phase 1: MM Libraries | bitmap, slab, raw_array, error | 4 | 1,840 | 6,131 | ~200 |
| Phase 2: MM Kernel | frame, kpool, upool, kheap, kstack, ustack, kredzone, vmem, kpage, virt | 11 | 2,772 | 13,843 | ~400 |
| Phase 3: PM + KCall | thread states, process states, sync, clock, syscalls, dispatch | 29 | 9,729 | 56,720 | ~1,257 |
| **Total** | | **44** | **~14,341** | **76,694** | **1,857** |

### 5.3 Review Session Statistics

The automated review pipeline generated 827 review artifacts across 43 modules. Each module undergoes multiple review rounds with up to 3 reviewer models per round:

| Metric | Value |
|--------|-------|
| Total review artifacts | 827 |
| Modules reviewed | 43 |
| Reviewer models used | Claude Opus 4.6, GPT-5.2-Codex, Gemini 3 Pro |
| Avg. review rounds per module | ~2.5 |
| Consistency/simplification/strengthening reports | 27 |

### 5.4 Verification Property Statistics

| Metric | Count |
|--------|-------|
| Verified properties (Verus) | 1,857 |
| Specification functions (`spec fn`) | 1,098 |
| Proof functions (`proof fn`) | 1,120 |
| Executable functions verified | 636 |
| `requires` clauses | 1,480 |
| `ensures` clauses | 1,892 |
| Invariant annotations | 339 |
| `external_body` annotations | 134 |
| `assume` statements | 1 |

### 5.5 Verus Execution Performance

Verification time breakdown (57-thread parallel):

| Phase | Time (ms) |
|-------|-----------|
| Rust compilation & types | 4,526 |
| VIR generation | 2,325 |
| SMT solving | 7,784 |
| **Total wall-clock** | **10,987** |

Sub-11-second full verification for ~14,300 lines of source code (76,694 lines of Verus code) enables rapid iteration during AI proof refinement.

### 5.6 Comparison with Traditional Verification

| Project | Target | Original LoC | Verus/Proof LoC | Properties | Human Effort | Bugs Found |
|---------|--------|-------------|-----------------|------------|--------------|------------|
| seL4 [1] | Microkernel (C) | 10,000 | ~200,000 (Isabelle) | ~10,000 | 11 person-years | — |
| CertiKOS [6] | Concurrent kernel | 6,500 | ~100,000 (Coq) | — | 5 person-years | — |
| **Nanvix (Ours)** | **MM + PM (Rust)** | **~14,300** | **76,694 (Verus)** | **1,857** | **~2 person-weeks** | **11** |

Note: Direct comparison is imperfect—seL4 and CertiKOS verify different properties at different abstraction levels, and our verified LoC includes specification and proof code. However, the productivity improvement is substantial.

---

## 6. Discussion

### 6.1 What the Prover Does Well

Based on our experience, frontier LLMs demonstrate strong capabilities in:

1. **Direct Proof Synthesis**: Given source code and target properties, the prover generates complete Verus implementations that often verify on first attempt for simpler modules.

2. **Invariant Inference**: The prover correctly infers invariants from code structure, naming conventions, and error handling patterns.

3. **Proof Debugging**: When Verus fails, the prover effectively diagnoses and fixes issues—this reasoning ability is particularly valuable.

4. **Layered Composition**: The prover understands module dependencies and correctly builds higher-level proofs on lower-level ones.

5. **State Machine Modeling**: For the PM subsystem, the prover successfully modeled complex state machines (thread lifecycle, process states) with correct transition specifications.

### 6.2 What the Reviewers Catch

From our review sessions, reviewers consistently identified:

1. **Semantic Drift**: The prover sometimes changes semantics to simplify proofs. For example, `alloc_many` returning non-contiguous frames when the original returns contiguous.

2. **Missing Functions**: Less obvious helper functions are sometimes skipped. Reviewers explicitly check coverage.

3. **Weak Postconditions**: Initial proofs may verify but miss important guarantees (e.g., liveness properties).

4. **Trust Boundary Creep**: The prover may add `external_body` to avoid difficult proofs. Reviewers challenge each annotation.

5. **Ghost-Only Modules**: In some cases (e.g., `interrupted.rs`), the prover initially made the entire module ghost code—reviewers caught this and required executable implementations.

### 6.3 Scaling from Memory Management to Process Scheduling

A key finding is that our methodology scales from library-level data structure verification to complex kernel subsystem verification:

| Dimension | Memory Management | Process Scheduling |
|-----------|------------------|--------------------|
| Code complexity | Data structures, arithmetic | State machines, concurrency |
| Invariant type | Bounds, disjointness | State transition validity |
| Proof difficulty | Moderate (arithmetic) | High (state composition) |
| External_body density | Low (10/6,131 = 0.2%) | Higher (82/43,793 = 0.2%) |
| Bugs found | 7 (arithmetic) | 4 (logic, design) |
| Review iterations needed | 2-4 per module | 2-3 per module |

**Observations**:
1. **PM verification required more external_body annotations** due to unsafe operations in process/thread management, but the *density* remained comparable.
2. **Bug types differ qualitatively**: MM bugs are predominantly arithmetic (overflow/underflow), while PM bugs involve logic errors and design flaws.
3. **PM specifications are more abstract**: Instead of bit-level reasoning, PM specs use set-based abstractions for capabilities and state predicates.
4. **Model quality matters**: The upgrade from Claude Opus 4.5 (Phase 1) to Claude Opus 4.6 (Phase 3) was noted as producing significantly better initial verification quality.

### 6.4 Practical Insights

1. **Dependency-Ordered Verification**: Verifying modules in dependency order (types → primitives → managers → syscalls) is essential for compositional proofs. The PM verification was organized into 6 dependency phases.

2. **Specification Abstraction Level**: Early specifications that mirrored implementation details (bit-level capability checks) were fragile. Shifting to abstract specifications (set membership) produced more maintainable and robust proofs.

3. **Three-File Split**: Separating spec/proof/exec into distinct files improved both AI comprehension and human review. Specifications can be reviewed without understanding proof details.

4. **Automated Pipeline**: The transition from manual review invocation to automated workflow (`workflow.py`) reduced turnaround time and improved consistency.

### 6.5 Trust Decomposition Effectiveness

The trust decomposition achieves its goal of minimizing human involvement:

| Trust Level | Components | Human Effort |
|-------------|------------|--------------|
| Level 0 (Zero Trust) | 1,857 proofs | 0 hours (mechanical) |
| Level 1 (Semantic) | 1,098 spec functions | ~2 weeks intermittent review |
| Level 2 (Implementation) | 134 functions | ~2 hours audit |

### 6.6 Challenges Encountered

**Challenge 1: Prover Omissions**
The AI prover occasionally skipped functions, claiming they were "obvious" or "outside scope." The automated coverage check guardrail addresses this.

**Challenge 2: Specification Drift**
The prover sometimes modified source code semantics to simplify proofs. The consistency check phase generates detailed reports comparing original and verified code.

**Challenge 3: Proof Debugging**
When Verus fails, error messages can be cryptic. The AI prover's reasoning capabilities prove valuable for diagnosing and fixing proof failures.

**Challenge 4: Context Limits**
Large modules approach context window limits. We address this by verifying modules incrementally and providing summaries of dependencies.

**Challenge 5: Ghost-Only Shortcuts**
In some modules (e.g., `interrupted.rs`), the prover initially made functions entirely ghost—producing a specification rather than a verified implementation. Reviewers must check for this failure mode.

**Challenge 6: External Body Growth in PM**
The PM subsystem required more external_body annotations than MM due to inherently unsafe operations (process table access, thread context switching). Each annotation must be individually justified.

### 6.7 Limitations

1. **Shadow Model Verification**: We verify a *model* of the code, not the production code itself. Spec/implementation consistency is checked but not formally proven.

2. **External Body Trust**: The 134 external_body functions are assumed correct without verification.

3. **Single Assume**: The 1 remaining assume in `kredzone.rs` represents a gap that could theoretically be exploited.

4. **Fairness Properties**: While we discovered the quantum inheritance bug (Bug #10), comprehensive scheduler fairness properties (e.g., eventual scheduling guarantee) are not yet fully specified.

5. **Reproducibility**: AI model updates may affect proof generation quality.

6. **Reviewer Hallucinations**: Reviewers occasionally demand proofs for unreasonable properties. The prover must recognize and reject such demands.

7. **In-Tree Integration**: Integration of verified code back into the Nanvix source tree is blocked by Verus's requirement for specific Rust toolchain versions.

### 6.8 Semantic Equivalence Validation

To address the shadow model limitation, we conducted automated semantic equivalence analysis. The `exec_diff_report.md` documented:

| Severity | Issue Count |
|----------|-------------|
| Critical | 66 |
| High | 119 |
| Medium | 115 |
| Low | 1 |
| **Total** | **301** |

Most "critical" issues are structural differences (added ghost fields, Verus-required type changes) rather than semantic divergences. AI-assisted review classifies each difference as:
- **Ghost insertions** (acceptable—no runtime effect)
- **Verus compatibility changes** (document and accept)
- **True semantic changes** (require investigation and potential fixes)

---

## 7. Related Work

### 7.1 Traditional OS Verification

seL4 [1] achieved the first complete formal verification of an OS kernel, proving functional correctness and security properties using Isabelle/HOL. CertiKOS [6] extended this to concurrent kernels using Coq. Our work differs fundamentally in *who* does the proof work—AI agents versus human experts—and achieves comparable scope at orders-of-magnitude lower human effort.

### 7.2 Automated Verification Tools

Verus [5] and similar tools (Dafny, F*) provide verification infrastructure but still require human-authored specifications and proofs. Our methodology treats the verification tool as an oracle while automating the cognitive work of specification and proof construction.

### 7.3 AI for Theorem Proving

GPT-f [7], Baldur [8], and AlphaProof [9] apply LLMs to mathematical theorem proving. These works focus on *pure mathematics*; we address *systems verification* with its unique challenges (low-level memory, state machines, concurrency).

### 7.4 Multi-Agent AI Systems

Constitutional AI [10] and AI debate [11] use multi-agent interactions for alignment. We adapt this paradigm for verification, using multiple reviewers to improve verification quality.

**Positioning**: Our work uniquely combines (1) prover-reviewer iteration, (2) multi-model review diversity, (3) trust boundary decomposition, (4) practical OS kernel verification across both memory management and scheduling, and (5) a fully automated verification pipeline.

---

## 8. Conclusion

We have presented a practical AI-augmented methodology for formal verification that combines frontier LLMs as provers with multi-model review for iterative refinement.

Applied to Nanvix's memory management and process scheduling subsystems, our approach achieved:
- **~14,300 lines** of original Rust source code verified across 43+ modules
- **76,694 lines** of Verus verification code (5.3× expansion)
- **1,857 verified properties** mechanically checked by SMT solver
- **11 bugs discovered** including critical state logic errors and a scheduler fairness flaw
- **827 review artifacts** across multiple model families
- **~2 person-weeks** of human effort

Key lessons learned:
1. **Direct synthesis works**: Frontier AI can generate complete initial proofs without extensive scaffolding.
2. **Review iteration is essential**: Multiple reviewers from different model families catch complementary issues.
3. **Trust decomposition is practical**: Humans review only specifications, not proofs.
4. **Guardrails prevent cheating**: Automated checks for assumes, external_body, and timeouts are necessary.
5. **The methodology scales**: From library data structures to complex kernel state machines and scheduling.
6. **Verification finds real bugs**: 11 previously unknown bugs were discovered, including design-level flaws invisible to testing.
7. **Automation matters**: The transition from manual to automated pipeline significantly improved throughput and consistency.

Our experience suggests that AI-augmented verification can make formal methods accessible for real-world systems development. The approach is not fully automated—human specification review and occasional intervention remain required—but the effort reduction compared to traditional methods is transformative.

---

## References

[1] G. Klein et al., "seL4: Formal verification of an OS kernel," in *SOSP*, 2009.

[2] OpenAI, "GPT-4 Technical Report," 2023.

[3] Anthropic, "Claude 3.5 System Card," 2024.

[4] P. Penna et al., "Nanvix: A microkernel for serverless computing," 2024.

[5] A. Lattuada et al., "Verus: Verifying Rust programs using linear ghost types," in *OOPSLA*, 2023.

[6] R. Gu et al., "CertiKOS: An extensible architecture for building certified concurrent OS kernels," in *OSDI*, 2016.

[7] S. Polu and I. Sutskever, "Generative language modeling for automated theorem proving," in *NeurIPS*, 2020.

[8] E. First et al., "Baldur: Whole-proof generation and repair with LLMs," in *ICML*, 2023.

[9] DeepMind, "AlphaProof: AI achieves silver-medal performance in IMO," 2024.

[10] Y. Bai et al., "Constitutional AI: Harmlessness from AI feedback," 2022.

[11] G. Irving et al., "AI safety via debate," 2018.

---

## Appendix A: Complete Verification Statistics

### A.1 Per-Module Detailed Counts

| Module | Verified LoC | spec fn | proof fn | exec fn | requires | ensures | external_body |
|--------|-------------|---------|----------|---------|----------|---------|---------------|
| **Libraries** | | | | | | | |
| error | 80 | 0 | 0 | 4 | 0 | 1 | 2 |
| raw_array | 628 | 10 | 21 | 4 | 17 | 20 | 8 |
| bitmap | 2,418 | 17 | 41 | 11 | 64 | 51 | 0 |
| slab | 3,005 | 24 | 52 | 5 | 70 | 61 | 0 |
| **MM Physical** | | | | | | | |
| phys (frame+kpool+upool+manager) | 4,683 | 75 | 40 | 43 | 87 | 59 | 2 |
| **MM Virtual+Allocators** | | | | | | | |
| virt (vmem+kpage+init) | 4,421 | 53 | 26 | 43 | 64 | 75 | 11 |
| kheap | 2,013 | 17 | 29 | 2 | 30 | 35 | 0 |
| kredzone | 1,077 | 10 | 28 | 6 | 17 | 26 | 3 |
| kstack | 717 | 24 | 5 | 9 | 15 | 14 | 0 |
| ustack | 932 | 24 | 7 | 14 | 18 | 21 | 0 |
| **HAL** | | | | | | | |
| frame_address | 369 | 11 | 1 | 11 | 4 | 13 | 0 |
| **Kernel Call Dispatch** | | | | | | | |
| kcall (dispatcher+handler+scoreboard) | 7,707 | 116 | 127 | 64 | 94 | 214 | 26 |
| **PM System Types** | | | | | | | |
| sys (pid+tid+capability) | 2,535 | 27 | 50 | 45 | 13 | 101 | 14 |
| **PM Synchronization** | | | | | | | |
| sync (mutex+condvar+sema+spinlock+fence) | 5,088 | 64 | 134 | 36 | 140 | 178 | 0 |
| **PM Thread** | | | | | | | |
| thread (states+manager) | 6,788 | 147 | 160 | 64 | 152 | 238 | 3 |
| **PM Process** | | | | | | | |
| process (states+capability+managers) | 15,259 | 226 | 172 | 195 | 369 | 411 | 30 |
| **PM Clock** | | | | | | | |
| clock | 1,826 | 25 | 37 | 16 | 45 | 53 | 2 |
| **PM System Calls** | | | | | | | |
| pm/kcall (8 syscalls) | 12,297 | 200 | 155 | 48 | 166 | 231 | 33 |
| **Grand Total** | **76,694** | **1,098** | **1,120** | **636** | **1,480** | **1,892** | **134** |

### A.2 Aggregate Statistics

| Metric | Value |
|--------|-------|
| Total Verus verification code | 76,694 |
| Total original source lines verified | ~14,341 |
| Expansion factor | 5.3× |
| Verification wall-clock time | 10.987 seconds |
| SMT solver time (parallel) | 7,784 ms (57 threads) |
| Total SMT rlimit consumed | 56,776,544 |
| Properties verified | 1,857 |
| Assume statements | 1 |
| External_body annotations | 134 |
| Verification files (.rs) | 179 |

## Appendix B: Module Dependency Graph

```
                         error
                           │
                    raw_array
                      │    │
                   bitmap  │
                   │    │  │
                 slab   │  │
                  │     │  │
               kheap    │  │
                        │  │
               frame_address (HAL)
                        │
                    ┌────┤
                    │    │
                 frame   │
                 │    │  │
              upool  kpool
                 │    │
              kstack ustack kredzone
                        │
                 ┌──────┤
                 │      │
              kpage   vmem
                 │      │
              virt_init │
                        │
          ┌─────────────┤
          │             │
     pid, tid      capability (sys)
          │             │
     ┌────┴─────────────┤
     │                  │
  thread states    process states
     │                  │
  thread_manager   process_manager
     │                  │
     ├──────────────────┤
     │                  │
  sync primitives   clock
     │                  │
     └────────┬─────────┘
              │
     kcall dispatcher/handler/scoreboard
              │
     pm/kcall (create_thread, join_thread,
               terminate, lock_mutex,
               unlock_mutex, signal_cond,
               wait_cond, sleep)
```

## Appendix C: Bug Discovery Details

### C.1 Zombie Thread Loss (Bug #1 — Critical)

See Section 5.1 Case Study for detailed analysis.

### C.2 Quantum Inheritance Starvation (Bug #2 — Design Flaw)

**Location**: `kernel/pm/process/state/running.rs::switch()`

**Root Cause**: When the scheduler performs a context switch between threads of the *same* process, the outgoing thread's remaining time quantum is not reset. The incoming thread inherits the full quantum. Over many scheduling cycles, this creates a compounding effect where intra-process thread switches accumulate quantum, resulting in up to 1000:1 execution time ratio between processes with many threads and processes with few threads.

**Impact**: Severe scheduling unfairness in multi-tenant serverless environments.

**Discovery Method**: During specification analysis for the `clock` module, the AI prover noted that the `switch()` function's quantum management did not satisfy fairness invariants. Meeting discussion confirmed this was an intentional but flawed design choice.

### C.3 Mutex Capacity Check (Bug #3 — Logic Ordering)

**Location**: `kernel/pm/process/state/mod.rs::get_mutex()` and `get_cond()`

**Root Cause**: The capacity check (`self.mutex_count >= MAX`) is evaluated *before* the presence check (`self.mutex_addrs.contains(addr)`). When the resource map is full, a request for an *already-registered* mutex is incorrectly rejected with a capacity error instead of returning the existing entry.

**Impact**: Under high resource pressure, existing synchronization primitives become inaccessible, potentially causing deadlocks.

**Discovery Method**: The AI prover identified the ordering issue during verification of the process state module.

### C.4 Slab Underflow (Bug #4 — Arithmetic)

**Location**: `libs/slab/src/lib.rs::from_raw_parts()`

**Root Cause**: The subtraction `total_num_blocks - num_index_blocks` is performed without checking that `total_num_blocks >= num_index_blocks`. When the index blocks exceed the total, the subtraction underflows (wraps in release mode), corrupting the slab's data block count.

**Impact**: Slab metadata corruption leading to out-of-bounds memory access.

**Discovery Method**: Verus requires proving overflow-freedom for all arithmetic; the prover could not discharge the subtraction safety obligation without an added precondition.

### C.5 Bitmap Index Overflow (Bug #5 — Arithmetic)

**Location**: `libs/bitmap/src/lib.rs::index()`

**Root Cause**: The bounds check recomputes `bits.len() * 8` instead of using the stored `number_of_bits` field. For large arrays, this multiplication can overflow, wrapping to a small value and allowing out-of-bounds indices to pass the check.

**Impact**: Out-of-bounds bit manipulation, corrupting adjacent memory.

**Discovery Method**: Verus flagged the unchecked multiplication as a potential overflow.

### C.6 Bitmap Multiplication Overflow (Bug #6 — Arithmetic)

**Location**: `libs/bitmap/src/lib.rs::from_raw_array()`

**Root Cause**: The constructor computes `array.len() * 8` to determine the number of bits. For arrays larger than `usize::MAX / 8`, this multiplication overflows silently, producing an incorrect `number_of_bits` field.

**Impact**: Bitmap tracks fewer bits than the underlying array, leaving the tail portion unmanaged.

**Discovery Method**: Verus overflow check on the multiplication expression.

### C.7 Slab Deallocate Alignment (Bug #7 — Missing Check)

**Location**: `libs/slab/src/lib.rs::deallocate()`

**Root Cause**: The function does not verify that the pointer being freed is aligned to `block_size`. An unaligned pointer produces an incorrect block index via integer division, causing the wrong block to be marked as free.

**Impact**: Silent corruption of allocation state—a different block is freed than intended.

**Discovery Method**: The AI prover attempted to prove that `deallocate` frees the correct block and could not establish the correspondence without an alignment precondition.

### C.8 Slab Deallocate Bounds Overflow (Bug #8 — Arithmetic)

**Location**: `libs/slab/src/lib.rs::deallocate()`

**Root Cause**: The upper bounds check computes `num_data_blocks * block_size` without overflow protection. When this multiplication wraps, the bounds check is effectively disabled, allowing out-of-range pointers to be accepted.

**Impact**: Freeing memory outside the slab's managed region.

**Discovery Method**: Verus overflow check on the multiplication in the bounds comparison.

### C.9 Slab Address Overflow (Bug #9 — Arithmetic)

**Location**: `libs/slab/src/lib.rs::from_raw_parts()`

**Root Cause**: The data area start address is computed as `addr + (num_index_blocks * block_size)`. Neither the multiplication nor the addition is checked for overflow.

**Impact**: Wrapped address points to unrelated memory; all subsequent allocations use incorrect addresses.

**Discovery Method**: Verus flagged the unchecked addition as a potential overflow.

### C.10 Slab Multiplication Overflow (Bug #10 — Arithmetic)

**Location**: `libs/slab/src/lib.rs::from_raw_parts()`

**Root Cause**: The intermediate computation `num_index_blocks * block_size` can overflow independently of the subsequent addition, producing a small offset that passes later bounds checks.

**Impact**: Incorrect slab layout computation; index and data regions overlap.

**Discovery Method**: Verus overflow check on the multiplication expression.

### C.11 Address Overflow (Bug #11 — Arithmetic)

**Location**: `libs/slab/src/lib.rs`

**Root Cause**: Base address computation in the slab allocator can overflow when the slab is initialized with a high base address and large block parameters.

**Impact**: Wrapped base address leads to memory access outside the intended region.

**Discovery Method**: Verus overflow check during slab construction verification.

---

*This paper describes research conducted in 2025-2026. The Nanvix project is open source and available at https://github.com/nanvix/nanvix.*
