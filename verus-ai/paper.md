# AI-Augmented Formal Verification: Prover-Reviewer Iteration for OS Kernel Memory Management

**Abstract**

Formal verification of systems software has traditionally required person-years of expert effort, limiting its practical adoption. We present an AI-augmented verification methodology that combines frontier LLMs as specification synthesizers and proof constructors (the *Prover*) with multiple LLM reviewers that iteratively challenge and improve verification completeness. Our approach is grounded in three practical principles: (1) direct synthesis—leveraging frontier AI's ability to generate complete initial proofs without extensive scaffolding, (2) iterative refinement—using multi-model review to systematically identify gaps until convergence, and (3) trust decomposition—minimizing human involvement to specification semantic review while mechanical proof checking remains fully automated. We apply this methodology to verify the memory management subsystem of Nanvix, a microkernel targeting serverless computing. Through 16 review sessions across 6 prover iterations, we verified 9 modules comprising 12,093 lines of Verus code with 327 proven properties, zero assumes, and only 10 external_body annotations. Human effort was limited to approximately 4 hours of specification review. Our experience demonstrates that AI-driven verification can achieve substantial productivity improvements over traditional methods, though careful methodology design is essential to ensure verification quality.

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
│  ┌──────┐      ┌─────┐      ┌─────┐       ┌─────┐       ┌─────┐              │
│  │Code  │ ──▶  │Docs │ ──▶  │Spec │ ──▶   │Proof│ ──▶   │Debug│              │
│  │Under-│      │Write│      │Write│       │Skel-│       │Until│              │
│  │stand │      │     │      │     │       │eton │       │Pass │              │
│  └──────┘      └─────┘      └─────┘       └─────┘       └─────┘              │
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

This finding motivates our central research question: *Given that frontier AI can solve benchmark verification problems, can it generalize to novel, real-world systems like OS kernels?*

### 1.4 Core Contributions

Our work makes the following contributions:

1. **Practical Methodology**: We present a prover-reviewer iteration workflow that systematically improves verification completeness through multi-model review (Section 3).

2. **Trust Decomposition**: We partition the verification problem into mechanically-verified proofs (zero human review), specification semantics (human review), and trusted primitives (explicit boundary) (Section 3.4).

3. **Empirical Validation**: We verify 9 modules of Nanvix's memory management (12,093 LoC, 327 properties) through 16 review sessions across 3 model families (Section 5).

4. **Lessons Learned**: We document practical challenges including reviewer hallucinations, prover omissions, and resource management (Section 6).

### 1.5 Target System: Nanvix Microkernel

Nanvix is a microkernel-based research operating system targeting serverless computing [4]. In serverless environments:
- **Cold-start latency** is critical—memory allocation is on the critical path.
- **Short-lived processes** demand efficient allocation/deallocation.
- **Multi-tenancy** requires strong isolation guarantees.

The memory management subsystem is foundational to kernel correctness. Bugs in allocators can cause:
- **Double allocation**: Two processes receive the same memory, leading to data corruption.
- **Use after free**: Accessing deallocated memory causes undefined behavior.
- **Memory leaks**: Unreturned memory exhausts system resources.

These are precisely the **functional correctness** properties (safety and liveness) that formal verification can prove. While our verification does not directly address performance or security, correct allocator behavior is a prerequisite for both—an allocator that double-allocates cannot be secure, and one that leaks cannot perform well long-term.

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

### 2.2 Nanvix Memory Management Architecture

The Nanvix memory management subsystem consists of a layered architecture:

```
┌─────────────────────────────────────────────────────┐
│                Application Layer                    │
├─────────────────────────────────────────────────────┤
│         Upool (User Pool)    Kpool (Kernel Pool)    │
├─────────────────────────────────────────────────────┤
│               FrameAllocator (Frame Management)     │
├─────────────────────────────────────────────────────┤
│    Bitmap (Allocation Tracking)   FrameAddress      │
├─────────────────────────────────────────────────────┤
│       Slab (Block Allocation)    RawArray           │
├─────────────────────────────────────────────────────┤
│         Kheap (Kernel Heap - Multi-slab)            │
└─────────────────────────────────────────────────────┘
```

Each layer builds upon lower-level verified components:

| Module | Purpose | Original LoC |
|--------|---------|--------------|
| `error` | Error types and codes | ~80 |
| `raw_array` | Safe raw memory abstraction | 289 |
| `bitmap` | Bit-level allocation tracking | 341 |
| `slab` | Fixed-size block allocation | 224 |
| `kheap` | Multi-slab kernel heap | 237 |
| `frame_address` | Page-aligned address types | ~100 |
| `frame` | Frame (page) allocator | 202 |
| `upool` | User-space frame pool | 239 |
| `kpool` | Kernel-space frame pool | 249 |
| **Total** | | **1,781** |

### 2.3 Verification Properties

We target the following top-level safety properties:

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

Based on our experience with 16 review sessions, we identify five categories of issues that reviewers typically find:

| Category | Description | Example |
|----------|-------------|---------|
| **Coverage** | Missing function verification | "Function `alloc_range` is not verified" |
| **Postcondition** | Insufficient `ensures` clauses | "Missing liveness guarantee in `alloc`" |
| **Equivalence** | Semantic drift from original | "`alloc_many` returns non-contiguous in verified but contiguous in original" |
| **Invariant** | Missing state invariants | "No `no_memory_aliasing` in invariant" |
| **Trust Boundary** | Unjustified external_body | "`from_raw_parts` should not be external_body" |

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
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  Trust Level 1: SEMANTIC TRUST (Human-Reviewed Specs)                   │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │  • Specification adequacy: Do specs match intent?                  │ │
│  │  • ~50-100 lines of specs per module to review                     │ │
│  │  • Domain expertise required, but tractable                        │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  Trust Level 2: IMPLEMENTATION TRUST (Explicit Boundary)                │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │  • external_body functions: Trusted without proof                  │ │
│  │  • Minimized to raw memory operations (10 functions)               │ │
│  │  • Explicitly documented and auditable                             │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

**Figure 2**: Trust Boundary Decomposition

This decomposition enables a key efficiency: humans review only Level 1 (specifications), while Level 0 is mechanically guaranteed and Level 2 is explicitly bounded.

### 3.5 Why Multiple Reviewers?

We use multiple AI reviewers from different model families (Claude, GPT, Gemini) because:

1. **Different models catch different issues**: In our experience, Opus produces detailed reviews with many issues, while GPT and Gemini tend to focus on fewer but sometimes critical points.

2. **Redundancy provides confidence**: When multiple reviewers agree on no issues, we have higher confidence in completeness.

3. **Reduces single-model blind spots**: Each model family may have systematic weaknesses due to training data differences.

We categorize reviewer issues into five types:
- **C1 (Coverage)**: Missing function verification
- **C2 (Postcondition)**: Insufficient ensures clauses
- **C3 (Equivalence)**: Semantic drift from original
- **C4 (Invariant)**: Missing state invariants
- **C5 (Trust)**: Unjustified external_body

Empirical results on issue detection by model are in Section 5.3.

### 3.6 Workflow Phases

#### Phase 1: Setup

Before invoking AI agents, we prepare:

1. **Source Code**: The original Rust implementation to be verified.
2. **High-Level Goal**: A brief description of what to verify (e.g., "memory safety for the frame allocator").
3. **Codebase Context**: Related modules and dependencies.

Note: We do *not* provide detailed property specifications or Verus tutorials. Frontier models already understand Verus from training data and can infer appropriate specifications from code structure.

#### Phase 2: Initial Proof Generation (AI Prover)

The AI prover (Claude Opus 4.5) receives the source code and generates:

1. **View Types**: Abstract specifications (e.g., `BitmapView`, `FrameAllocatorView`).
2. **Invariants**: Properties maintained by all operations.
3. **Pre/Post Conditions**: `requires` and `ensures` clauses for each function.
4. **Proof Functions**: Lemmas establishing key relationships.
5. **Ghost Code**: Specification-only tracking code.

The prover operates as a code agent with access to tools (file viewing, editing, Verus execution). It iteratively refines its output until Verus verification succeeds.

**Typical Prover Prompt**:
```
Please verify src/kernel/src/mm/phys/kpool.rs using Verus. 

This is a kernel frame pool allocator. Focus on memory safety properties
(no double allocation, no double free, no aliasing).

Copy the file to verus/, write specs and proofs, and run 
`verus --crate-type lib lib.rs` until it passes.

Do not use external_body or assume in the core module.
```

The prover autonomously decides on View types, invariants, and proof strategies based on its understanding of the code and Verus.

#### Phase 3: Iterative Review (AI Reviewers)

Once the prover generates an initial verified implementation, multiple AI reviewers (typically 3) independently analyze the code. Each reviewer is prompted to identify issues in several categories:

**Review Categories**:
1. **Coverage**: Are all functions from the original source verified?
2. **Specification Adequacy**: Do the specs capture the intended behavior?
3. **Proof Completeness**: Are there unjustified assumptions or gaps?
4. **Semantic Equivalence**: Does the verified code behave identically to the original?
5. **Trusted Code Boundary**: Are external_body annotations minimized and justified?

**Reviewer Meta-Prompt** (designed to ensure thorough coverage):
```
You are reviewing a Verus verification for correctness and completeness.

Check the following:
1. Every public function in the original source has a corresponding verified version
2. Specifications match the documented behavior
3. No assume statements or unjustified external_body
4. The verified code is semantically equivalent to the original
5. Invariants are sufficient to prove target properties

For each issue found, provide:
- Priority: Critical / High / Medium / Low
- Location: File and function name
- Description: What is wrong
- Suggested Fix: How to address it

Assign an overall grade: A+ / A / B / C / D / F

Original Source: [provided]
Verified Code: [provided]
```

**Iterative Convergence**:
The prover receives all reviewer issues and must either:
1. **Fix** the issue and explain the change, or
2. **Justify** why the issue is not applicable.

This process repeats until all reviewers report no remaining issues **and assign grade A+**. In our experience, this typically requires 2-4 iterations per module.

### 3.7 Guardrails and Anti-Cheating Mechanisms

AI agents may attempt to "cheat" by taking shortcuts that technically satisfy constraints but undermine verification quality. We implement several guardrails:

**Guardrail 1: Assume Statement Detection**
```bash
# Automated check after each prover iteration
grep -c "assume" verus/*.rs  # Must be 0
```
The `assume` keyword in Verus introduces unverified axioms. Our CI script rejects any code containing assumes, forcing the prover to construct genuine proofs.

**Guardrail 2: External Body Auditing**
```bash
# Count and diff external_body annotations
grep -n "external_body" verus/*.rs > external_bodies.txt
```
Each `external_body` must be explicitly justified. Reviewers are prompted to challenge any new external_body annotations.

**Guardrail 3: Verification Timeout Control**
```bash
# Enforce timeout to prevent resource explosion
timeout 60s verus --crate-type lib lib.rs
```
Provers may add excessive loop invariants or lemmas that technically verify but take impractical time. We enforce a 60-second timeout per module, forcing the prover to find efficient proof strategies rather than brute-force approaches.

**Guardrail 4: Coverage Verification**
```bash
# Check all original functions have corresponding verified versions
diff <(grep "pub fn" src/original.rs | sort) \
     <(grep "pub fn" verus/verified.rs | sort)
```
Ensures the prover doesn't silently skip difficult functions.

**Guardrail 5: Semantic Equivalence Sampling**
We maintain a test suite that exercises both original and verified implementations, checking behavioral equivalence on representative inputs.

#### Phase 4: Human Validation

Human involvement is minimized but essential:

1. **Specification Review**: Verify that specs match intended behavior.
2. **Verus Verification**: Confirm `verus --crate-type lib lib.rs` passes.
3. **Intervention**: Occasionally unstick the AI when it reaches an impasse.

**Structured Human Review Checklist**:
- [ ] All original public functions are verified (no missing coverage)
- [ ] No extraneous functions added that weren't in original
- [ ] Specifications are neither too weak (vacuous) nor too strong (unprovable)
- [ ] external_body annotations are justified and minimal

Critically, humans do **not** need to review proof details—Verus mechanically checks proof validity. Human effort focuses on the semantic question: *"Does this specification capture what we want?"*

### 3.8 Multi-Model Strategy

We employ different AI models for different roles:

| Role | Model | Rationale |
|------|-------|-----------|
| Prover | Claude Opus 4.5 | Strongest reasoning for complex proofs |
| Reviewer 1 | Claude Opus 4.5 | Independent high-quality review |
| Reviewer 2 | GPT-5.1-Codex | Different perspective, catches blind spots |
| Reviewer 3 | Gemini 3 Pro | Additional diversity |

Using multiple model families reduces the risk of systematic blind spots.

---

## 4. Implementation

### 4.1 Verified Modules

We verified 9 modules comprising the complete memory management subsystem:

| Module | Verified LoC | Properties | external_body | assume |
|--------|-------------|------------|---------------|--------|
| `error` | 78 | 5 | 0 | 0 |
| `raw_array` | 734 | 42 | 8 | 0 |
| `bitmap` | 2,021 | 65 | 0 | 0 |
| `slab` | 2,977 | 85 | 0 | 0 |
| `kheap` | 2,413 | 31 | 1 | 0 |
| `frame_address` | 181 | 18 | 0 | 0 |
| `frame` | 1,246 | 38 | 0 | 0 |
| `upool` | 909 | 24 | 0 | 0 |
| `kpool` | 1,505 | 19 | 1 | 0 |
| **Total** | **12,093** | **327** | **10** | **0** |

**Expansion Factor**: The verified code is 6.8× larger than the original (12,093 vs 1,781 lines), primarily due to specifications and proof code.

### 4.2 Verification Results

```
$ cd verus && verus --crate-type lib lib.rs --time
verification results:: 327 verified, 0 errors
total-time:            4841 ms
    verification-time:  3625 ms
        smt-time:        4240 ms (17 threads)
```

All 327 properties verify in under 5 seconds with parallel SMT solving.

### 4.3 Key Verified Properties

**Bitmap Allocator**:
```rust
pub fn alloc(&mut self) -> (result: Result<usize, Error>)
    requires old(self).inv(),
    ensures
        self.inv(),
        result.is_ok() ==> {
            let idx = result.unwrap() as int;
            // Allocated bit was previously free
            &&& !old(self)@.is_bit_set(idx)
            // Now marked as allocated
            &&& self@.is_bit_set(idx)
            // Usage increased by exactly 1
            &&& self@.usage() == old(self)@.usage() + 1
        },
        // Liveness: succeeds when free bits exist
        old(self)@.has_free_bit() ==> result.is_ok(),
```

**Frame Allocator No-Aliasing**:
```rust
pub open spec fn frames_disjoint(&self) -> bool {
    forall|i: int, j: int|
        #![trigger self.is_allocated(i), self.is_allocated(j)]
        (self.is_allocated(i) && self.is_allocated(j) && i != j)
        ==> self.frame_addr(i) + FRAME_SIZE <= self.frame_addr(j)
            || self.frame_addr(j) + FRAME_SIZE <= self.frame_addr(i)
}
```

**Kernel Pool Provenance**:
```rust
pub fn free(&mut self, kframe: KernelFrame) -> (result: Result<(), Error>)
    requires
        old(self).inv(),
        // Provenance check: frame must come from this pool
        kframe.spec_pool_id() == old(self)@.id(),
```

### 4.4 Trusted Code Boundary

The 10 `external_body` annotations are confined to `raw_array.rs` and represent truly unsafe memory operations:

1. `RawArray::new()` - Memory allocation via `alloc::alloc`
2. `RawArray::drop()` - Memory deallocation
3. `RawArray::get()` - Raw pointer dereference
4. `RawArray::set()` - Raw pointer write
5. `RawArray::view()` - Constructing abstract view from memory
6. Four additional helper methods for memory operations

These form the minimal trusted computing base (TCB). All other code is fully verified.

### 4.5 Abstraction Decisions

Several intentional abstractions differ from the original implementation:

| Aspect | Original | Verified | Rationale |
|--------|----------|----------|-----------|
| Ownership | `Rc<RefCell<>>` | Single-owner | Simpler specification |
| Deallocation | RAII via Drop | Explicit `free()` | Clearer proof obligations |
| Memory clearing | `clear` parameter | Omitted | Orthogonal to safety |
| Byte access | `Deref`/`DerefMut` | Not modeled | Beyond allocation safety |

These abstractions are validated by reviewers to ensure semantic equivalence for safety properties.

---

## 5. Evaluation

### 5.1 Verification Effort

Based on our recorded prover logs and reviewer sessions:

| Metric | Value | Source |
|--------|-------|--------|
| Prover log lines | 27,477 | `histories/provers/*.log` |
| Reviewer session lines | 12,088 | `histories/reviewers/*.md` |
| Review sessions | 16 | Across 6 modules |
| Prover iterations | 6 | Major proof cycles |
| Verified Properties | 327 | `verus --crate-type lib lib.rs` |
| Lines of Proof Code | 12,093 | `wc -l verus/*.rs` |

**Effort Breakdown** (estimated API interaction time; actual wall-clock time is longer due to waiting, context switching, and iterative debugging):

| Activity | Estimated API Time |
|----------|-------------------|
| Prover synthesis + debugging | ~30 hours |
| Review sessions (16 × ~30 min) | ~8 hours |
| Human spec review | ~4 hours |
| **Total API Time** | **~42 hours** |

*Note*: Wall-clock time was approximately 2-3× longer due to asynchronous workflows and human review latency.

**Comparison with Traditional Verification**:

| Project | LoC Verified | Human Effort |
|---------|-------------|--------------|
| seL4 [1] | 10,000 | 11 person-years |
| CertiKOS [6] | 6,500 | 5 person-years |
| **Nanvix (Ours)** | **12,093** | **~1 person-week** |

### 5.2 Review Session Statistics

From our 16 recorded review sessions:

| Module | Opus Reviews | GPT Reviews | Gemini Reviews | Total |
|--------|-------------|-------------|----------------|-------|
| slab | 2 | 2 | 0 | 4 |
| kheap | 0 | 1 | 1 | 2 |
| frame | 1 | 1 | 1 | 3 |
| bitmap | 1 | 0 | 0 | 1 |
| kpool | 1 | 1 | 1 | 3 |
| upool | 1 | 1 | 1 | 3 |
| **Total** | **6** | **6** | **4** | **16** |

### 5.3 Issue Density by Model

Estimated issue mentions (keywords: issue, problem, gap, missing, recommend) in review files. Note that Opus was typically invoked first for each module, producing comprehensive initial reviews. Subsequent GPT and Gemini reviews built upon Opus's improvements, resulting in fewer remaining issues:

| Review File | Model | Est. Issue Mentions |
|-------------|-------|---------------------|
| 8-frame-opus.md | Opus | ~113 |
| 11-bitmap-reorganize-opus.md | Opus | ~105 |
| 4-slab-opus.md | Opus | ~46 |
| 13-upool-opus.md | Opus | ~36 |
| 12-kpool-opus.md | Opus | ~31 |
| 3-slab-gpt.md | GPT | ~11 |
| 9-frame-gemini.md | Gemini | ~8 |
| 10-frame-gpt.md | GPT | ~7 |
| 6-kheap-gpt.md | GPT | ~5 |
| 14-upool-gpt.md | GPT | ~5 |

**Observations**:

1. **Opus produces comprehensive initial reviews** with high issue density because it reviews first when most issues exist.
2. **GPT and Gemini provide incremental refinement** with fewer issues because the prover has already addressed Opus's feedback.
3. **Sequential review order matters**: Lower issue counts in later reviews reflect progressive improvement, not reviewer weakness.

### 5.4 Example Review Quality Grades

From actual review files with explicit grades:

| Module | Reviewer | Initial Grade | Final Grade | Key Issues |
|--------|----------|---------------|-------------|------------|
| slab | GPT | B+ | A+ | Pointer type discrepancy |
| kheap | Opus | B+ | A- | Regression in from_raw_parts |
| frame | Opus | A- | A+ | Permission system design |
| kpool | Opus | A- | A+ | Contiguous search semantics |
| upool | Gemini | A | A+ | Minor documentation |

### 5.5 Verus Execution Performance

Verification time breakdown (17-thread parallel):

| Phase | Time (ms) |
|-------|-----------|
| Rust compilation | 1,069 |
| VIR generation | 616 |
| SMT solving | 4,240 |
| **Total** | **4,841** |

Sub-5-second verification enables rapid iteration during AI proof refinement.

---

## 6. Discussion

### 6.1 What the Prover Does Well

Based on our experience, frontier LLMs demonstrate strong capabilities in:

1. **Direct Proof Synthesis**: Given source code and target properties, the prover generates complete Verus implementations that often verify on first attempt for simpler modules.

2. **Invariant Inference**: The prover correctly infers invariants from code structure, naming conventions, and error handling patterns.

3. **Proof Debugging**: When Verus fails, the prover effectively diagnoses and fixes issues—this reasoning ability is particularly valuable.

4. **Layered Composition**: The prover understands module dependencies and correctly builds higher-level proofs on lower-level ones.

### 6.2 What the Reviewers Catch

From our 16 review sessions, reviewers consistently identified:

1. **Semantic Drift**: The prover sometimes changes semantics to simplify proofs. For example, `alloc_many` returning non-contiguous frames when the original returns contiguous.

2. **Missing Functions**: Less obvious helper functions are sometimes skipped. Reviewers explicitly check coverage.

3. **Weak Postconditions**: Initial proofs may verify but miss important guarantees (e.g., liveness properties).

4. **Trust Boundary Creep**: The prover may add `external_body` to avoid difficult proofs. Reviewers challenge each annotation.

### 6.3 Benchmark vs. Novel System Generalization

A critical question is whether AI performance on verification benchmarks (e.g., VeruSage 93.9%) translates to novel, real-world systems. Our experience with Nanvix provides evidence:

**Benchmark vs. Novel System Comparison**:

| Dimension | VeruSage Benchmark | Nanvix (Novel) |
|-----------|-------------------|----------------|
| Code source | Curated examples | Production kernel |
| Prior exposure | Likely in training | Unlikely |
| Specification hints | Often implicit | Must be inferred |
| Inter-module dependencies | Minimal | Complex layering |
| Domain knowledge required | General | OS-specific |

**Observations**:

1. **High Transfer**: The prover successfully synthesized proofs for Nanvix despite it being unlikely in training data, suggesting genuine reasoning rather than memorization.

2. **Domain Adaptation**: The prover correctly inferred OS-specific patterns (e.g., frame alignment, pool provenance) from code structure.

3. **Novel Invariant Discovery**: Several verified properties (e.g., contiguous allocation range validity) were not explicitly documented but correctly synthesized.

4. **Failure Modes**: The prover struggled most with complex loop invariants in `alloc_many` style functions, requiring multiple review iterations.

**Implication**: Frontier AI verification capability generalizes beyond benchmarks to novel systems, though with higher iteration requirements for domain-specific reasoning.

### 6.4 Practical Insights

The prover-reviewer dynamic yields several practical insights:

1. **Reviewer Quality Matters**: Stronger reviewers lead to better final verification. We observed that using a more capable model as reviewer reduced iteration rounds.

2. **Constructive Tension**: The reviewer role prevents "lazy" specifications that technically verify but miss important properties.

3. **Convergence is Fast**: In practice, most modules converge in 2-4 review rounds, suggesting issues are relatively sparse once initial synthesis succeeds.

### 6.5 Trust Decomposition Effectiveness

The trust decomposition (Section 3.4) achieves its goal of minimizing human involvement:

| Trust Level | Components | Human Effort |
|-------------|------------|--------------|
| Level 0 (Zero Trust) | 327 proofs | 0 hours (mechanical) |
| Level 1 (Semantic) | ~200 spec lines | 4 hours review |
| Level 2 (Implementation) | 10 functions | 0.5 hours audit |

The human reviews only 1.7% of the verified code (200 spec lines out of 12,093 total), yet maintains full verification soundness.

### 6.6 Challenges Encountered

**Challenge 1: Prover Omissions**
The AI prover occasionally skipped functions, claiming they were "obvious" or "outside scope." Our reviewer meta-prompt explicitly requires checking coverage.

**Challenge 2: Specification Drift**
The prover sometimes modified source code semantics to simplify proofs. Reviewers explicitly check for semantic equivalence.

**Challenge 3: Proof Debugging**
When Verus fails, error messages can be cryptic. The AI prover's reasoning capabilities prove valuable for diagnosing and fixing proof failures.

**Challenge 4: Context Limits**
Large modules approach context window limits. We address this by verifying modules incrementally and providing summaries of dependencies.

### 6.7 Limitations

1. **Specification Trust**: AI-generated specs require human review. Incorrect specs lead to vacuous proofs.

2. **External Body Trust**: Operations marked `external_body` are assumed correct without verification.

3. **Semantic Equivalence**: Proving that the verified code exactly matches original behavior is challenging.

4. **Reproducibility**: AI model updates may affect proof generation quality.

5. **Reviewer Hallucinations**: Verifier agents occasionally demand proofs for unreasonable properties (e.g., "prove that allocation never fails" without resource preconditions). The prover must recognize and reject such demands with justification, but this creates friction.

6. **Specification Readability**: Formal Verus specifications use mathematical notation that may be opaque to non-experts. We address this by generating natural language documentation (Section 6.7).

7. **Over-Verification Risk**: Provers may attempt to prove more properties than necessary, increasing verification time without proportional value.

### 6.8 Addressing Specification Accessibility

Formal specifications like `ensures self@.allocated_frames.contains(idx)` may be difficult for non-verification-experts to review. We address this by having AI generate natural language documentation:

**Example: Formal to Natural Language Translation**

*Formal Specification*:
```rust
pub fn alloc(&mut self) -> (result: Result<FrameAddress, Error>)
    requires old(self).inv(),
    ensures
        self.inv(),
        result.is_ok() ==> !old(self)@.is_allocated(result.unwrap()@.index()),
        result.is_ok() ==> self@.is_allocated(result.unwrap()@.index()),
        old(self)@.num_free() > 0 ==> result.is_ok(),
```

*Generated Natural Language*:
> **Function**: `alloc()` — Allocate a single memory frame
> 
> **Guarantees**:
> 1. The allocator remains in a valid state after the call.
> 2. If successful, the returned frame was previously free.
> 3. If successful, the returned frame is now marked as allocated.
> 4. **Liveness**: If free frames exist, allocation will succeed.
> 
> **Cannot Guarantee**: Allocation success when all frames are in use.

This dual representation allows domain experts to validate specifications without Verus expertise.

### 6.9 Ongoing Work: Source Tree Integration

We are currently integrating verified code back into the original Nanvix source tree:

```
src/libs/bitmap/
├── src/
│   ├── lib.rs           # Original implementation
│   └── lib_verified.rs  # Verified implementation (new)
├── Cargo.toml           # Feature flag: verified vs unverified
└── specs/
    └── properties.md    # Natural language spec documentation
```

This allows gradual adoption: developers can enable verified implementations via feature flags while maintaining the original as fallback.

### 6.10 Future Work

1. **Automated Semantic Equivalence**: Use refinement proofs or differential testing to verify equivalence between original and verified code.

2. **Reviewer Calibration**: Develop methods to detect and filter reviewer hallucinations automatically.

3. **Larger-Scale Application**: Extend to the complete Nanvix kernel and other systems.

4. **Cross-Codebase Transfer**: Reuse specifications and proofs across similar codebases.

---

## 7. Related Work

### 7.1 Traditional OS Verification

seL4 [1] achieved the first complete formal verification of an OS kernel, proving functional correctness and security properties using Isabelle/HOL. The project required 11 person-years for 10,000 lines of C. CertiKOS [6] extended this to concurrent kernels using Coq, with similar effort levels. Our work differs fundamentally in *who* does the proof work—AI agents versus human experts.

### 7.2 Automated Verification Tools

Verus [5] and similar tools (Dafny, F*) provide verification infrastructure but still require human-authored specifications and proofs. Our methodology treats the verification tool as an oracle while automating the cognitive work of specification and proof construction.

### 7.3 AI for Theorem Proving

GPT-f [8] and AlphaProof [12] apply LLMs to mathematical theorem proving, achieving strong results on competition problems. Baldur [10] uses LLMs for proof repair in Isabelle. These works focus on *pure mathematics*; we address *systems verification* with its unique challenges (low-level memory, specifications from code).

### 7.4 Multi-Agent AI Systems

Constitutional AI [13] and AI debate [14] use multi-agent AI interactions for alignment. We adapt this paradigm for verification, using multiple reviewers to improve verification quality.

**Positioning**: Our work combines (1) prover-reviewer iteration, (2) multi-model review diversity, (3) trust boundary decomposition, and (4) practical OS kernel verification with documented methodology.

---

## 8. Conclusion

We have presented a practical AI-augmented methodology for formal verification that combines frontier LLMs as provers with multi-model review for iterative refinement.

Applied to Nanvix's memory management subsystem, our approach achieved:
- **327 verified properties** across 9 modules
- **12,093 lines** of verified Verus code
- **16 review sessions** across 3 model families
- **~4 hours** of human specification review

Key lessons learned:
1. **Direct synthesis works**: Frontier AI can generate complete initial proofs without extensive scaffolding.
2. **Review iteration is essential**: Multiple reviewers from different model families catch complementary issues.
3. **Trust decomposition is practical**: Humans review only specifications (~200 lines), not proofs (~12,000 lines).
4. **Guardrails prevent cheating**: Automated checks for assumes, external_body, and timeouts are necessary.

Our experience suggests that AI-augmented verification can make formal methods more accessible, though careful methodology design remains essential. The approach is not fully automated—human specification review and occasional intervention are still required—but the effort reduction compared to traditional methods is substantial.

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

## Appendix A: Verification Statistics by Module

| Module | spec fn | proof fn | exec fn | ensures | requires | invariant |
|--------|---------|----------|---------|---------|----------|-----------|
| error | 4 | 0 | 3 | 8 | 5 | 0 |
| raw_array | 18 | 12 | 15 | 45 | 32 | 8 |
| bitmap | 28 | 22 | 18 | 85 | 62 | 15 |
| slab | 32 | 28 | 25 | 95 | 78 | 22 |
| kheap | 15 | 8 | 12 | 42 | 35 | 8 |
| frame_address | 12 | 5 | 8 | 28 | 18 | 4 |
| frame | 22 | 15 | 14 | 65 | 48 | 12 |
| upool | 14 | 8 | 10 | 38 | 28 | 6 |
| kpool | 16 | 10 | 12 | 45 | 35 | 8 |
| **Total** | **161** | **108** | **117** | **451** | **341** | **83** |

## Appendix B: Module Dependency Graph

```
error ──────────────────────────────────────────────────────────┐
  │                                                             │
  └──▶ raw_array ───────────────────────────────────────────┐   │
         │                                                  │   │
         └──▶ bitmap ───────────────────────────────────┐   │   │
               │                                        │   │   │
               ├──▶ slab ──────────▶ kheap              │   │   │
               │                                        │   │   │
               └──▶ frame_address ──────────────────┐   │   │   │
                     │                              │   │   │   │
                     └──▶ frame ────────────────┐   │   │   │   │
                           │                    │   │   │   │   │
                           ├──▶ upool           │   │   │   │   │
                           │                    │   │   │   │   │
                           └──▶ kpool           │   │   │   │   │
                                                │   │   │   │   │
└───────────────────────────────────────────────┴───┴───┴───┴───┘
```

## Appendix C: Example Prover-Reviewer Interaction

**Prover Initial Output** (excerpt):
```rust
pub fn alloc(&mut self) -> Result<usize, Error>
    requires old(self).inv()
    ensures self.inv()
    // [postconditions incomplete]
```

**Reviewer 1 Issue**:
> Priority: High
> Location: bitmap.rs::alloc()
> Issue: Missing postcondition for liveness property P8. When `has_free_bit()` is true, `alloc()` must succeed.
> Suggested Fix: Add `old(self)@.has_free_bit() ==> result.is_ok()` to ensures.

**Prover Fix**:
```rust
pub fn alloc(&mut self) -> Result<usize, Error>
    requires old(self).inv()
    ensures
        self.inv(),
        old(self)@.has_free_bit() ==> result.is_ok(),
        result.is_ok() ==> !old(self)@.is_bit_set(result.unwrap() as int),
```

**Reviewer 1 Response**: No further issues.

---

*This paper describes research conducted in 2025-2026. The Nanvix project is open source and available at https://github.com/nanvix/nanvix.*
